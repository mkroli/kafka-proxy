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

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(name = "delete")]
    Delete(DeleteCommand),
    #[command(name = "send")]
    Send(SendCommand),
}

#[derive(Args, Debug)]
struct DeleteCommand {
    #[arg()]
    name: String,
}

#[derive(Args, Debug)]
struct SendCommand {
    #[arg(short = 'n', long = "num", default_value_t = 1)]
    iterations: i32,
    #[arg()]
    name: String,
    #[arg()]
    message: String,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Delete(delete) => {
            posixmq::remove_queue(&delete.name).unwrap();
        }
        Command::Send(send) => {
            let mq = posixmq::OpenOptions::writeonly()
                .create()
                .open(&send.name)
                .unwrap();
            for _ in 0..send.iterations {
                mq.send(0, send.message.as_bytes()).unwrap();
            }
        }
    }
}
