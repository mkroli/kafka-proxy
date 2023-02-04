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

mod cleanup;
mod datagram;
mod file;
#[cfg(feature = "posixmq")]
mod posixmq;
mod socket;

use crate::kafka::KafkaProducer;
use crate::server::Server;
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::StreamExt;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_stream::Stream;

type BytesStream = Box<dyn Stream<Item = Result<Vec<u8>>> + Send + Unpin>;

#[async_trait]
trait MessageStream {
    fn concurrency_limit(&self) -> usize;

    async fn stream(&self, mut shutdown_trigger_receiver: Receiver<()>) -> Result<BytesStream>;
}

#[async_trait]
impl<T> Server for T
where
    T: MessageStream + Sync,
{
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> Result<()> {
        let stream = self.stream(shutdown_trigger_receiver.resubscribe());
        let messages = tokio::select! {
            _ = shutdown_trigger_receiver.recv() => return Ok(()),
            messages = stream => messages?,
        };
        messages
            .for_each_concurrent(self.concurrency_limit(), |msg| async {
                match msg {
                    Err(e) => log::error!("{}", e),
                    Ok(msg) => match kafka_producer.send(&msg).await {
                        Ok(()) => (),
                        Err(e) => log::warn!("{}", e),
                    },
                };
            })
            .await;
        Ok(())
    }
}
