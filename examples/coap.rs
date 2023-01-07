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

use clap::Parser;
use coap::CoAPClient;
use coap_lite::ResponseType;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short = 'n', long = "num", default_value_t = 1)]
    iterations: i32,
    #[arg(short, long, default_value_t = String::from("coap://127.0.0.1:5683/produce"))]
    url: String,
    #[arg()]
    message: String,
}

fn main() {
    let cli = Cli::parse();
    let data = Vec::from(cli.message);
    for _ in 0..cli.iterations {
        let response = CoAPClient::post(&cli.url, data.clone()).unwrap();
        if response.get_status() != &ResponseType::Changed {
            log::error!("response was {:?}", &response)
        }
    }
}
