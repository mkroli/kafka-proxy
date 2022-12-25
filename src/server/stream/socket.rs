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

use crate::cli::{TcpSocketServer, UnixSocketServer};
use crate::server::stream::{BytesStream, MessageStream};
use anyhow::Result;
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufStream};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

macro_rules! socket_string_stream {
    ($tp:ty, $self:ident => $listener:expr) => {
        #[async_trait]
        impl MessageStream for $tp {
            async fn stream(&$self) -> Result<BytesStream> {
                let listener = $listener;
                let (snd, rcv) = mpsc::channel(1);
                tokio::spawn(async move {
                    while let Ok((stream, _)) = listener.accept().await {
                        let snd = snd.clone();
                        tokio::spawn(async move {
                            let mut lines = BufStream::new(stream).lines();
                            while let Ok(Some(l)) = lines.next_line().await {
                                match snd.send(Ok(l.into())).await {
                                    Ok(()) => (),
                                    Err(_) => break,
                                }
                            }
                        });
                    }
                });
                Ok(Box::new(ReceiverStream::new(rcv)))
            }
        }
    };
}

socket_string_stream!(UnixSocketServer, self => UnixListener::bind(&self.file)?);

socket_string_stream!(TcpSocketServer, self => TcpListener::bind(self.address).await?);
