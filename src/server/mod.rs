use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

use crate::kafka_producer::KafkaProducer;

pub mod coap;
pub mod file;
pub mod rest;
pub mod udp;
pub mod unix_datagram;

#[async_trait]
pub trait Server {
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> Result<()>;
}
