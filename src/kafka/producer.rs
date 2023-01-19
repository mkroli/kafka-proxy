/*
 * Copyright 2022 Michael Krolikowski
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::time::Duration;

use anyhow::Result;
use opentelemetry::metrics::{Counter, Meter};
use opentelemetry::KeyValue;
use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

use crate::cli::Producer;
use crate::kafka::schema_registry::SchemaRegistry;
use crate::kv;
use crate::metrics::counter_inc;

const TIMEOUT: Timeout = Timeout::After(Duration::from_millis(3000));

pub struct KafkaProducer {
    topic: String,
    producer: FutureProducer,
    schema_registry: Option<SchemaRegistry>,
    producer_requests_counter: Counter<u64>,
    producer_sent_counter: Counter<u64>,
}

impl KafkaProducer {
    pub async fn new(cfg: Producer, meter: Meter) -> Result<KafkaProducer> {
        let client_config = cfg.client_config(vec![
            ("client.id", "kafka-proxy"),
            ("bootstrap.servers", &cfg.bootstrap_server),
        ]);
        let producer: FutureProducer = client_config.create()?;

        let schema_registry = match cfg.schema_registry_url {
            None => None,
            Some(url) => Some(SchemaRegistry::new(url, cfg.topic.clone()).await?),
        };

        let producer_requests_counter = meter
            .u64_counter("producer.requests")
            .with_description("Number of requests")
            .init();
        let producer_sent_counter = meter
            .u64_counter("kafka.produced")
            .with_description("Number of produced Kafka Records")
            .init();

        Ok(KafkaProducer {
            topic: cfg.topic,
            producer,
            schema_registry,
            producer_requests_counter,
            producer_sent_counter,
        })
    }

    async fn encode(&self, payload: &[u8]) -> Result<Vec<u8>> {
        match &self.schema_registry {
            None => Ok(Vec::from(payload)),
            Some(schema_registry) => schema_registry.encode(payload).await,
        }
    }

    async fn produce<K>(&self, key: &K, payload: &[u8]) -> Result<()>
    where
        K: ToBytes + ?Sized,
    {
        let payload = self.encode(payload).await?;
        let record = FutureRecord::to(&self.topic).key(key).payload(&payload);
        self.producer
            .send(record, TIMEOUT)
            .await
            .map_err(|(e, _)| e)?;
        Ok(())
    }

    pub async fn send<K>(&self, key: &K, payload: &[u8]) -> Result<()>
    where
        K: ToBytes + ?Sized,
    {
        match self.produce(key, payload).await {
            Ok(()) => {
                counter_inc(&self.producer_requests_counter, kv!["success" => true]);
                counter_inc(&self.producer_sent_counter, kv![]);
                Ok(())
            }
            Err(e) => {
                counter_inc(&self.producer_requests_counter, kv!["success" => false]);
                Err(e)
            }
        }
    }
}
