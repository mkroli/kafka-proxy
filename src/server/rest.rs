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

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use rdkafka::message::ToBytes;
use tokio::net::TcpListener;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

use crate::Server;
use crate::cli::RestServer;
use crate::kafka::KafkaProducer;

async fn produce_handler(
    State(kafka_producer): State<Arc<KafkaProducer>>,
    bytes: Bytes,
) -> std::result::Result<StatusCode, StatusCode> {
    match kafka_producer.send(bytes.to_bytes()).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            log::warn!("{e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[async_trait]
impl Server for RestServer {
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> Result<()> {
        let app = Router::new()
            .route("/produce", post(produce_handler))
            .with_state(Arc::new(kafka_producer));
        let listener = TcpListener::bind(&self.address).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_trigger_receiver.recv().await;
            })
            .await?;
        Ok(())
    }
}
