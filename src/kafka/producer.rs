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
use base64::Engine;
use opentelemetry::metrics::{Counter, Meter};
use opentelemetry::KeyValue;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::cli::Producer;
use crate::kafka::schema_registry::SchemaRegistry;
use crate::kafka::telemetry_client_context::TelemetryClientContext;
use crate::metrics::counter_inc;
use crate::{kv, ENGINE};

const TIMEOUT: Timeout = Timeout::After(Duration::from_millis(3000));

pub struct KafkaProducer {
    topic: String,
    producer: FutureProducer<TelemetryClientContext>,
    schema_registry: Option<SchemaRegistry>,
    dead_letters: Option<Mutex<File>>,
    producer_requests_counter: Counter<u64>,
    producer_sent_counter: Counter<u64>,
}

impl KafkaProducer {
    pub async fn new(cfg: Producer, meter: &Meter) -> Result<KafkaProducer> {
        let client_config = cfg.client_config(vec![
            ("client.id", "kafka-proxy"),
            ("bootstrap.servers", &cfg.bootstrap_server),
            (
                "statistics.interval.ms",
                &crate::metrics::COLLECT_PERIOD_MS.to_string(),
            ),
        ]);
        let context = TelemetryClientContext::new(meter)?;
        let producer: FutureProducer<TelemetryClientContext, _> =
            client_config.create_with_context(context)?;

        let schema_registry = match &cfg.schema_registry.schema_registry_url {
            None => None,
            Some(_) => Some(SchemaRegistry::new(cfg.topic.clone(), &cfg.schema_registry).await?),
        };

        let dead_letters = match cfg.dead_letters {
            None => None,
            Some(path) => {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .await?;
                Some(Mutex::new(file))
            }
        };

        let producer_requests_counter = meter
            .u64_counter("kafkaproxy.requests")
            .with_description("Number of requests")
            .build();
        let producer_sent_counter = meter
            .u64_counter("kafkaproxy.produced")
            .with_description("Number of produced Kafka Records")
            .build();

        Ok(KafkaProducer {
            topic: cfg.topic,
            producer,
            schema_registry,
            dead_letters,
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

    async fn produce(&self, payload: &[u8]) -> Result<()> {
        let payload = self.encode(payload).await?;
        let record: FutureRecord<Vec<u8>, Vec<u8>> =
            FutureRecord::to(&self.topic).payload(&payload);
        self.producer
            .send(record, TIMEOUT)
            .await
            .map_err(|(e, _)| e)?;
        Ok(())
    }

    async fn dead_letter(&self, payload: &[u8]) -> Result<()> {
        if let Some(file) = &self.dead_letters {
            let mut str = ENGINE.encode(payload);
            str.push('\n');
            let mut file = file.lock().await;
            file.write_all(str.as_bytes()).await?;
        };
        Ok(())
    }

    pub async fn send(&self, payload: &[u8]) -> Result<()> {
        match self.produce(payload).await {
            Ok(()) => {
                counter_inc(&self.producer_requests_counter, kv!["success" => true]);
                counter_inc(&self.producer_sent_counter, kv![]);
                Ok(())
            }
            Err(e) => {
                counter_inc(&self.producer_requests_counter, kv!["success" => false]);
                self.dead_letter(payload).await?;
                Err(e)
            }
        }
    }
}
