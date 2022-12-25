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
use crate::server::stream::{BytesStream, MessageStream};
use anyhow::Result;
use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio::net::UnixDatagram;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

macro_rules! datagram_socket_message_stream {
    ($tp:ty, $self:ident => $socket:expr) => {
        #[async_trait]
        impl MessageStream for $tp {
            async fn stream(&$self) -> Result<BytesStream> {
                let socket = $socket;
                let mut buf = [0; 8192];
                let (snd, rcv) = mpsc::channel(1);
                tokio::spawn(async move {
                    loop {
                        let msg = match socket.recv(&mut buf).await {
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

datagram_socket_message_stream!(UnixDatagramServer, self => UnixDatagram::bind(&self.path)?);