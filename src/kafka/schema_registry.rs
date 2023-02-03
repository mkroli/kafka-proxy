/*
 * Copyright 2023 Michael Krolikowski
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

use crate::kafka::serde::Deserializer;
use anyhow::{bail, Result};
use apache_avro::types::Value;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;

pub struct SchemaRegistry {
    subject_name_strategy: SubjectNameStrategy,
    deserializer: Deserializer,
    encoder: AvroEncoder<'static>,
}

impl SchemaRegistry {
    pub async fn new(
        topic_name: String,
        schema_registry: &crate::cli::schema_registry::SchemaRegistry,
    ) -> Result<SchemaRegistry> {
        let sr_settings = schema_registry.sr_settings()?;

        let schema = schema_registry
            .schema(&sr_settings, topic_name.clone())
            .await?;
        let deserializer = Deserializer::new(schema);

        let subject_name_strategy = schema_registry.subject_name_strategy(topic_name);

        let encoder = AvroEncoder::new(sr_settings);

        Ok(SchemaRegistry {
            subject_name_strategy,
            deserializer,
            encoder,
        })
    }

    pub async fn encode(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let json = serde_json::from_slice(payload)?;
        let value = self.deserializer.deserialize_json(json)?;

        let fields = match value {
            Value::Record(ref m) => {
                let mut fields: Vec<(&str, Value)> = Vec::with_capacity(m.len());
                for (k, v) in m {
                    fields.push((k, v.clone()));
                }
                fields
            }
            _ => bail!("Only records are supported"),
        };

        let encoded = self
            .encoder
            .encode(fields, self.subject_name_strategy.clone())
            .await?;
        Ok(encoded)
    }
}
