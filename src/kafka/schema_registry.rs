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

use anyhow::{bail, Result};
use apache_avro::types::Value;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;

pub struct SchemaRegistry {
    subject_name_strategy: SubjectNameStrategy,
    encoder: AvroEncoder<'static>,
}

impl SchemaRegistry {
    pub fn new(
        subject_name_strategy: SubjectNameStrategy,
        encoder: AvroEncoder<'static>,
    ) -> SchemaRegistry {
        SchemaRegistry {
            subject_name_strategy,
            encoder,
        }
    }

    pub async fn encode(&self, payload: &[u8]) -> Result<Vec<u8>> {
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
        let encoded = self
            .encoder
            .encode(fields, self.subject_name_strategy.clone())
            .await?;
        Ok(encoded)
    }
}
