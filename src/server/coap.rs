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

use crate::cli::CoapServer;
use crate::kafka::KafkaProducer;
use crate::server::Server;
use async_trait::async_trait;
use coap::request::{CoapRequest, Method, Status};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

#[async_trait]
impl Server for CoapServer {
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> anyhow::Result<()> {
        let kafka_producer = Arc::new(kafka_producer);
        let server = coap::Server::new_udp(self.address)?;
        let run = server.run(move |mut request: Box<CoapRequest<SocketAddr>>| {
            let kafka_producer = kafka_producer.clone();
            async move {
                let response_status = match request.get_method() {
                    &Method::Post => match request.get_path().as_str() {
                        "produce" => match kafka_producer.send(&request.message.payload).await {
                            Ok(()) => Status::Changed,
                            Err(e) => {
                                log::warn!("{e}");
                                Status::InternalServerError
                            }
                        },
                        _ => Status::NotFound,
                    },
                    _ => Status::MethodNotAllowed,
                };

                if let Some(ref mut message) = request.response {
                    message.set_status(response_status);
                    message.message.payload = Vec::new();
                }

                request
            }
        });
        tokio::select! {
            result = run => result?,
            result = shutdown_trigger_receiver.recv() => result?,
        }
        Ok(())
    }
}
