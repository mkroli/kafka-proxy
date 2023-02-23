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

use clap::Args;

#[derive(Debug, Args)]
pub struct SchemaRegistry {
    #[arg(long, env = "KAFKA_PROXY_SCHEMA_REGISTRY_URL")]
    pub schema_registry_url: Option<String>,
    #[arg(
        long,
        requires = "schema_registry_url",
        env = "KAFKA_PROXY_SCHEMA_ID",
        help = "Use a specific schema id rather than the latest version"
    )]
    pub schema_id: Option<u32>,
    #[arg(
        long,
        group = "strategy",
        requires = "schema_registry_url",
        help = "Use TopicNameStrategy to derive the subject name (default)"
    )]
    pub topic_name: bool,
    #[arg(
        long,
        group = "strategy",
        requires = "schema_registry_url",
        value_name = "RECORD_NAME",
        help = "Use RecordNameStrategy to derive the subject name",
        env = "KAFKA_PROXY_SCHEMA_REGISTRY_RECORD_NAME"
    )]
    pub record_name: Option<String>,
    #[arg(
        long,
        group = "strategy",
        requires = "schema_registry_url",
        value_name = "RECORD_NAME",
        help = "Use TopicRecordNameStrategy to derive the subject name",
        env = "KAFKA_PROXY_SCHEMA_REGISTRY_TOPIC_RECORD_NAME"
    )]
    pub topic_record_name: Option<String>,
}
