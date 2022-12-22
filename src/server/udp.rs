use crate::cli::UdpSocketServer;
use crate::kafka_producer::KafkaProducer;
use crate::server::Server;
use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

#[async_trait]
impl Server for UdpSocketServer {
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> anyhow::Result<()> {
        let socket = UdpSocket::bind(self.address).await?;
        let mut buf = [0; 8192];
        loop {
            let len = tokio::select! {
                _ = shutdown_trigger_receiver.recv() => return Ok(()),
                len = socket.recv(&mut buf) => len?,
            };
            kafka_producer.send(&vec![], &buf[..len]).await?;
        }
    }
}
