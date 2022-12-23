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

use clap::{Args, Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(flatten, next_help_heading = "Kafka Options")]
    pub producer: Producer,
    #[arg(long = "prometheus", value_name = "ADDRESS")]
    pub prometheus_address: Option<SocketAddr>,
    #[command(subcommand)]
    pub server: ServerCommand,
}

#[derive(Args)]
pub struct Producer {
    #[arg(short, long, value_name = "ADDRESS_LIST", default_value_t = String::from("127.0.0.1:9092"))]
    pub bootstrap_server: String,
    #[arg(short, long)]
    pub topic: String,
    #[arg(long)]
    pub schema_registry_url: Option<String>,
}

#[derive(Subcommand)]
pub enum ServerCommand {
    #[command(name = "stdin")]
    StdIn(StdInServer),
    #[command(name = "file")]
    File(FileServer),
    #[command(name = "unix-dgram")]
    UnixDatagram(UnixDatagramServer),
    #[command(name = "unix")]
    UnixSocket(UnixSocketServer),
    #[command(name = "udp")]
    UdpSocket(UdpSocketServer),
    #[command(name = "tcp")]
    TcpSocket(TcpSocketServer),
    #[command(name = "coap")]
    Coap(CoapServer),
    #[command(name = "rest")]
    Rest(RestServer),
}

#[derive(Args)]
pub struct UnixDatagramServer {
    pub path: PathBuf,
}

#[derive(Args)]
pub struct RestServer {
    #[arg(short, long, default_value_t = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 8080))]
    pub address: SocketAddr,
}

#[derive(Args)]
pub struct CoapServer {
    #[arg(short, long, default_value_t = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 5683))]
    pub address: SocketAddr,
}

#[derive(Args)]
pub struct StdInServer {}

#[derive(Args)]
pub struct FileServer {
    #[arg()]
    pub file: PathBuf,
}

#[derive(Args)]
pub struct UnixSocketServer {
    #[arg()]
    pub file: PathBuf,
}

#[derive(Args)]
pub struct TcpSocketServer {
    #[arg()]
    pub address: SocketAddr,
}

#[derive(Args)]
pub struct UdpSocketServer {
    #[arg()]
    pub address: SocketAddr,
}
