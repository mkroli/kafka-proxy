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

use crate::kafka::serde::deserialize_json;
use anyhow::Result;
use apache_avro::Schema;

pub struct SchemaRegistry {
    id: u32,
    schema: Schema,
}

impl SchemaRegistry {
    pub async fn new(
        topic_name: String,
        schema_registry: &crate::cli::schema_registry::SchemaRegistry,
    ) -> Result<SchemaRegistry> {
        let (id, schema) = schema_registry.schema(topic_name).await?;
        Ok(SchemaRegistry { id, schema })
    }

    pub async fn encode(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let json = serde_json::from_slice(payload)?;
        let value = deserialize_json(&self.schema, json)?;
        let serialized = apache_avro::to_avro_datum(&self.schema, value)?;

        let mut bytes = vec![0u8];
        bytes.extend_from_slice(&self.id.to_be_bytes());
        bytes.extend_from_slice(&serialized);
        Ok(bytes)
    }
}
