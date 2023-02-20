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

use std::net::IpAddr::V4;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum ServerCommand {
    #[command(name = "stdin", long_about = "Read one message per line from stdin")]
    StdIn(StdInServer),
    #[command(name = "file", long_about = "Read one message per line from a file")]
    File(FileServer),
    #[command(
        name = "unix-dgram",
        long_about = "Receive messages from a Unix Datagram socket"
    )]
    UnixDatagram(UnixDatagramServer),
    #[command(
        name = "unix",
        long_about = "Receive one message per line from a Unix socket"
    )]
    UnixSocket(UnixSocketServer),
    #[command(name = "udp", long_about = "Receive messages from a UDP socket")]
    UdpSocket(UdpSocketServer),
    #[command(
        name = "tcp",
        long_about = "Receive one message per line from a TCP socket"
    )]
    TcpSocket(TcpSocketServer),
    #[cfg(feature = "coap")]
    #[command(name = "coap", long_about = "Receive messages via CoAP")]
    Coap(CoapServer),
    #[command(name = "rest", long_about = "Receive messages via HTTP")]
    Rest(RestServer),
    #[cfg(feature = "posixmq")]
    #[command(name = "posixmq", long_about = "Receive messages via Posix MQ")]
    PosixMQ(PosixMQServer),
}

#[derive(Debug, Args)]
pub struct UnixDatagramServer {
    #[arg(
        long,
        default_value_t = 1024,
        help = "Maximum number of unacknowledged messages"
    )]
    pub concurrency_limit: usize,
    #[arg()]
    pub path: PathBuf,
}

#[derive(Debug, Args)]
pub struct RestServer {
    #[arg(
        short,
        long,
        default_value_t = SocketAddr::new(V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
    )]
    pub address: SocketAddr,
}

#[derive(Debug, Args)]
pub struct CoapServer {
    #[arg(
        short,
        long,
        default_value_t = SocketAddr::new(V4(Ipv4Addr::new(127, 0, 0, 1)), 5683)
    )]
    pub address: SocketAddr,
}

#[derive(Debug, Args)]
pub struct StdInServer {
    #[arg(
        long,
        default_value_t = 1024,
        help = "Maximum number of unacknowledged messages"
    )]
    pub concurrency_limit: usize,
    #[arg(short, long, default_value_t = false, help = "Base64-decode input")]
    pub base64: bool,
}

#[derive(Debug, Args)]
pub struct FileServer {
    #[arg(
        long,
        default_value_t = 1024,
        help = "Maximum number of unacknowledged messages"
    )]
    pub concurrency_limit: usize,
    #[arg(short, long, default_value_t = false, help = "Base64-decode input")]
    pub base64: bool,
    #[arg()]
    pub file: PathBuf,
}

#[derive(Debug, Args)]
pub struct UnixSocketServer {
    #[arg(
        long,
        default_value_t = 1024,
        help = "Maximum number of unacknowledged messages"
    )]
    pub concurrency_limit: usize,
    #[arg(short, long, default_value_t = false, help = "Base64-decode input")]
    pub base64: bool,
    #[arg()]
    pub file: PathBuf,
}

#[derive(Debug, Args)]
pub struct TcpSocketServer {
    #[arg(
        long,
        default_value_t = 1024,
        help = "Maximum number of unacknowledged messages"
    )]
    pub concurrency_limit: usize,
    #[arg(short, long, default_value_t = false, help = "Base64-decode input")]
    pub base64: bool,
    #[arg()]
    pub address: SocketAddr,
}

#[derive(Debug, Args)]
pub struct UdpSocketServer {
    #[arg(
        long,
        default_value_t = 1024,
        help = "Maximum number of unacknowledged messages"
    )]
    pub concurrency_limit: usize,
    #[arg()]
    pub address: SocketAddr,
}

#[derive(Debug, Args)]
pub struct PosixMQServer {
    #[arg(short, long, default_value_t = 10)]
    pub capacity: usize,
    #[arg(
        long,
        default_value_t = 1024,
        help = "Maximum number of unacknowledged messages"
    )]
    pub concurrency_limit: usize,
    #[arg()]
    pub name: String,
}
