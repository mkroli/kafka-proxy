use std::time::Duration;

use anyhow::{bail, Result};
use apache_avro::types::Value;
use opentelemetry::metrics::{Counter, Meter};
use opentelemetry::Context;
use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;

use crate::cli::Producer;

const TIMEOUT: Timeout = Timeout::After(Duration::from_millis(3000));

pub struct KafkaProducer {
    topic: String,
    producer: FutureProducer,
    schema_registry: Option<SchemaRegistry>,
    producer_sent_counter: Counter<u64>,
}

struct SchemaRegistry {
    subject_name_strategy: SubjectNameStrategy,
    encoder: AvroEncoder<'static>,
}

impl KafkaProducer {
    pub async fn new(cfg: Producer, meter: Meter) -> Result<KafkaProducer> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", cfg.bootstrap_server)
            .create()?;

        let schema_registry = match cfg.schema_registry_url {
            None => None,
            Some(ref url) => {
                let subject_name_strategy =
                    SubjectNameStrategy::TopicNameStrategy(cfg.topic.clone(), false);
                let sr_settings = SrSettings::new(String::from(url));
                let encoder = AvroEncoder::new(sr_settings);

                Some(SchemaRegistry {
                    subject_name_strategy,
                    encoder,
                })
            }
        };

        let producer_sent_counter = meter
            .u64_counter("kafka.produced")
            .with_description("Number of produced Kafka Records")
            .init();
        producer_sent_counter.add(&Context::current(), 0, &[]);

        Ok(KafkaProducer {
            topic: cfg.topic,
            producer,
            schema_registry,
            producer_sent_counter,
        })
    }

    async fn encode(&self, payload: &[u8]) -> Result<Vec<u8>> {
        match &self.schema_registry {
            None => Ok(Vec::from(payload)),
            Some(schema_registry) => {
                let json: serde_json::Value = serde_json::from_slice(payload)?;
                let value: Value = json.into();
                let fields = match value {
                    Value::Map(ref m) => {
                        let mut fields: Vec<(&str, Value)> = Vec::with_capacity(m.len());
                        for (k, v) in m {
                            fields.push((k, v.clone()));
                        }
                        fields
                    }
                    _ => bail!("Only objects are supported"),
                };
                let encoded = schema_registry
                    .encoder
                    .encode(fields, schema_registry.subject_name_strategy.clone())
                    .await?;
                Ok(encoded)
            }
        }
    }

    pub async fn send<K>(&self, key: &K, payload: &[u8]) -> Result<()>
    where
        K: ToBytes + ?Sized,
    {
        let payload = self.encode(payload).await?;
        let record = FutureRecord::to(&self.topic).key(key).payload(&payload);
        self.producer
            .send(record, TIMEOUT)
            .await
            .map_err(|(e, _)| e)?;
        self.producer_sent_counter.add(&Context::current(), 1, &[]);
        Ok(())
    }
}
