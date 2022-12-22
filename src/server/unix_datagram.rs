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

use std::net::Shutdown;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use crate::cli::UnixDatagramServer;
use crate::kafka_producer::KafkaProducer;
use anyhow::Result;
use async_trait::async_trait;
use tokio::io;
use tokio::net::UnixDatagram;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

use crate::server::Server;

struct UnixDatagramSocket {
    path: PathBuf,
    socket: UnixDatagram,
}

impl UnixDatagramSocket {
    fn bind(path: &Path) -> Result<UnixDatagramSocket> {
        let socket = UnixDatagram::bind(path)?;
        Ok(UnixDatagramSocket {
            path: path.to_path_buf(),
            socket,
        })
    }
}

impl Drop for UnixDatagramSocket {
    fn drop(&mut self) {
        let _ = self.socket.shutdown(Shutdown::Both);
        let _ = std::fs::remove_file(&self.path);
    }
}

impl Deref for UnixDatagramSocket {
    type Target = UnixDatagram;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}

#[async_trait]
impl Server for UnixDatagramServer {
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> Result<()> {
        let socket = UnixDatagramSocket::bind(&self.path)?;
        loop {
            tokio::select! {
                _ = socket.readable() => {
                    let mut buf = [0; 8192];
                    match socket.try_recv(&mut buf) {
                        Ok(n) => {
                            kafka_producer.send(&vec!(), &buf[..n]).await?;
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
                        Err(e) => {
                            return Err(anyhow::Error::from(e));
                        }
                    }
                },
                _ = shutdown_trigger_receiver.recv() => {
                    return Ok(());
                },
            }
        }
    }
}
