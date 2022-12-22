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
