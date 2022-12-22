mod file;
mod socket;

use crate::kafka_producer::KafkaProducer;
use crate::server::Server;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_stream::{Stream, StreamExt};

type StringStreamResult = Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>>;

#[async_trait]
trait StringStream {
    async fn stream(&self) -> StringStreamResult;
}

#[async_trait]
impl<T> Server for T
where
    T: StringStream + Sync,
{
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> Result<()> {
        let mut lines = tokio::select! {
            _ = shutdown_trigger_receiver.recv() => return Ok(()),
            lines = self.stream() => lines?,
        };
        loop {
            tokio::select! {
                _ = shutdown_trigger_receiver.recv() => break,
                line = lines.next() => match line {
                    None => break,
                    Some(Err(e)) => log::error!("{}", e),
                    Some(Ok(line)) => kafka_producer.send(&vec!(), line.as_bytes()).await?,
                },
            }
        }
        Ok(())
    }
}
