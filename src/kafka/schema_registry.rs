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
use anyhow::{Result, bail};
use apache_avro::Schema;
use schema_registry_converter::async_impl::schema_registry;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use schema_registry_converter::schema_registry_common::{RegisteredSchema, SubjectNameStrategy};

pub struct SchemaRegistry {
    id: u32,
    schema: Schema,
}

fn sr_settings(
    schema_registry: &crate::cli::schema_registry::SchemaRegistry,
) -> Result<SrSettings> {
    match &schema_registry.schema_registry_url {
        None => bail!("No Schema Registry URL configured"),
        Some(url) => Ok(SrSettings::new(url.clone())),
    }
}

fn subject_name_strategy(
    schema_registry: &crate::cli::schema_registry::SchemaRegistry,
    topic: String,
) -> SubjectNameStrategy {
    match schema_registry {
        crate::cli::schema_registry::SchemaRegistry {
            record_name: Some(record_name),
            ..
        } => SubjectNameStrategy::RecordNameStrategy(record_name.clone()),
        crate::cli::schema_registry::SchemaRegistry {
            topic_record_name: Some(record_name),
            ..
        } => SubjectNameStrategy::TopicRecordNameStrategy(topic, record_name.clone()),
        _ => SubjectNameStrategy::TopicNameStrategy(topic, false),
    }
}

async fn registered_schema(
    schema_registry: &crate::cli::schema_registry::SchemaRegistry,
    sr_settings: &SrSettings,
    topic: String,
) -> Result<RegisteredSchema> {
    let schema = match &schema_registry.schema_id {
        Some(id) => schema_registry::get_schema_by_id(*id, sr_settings).await?,
        None => {
            schema_registry::get_schema_by_subject(
                sr_settings,
                &subject_name_strategy(schema_registry, topic),
            )
            .await?
        }
    };
    Ok(schema)
}

async fn schema(
    schema_registry: &crate::cli::schema_registry::SchemaRegistry,
    topic: String,
) -> Result<(u32, Schema)> {
    let sr_settings = sr_settings(schema_registry)?;
    let registered_schema = registered_schema(schema_registry, &sr_settings, topic).await?;
    let schema = Schema::parse_str(&registered_schema.schema)?;
    Ok((registered_schema.id, schema))
}

impl SchemaRegistry {
    pub async fn new(
        topic_name: String,
        schema_registry: &crate::cli::schema_registry::SchemaRegistry,
    ) -> Result<SchemaRegistry> {
        let (id, schema) = schema(schema_registry, topic_name).await?;
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
