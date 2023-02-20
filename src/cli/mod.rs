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

use std::net::SocketAddr;

use clap::{ColorChoice, Parser};

pub use producer::Producer;
pub use server::*;

pub mod producer;
pub mod schema_registry;
pub mod server;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    propagate_version = true,
    color = ColorChoice::Auto
)]
pub struct Cli {
    #[arg(
        long = "prometheus",
        env = "KAFKA_PROXY_PROMETHEUS_ADDRESS",
        value_name = "ADDRESS"
    )]
    pub prometheus_address: Option<SocketAddr>,
    #[command(subcommand)]
    pub server: ServerCommand,
    #[command(flatten, next_help_heading = "Kafka Options")]
    pub producer: Producer,
}

#[cfg(test)]
mod tests {
    use crate::Cli;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
