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

use anyhow::bail;
use clap::Parser;
use nng::{Message, Protocol, Socket};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short = 'n', long = "num", default_value_t = 1)]
    iterations: i32,
    #[arg(short, long, default_value_t = String::from("ipc:///tmp/kafka-proxy"))]
    address: String,
    #[arg(short, long, value_parser = parse_protocol)]
    protocol: Protocol,
    #[arg()]
    message: String,
}

fn parse_protocol(s: &str) -> anyhow::Result<Protocol> {
    match s {
        "pair0" => Ok(Protocol::Pair0),
        "pair1" => Ok(Protocol::Pair1),
        "pub0" => Ok(Protocol::Pub0),
        "push0" => Ok(Protocol::Push0),
        "req0" => Ok(Protocol::Req0),
        "surveyor0" => Ok(Protocol::Surveyor0),
        _ => bail!("Unsupported protocol"),
    }
}

fn main() {
    let cli = Cli::parse();
    let socket = Socket::new(cli.protocol).unwrap();
    socket.dial(&cli.address).unwrap();
    for _ in 0..cli.iterations {
        let msg: Message = cli.message.as_bytes().into();
        socket.send(msg).unwrap()
    }
}
