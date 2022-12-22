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

use crate::cli::UdpSocketServer;
use crate::kafka_producer::KafkaProducer;
use crate::server::Server;
use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

#[async_trait]
impl Server for UdpSocketServer {
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> anyhow::Result<()> {
        let socket = UdpSocket::bind(self.address).await?;
        let mut buf = [0; 8192];
        loop {
            let len = tokio::select! {
                _ = shutdown_trigger_receiver.recv() => return Ok(()),
                len = socket.recv(&mut buf) => len?,
            };
            if let Err(e) = kafka_producer.send(&vec![], &buf[..len]).await {
                log::warn!("{}", e);
            }
        }
    }
}
