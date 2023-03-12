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

use crate::cli::NngServer;
use crate::server::stream::{BytesStream, MessageStream};
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use nng::options::protocol::pubsub::Subscribe;
use nng::options::Options;
use nng::{Message, Protocol, Socket};
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;

async fn nng_loop(
    ack: bool,
    socket: Socket,
    snd: Sender<Result<Vec<u8>>>,
    mut shutdown_trigger_receiver: Receiver<()>,
) -> Result<()> {
    let socket = Arc::new(socket);
    loop {
        let sock = socket.clone();
        let recv = tokio::task::spawn_blocking(move || sock.recv());
        let msg = tokio::select! {
            _ = shutdown_trigger_receiver.recv() => break,
            msg = recv => msg??,
        };
        snd.send(Ok(msg.to_vec())).await?;

        if ack {
            let sock = socket.clone();
            let reply = tokio::task::spawn_blocking(move || sock.send(Message::new()));
            tokio::select! {
                _ = shutdown_trigger_receiver.recv() => break,
                reply = reply => reply?.map_err(|(_, e)| e)?,
            }
        }
    }
    Ok(())
}

#[async_trait]
impl MessageStream for NngServer {
    fn concurrency_limit(&self) -> usize {
        self.concurrency_limit
    }

    async fn stream(&self, shutdown_trigger_receiver: Receiver<()>) -> Result<BytesStream> {
        let (snd, rcv) = mpsc::channel(1);
        let acknowledge =
            self.acknowledge && self.protocol != Protocol::Pull0 && self.protocol != Protocol::Sub0;
        let socket = Socket::new(self.protocol)?;
        if self.protocol == Protocol::Sub0 {
            socket.set_opt::<Subscribe>(vec![])?;
        }
        socket.listen(&self.address).context("Invalid address")?;
        tokio::spawn(async move {
            if let Err(e) = nng_loop(acknowledge, socket, snd, shutdown_trigger_receiver).await {
                log::error!("{e}");
            }
        });
        Ok(Box::new(ReceiverStream::new(rcv)))
    }
}
