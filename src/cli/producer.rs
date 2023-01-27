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

use std::env;
use std::path::PathBuf;

use anyhow::{Error, Result};
use clap::Args;
use rdkafka::ClientConfig;

#[derive(Debug, Args)]
pub struct Producer {
    #[arg(
        short,
        long,
        env = "KAFKA_PROXY_BOOTSTRAP_SERVER",
        value_name = "ADDRESS_LIST",
        default_value_t = String::from("127.0.0.1:9092"),
    )]
    pub bootstrap_server: String,
    #[arg(short, long, env = "KAFKA_PROXY_TOPIC")]
    pub topic: String,
    #[arg(long, env = "KAFKA_PROXY_SCHEMA_REGISTRY_URL")]
    pub schema_registry_url: Option<String>,
    #[arg(
        long,
        required = false,
        value_name = "KEY=VALUE",
        help = "[env: KAFKA_PROXY_PRODUCER_<KEY>=]",
        value_parser = Producer::parse_tuple,
    )]
    pub producer_config: Vec<(String, String)>,
    #[arg(
        long,
        required = false,
        env = "KAFKA_PROXY_DEAD_LETTERS",
        value_name = "FILENAME"
    )]
    pub dead_letters: Option<PathBuf>,
}

impl Producer {
    fn parse_tuple(s: &str) -> Result<(String, String)> {
        let pivot = s.find('=').ok_or_else(|| Error::msg("Invalid format"))?;
        let key = &s[..pivot];
        let value = &s[pivot + 1..];
        Ok((key.to_string(), value.to_string()))
    }

    pub fn client_config(&self, defaults: Vec<(&str, &str)>) -> ClientConfig {
        let mut cfg = ClientConfig::new();

        // defaults
        for (key, value) in defaults {
            cfg.set(key, value);
        }

        // process environment variables
        for (key, value) in env::vars() {
            if let Some(key) = key.strip_prefix("KAFKA_PROXY_PRODUCER_") {
                let key = key.to_lowercase().replace('_', ".").replace("..", "_");
                cfg.set(key, value);
            }
        }

        // process arguments
        for (key, value) in &self.producer_config {
            cfg.set(key, value);
        }

        cfg
    }
}
