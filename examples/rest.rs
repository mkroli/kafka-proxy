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
use futures::StreamExt;
use futures::stream::iter;
use reqwest::StatusCode;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short = 'n', long = "num", default_value_t = 1)]
    iterations: i32,
    #[arg(short, long, default_value_t = String::from("http://127.0.0.1:8080/produce"))]
    url: String,
    #[arg()]
    message: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = reqwest::Client::new();
    let body = cli.message;
    iter(0..cli.iterations)
        .for_each_concurrent(8192, |_| async {
            let response = client.post(&cli.url).body(body.clone()).send().await;
            match response {
                Ok(r) if r.status() == StatusCode::NO_CONTENT => (),
                Ok(r) => log::error!("response was {:?}", r),
                Err(e) => log::error!("{}", e),
            }
        })
        .await;
}
