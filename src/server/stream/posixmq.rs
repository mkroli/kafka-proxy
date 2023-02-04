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

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use posixmq::PosixMq;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;

use crate::cli::PosixMQServer;
use crate::server::stream::{BytesStream, MessageStream};

async fn mq_loop(
    mq: Arc<PosixMq>,
    snd: Sender<Result<Vec<u8>>>,
    mut shutdown_trigger_receiver: Receiver<()>,
) -> Result<()> {
    loop {
        let mq = mq.clone();
        let recv = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
            let mut buf = [0; 8192];
            let (_, len) = mq.recv(&mut buf)?;
            Ok(buf[..len].into())
        });
        let msg = tokio::select! {
            _ = shutdown_trigger_receiver.recv() => break,
            msg = recv => msg??,
        };
        snd.send(Ok(msg)).await?;
    }
    Ok(())
}

#[async_trait]
impl MessageStream for PosixMQServer {
    fn concurrency_limit(&self) -> usize {
        self.concurrency_limit
    }

    async fn stream(&self, shutdown_trigger_receiver: Receiver<()>) -> Result<BytesStream> {
        let (snd, rcv) = mpsc::channel(1);

        let name = self.name.clone();
        let capacity = self.capacity;
        let mq = tokio::task::spawn_blocking(move || {
            posixmq::OpenOptions::readonly()
                .capacity(capacity)
                .max_msg_len(8192)
                .create()
                .open(&name)
        })
        .await??;
        let mq = Arc::new(mq);
        tokio::spawn(async {
            if let Err(e) = mq_loop(mq, snd, shutdown_trigger_receiver).await {
                log::error!("{}", e);
            }
        });

        Ok(Box::new(ReceiverStream::new(rcv)))
    }
}
