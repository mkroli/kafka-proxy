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

use crate::cli::{FileServer, StdInServer};
use crate::server::decoder::decode_line;
use crate::server::stream::{BytesStream, MessageStream};
use anyhow::Result;
use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

macro_rules! buf_reader_message_stream {
    ($tp:ty, $self:ident => $reader:expr) => {
        #[async_trait]
        impl MessageStream for $tp {
            fn concurrency_limit(&self) -> usize {
               self.concurrency_limit
            }

            async fn stream(&$self, mut shutdown_trigger_receiver: Receiver<()>) -> Result<BytesStream> {
                let reader = BufReader::new($reader);
                let mut lines = reader.lines();
                let base64 = $self.base64;
                let (snd, rcv) = mpsc::channel(1);
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = shutdown_trigger_receiver.recv() => break,
                            line = lines.next_line() => match line {
                                Ok(Some(l)) => {
                                    let b = match decode_line(l, base64) {
                                        Ok(l) => l,
                                        Err(e) => {
                                            log::warn!("Failed to decode: {e}");
                                            continue
                                        }
                                    };
                                    match snd.send(Ok(b)).await {
                                        Ok(()) => (),
                                        Err(e) => {
                                            log::warn!("Failed to send: {e}");
                                            break
                                        }
                                    }
                                },
                                _ => break,
                            }
                        };
                    }
                });
                Ok(Box::new(ReceiverStream::new(rcv)))
            }
        }
    };
}

buf_reader_message_stream!(StdInServer, self => tokio::io::stdin());

buf_reader_message_stream!(FileServer, self => File::open(&self.file).await?);
