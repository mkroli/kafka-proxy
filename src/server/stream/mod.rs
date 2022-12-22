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

mod file;
mod socket;

use crate::kafka_producer::KafkaProducer;
use crate::server::Server;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_stream::{Stream, StreamExt};

type StringStreamResult = Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>>;

#[async_trait]
trait StringStream {
    async fn stream(&self) -> StringStreamResult;
}

#[async_trait]
impl<T> Server for T
where
    T: StringStream + Sync,
{
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> Result<()> {
        let mut lines = tokio::select! {
            _ = shutdown_trigger_receiver.recv() => return Ok(()),
            lines = self.stream() => lines?,
        };
        loop {
            tokio::select! {
                _ = shutdown_trigger_receiver.recv() => break,
                line = lines.next() => match line {
                    None => break,
                    Some(Err(e)) => log::error!("{}", e),
                    Some(Ok(line)) => kafka_producer.send(&vec!(), line.as_bytes()).await?,
                },
            }
        }
        Ok(())
    }
}
