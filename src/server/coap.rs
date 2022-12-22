use crate::cli::CoapServer;
use crate::kafka_producer::KafkaProducer;
use crate::server::Server;
use async_trait::async_trait;
use coap_lite::{RequestType, ResponseType};
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

#[async_trait]
impl Server for CoapServer {
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> anyhow::Result<()> {
        let kafka_producer = Arc::new(kafka_producer);
        let mut server = coap::Server::new(self.address)?;
        let run = server.run(|request| async {
            let response_status = match request.get_method() {
                &RequestType::Post => match request.get_path().as_str() {
                    "produce" => match kafka_producer.send(&[], &request.message.payload).await {
                        Ok(()) => ResponseType::Changed,
                        Err(e) => {
                            log::error!("{}", e);
                            ResponseType::InternalServerError
                        }
                    },
                    _ => ResponseType::NotFound,
                },
                _ => ResponseType::MethodNotAllowed,
            };

            match request.response {
                Some(mut message) => {
                    message.set_status(response_status);
                    message.message.payload = Vec::new();
                    Some(message)
                }
                _ => None,
            }
        });
        tokio::select! {
            result = run => result?,
            result = shutdown_trigger_receiver.recv() => result?,
        }
        Ok(())
    }
}
