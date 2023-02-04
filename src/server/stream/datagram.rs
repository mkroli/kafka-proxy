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
use crate::cli::UnixDatagramServer;
use crate::server::stream::cleanup::ListenerCleanup;
use crate::server::stream::{BytesStream, MessageStream};
use anyhow::Result;
use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio::net::UnixDatagram;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

macro_rules! datagram_socket_message_stream {
    ($tp:ty, $self:ident => $socket:expr) => {
        #[async_trait]
        impl MessageStream for $tp {
            fn concurrency_limit(&self) -> usize {
                self.concurrency_limit
            }

            async fn stream(&$self, mut shutdown_trigger_receiver: Receiver<()>) -> Result<BytesStream> {
                let socket = $socket;
                let mut buf = [0; 8192];
                let (snd, rcv) = mpsc::channel(1);
                tokio::spawn(async move {
                    loop {
                        let len = tokio::select! {
                            _ = shutdown_trigger_receiver.recv() => break,
                            len = socket.recv(&mut buf) => len,
                        };
                        let msg = match len {
                            Ok(len) => Ok(buf[..len].into()),
                            Err(e) => Err(e.into()),
                        };
                        if let Err(e) = snd.send(msg).await {
                            log::warn!("{}", e);
                        }
                    }
                });
                Ok(Box::new(ReceiverStream::new(rcv)))
            }
        }
    };
}

datagram_socket_message_stream!(UdpSocketServer, self => UdpSocket::bind(self.address).await?);

datagram_socket_message_stream!(UnixDatagramServer, self => ListenerCleanup::<UnixDatagram>::bind(self.path.clone())?);
