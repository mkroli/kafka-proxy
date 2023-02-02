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
use axum::extract::{RawBody, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use hyper::Body;
use rdkafka::message::ToBytes;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

use crate::cli::RestServer;
use crate::kafka::KafkaProducer;
use crate::Server;

async fn produce(kafka_producer: Arc<KafkaProducer>, payload: Body) -> Result<()> {
    let payload = hyper::body::to_bytes(payload).await?;
    kafka_producer.send(payload.to_bytes()).await?;
    Ok(())
}

async fn produce_handler(
    State(kafka_producer): State<Arc<KafkaProducer>>,
    RawBody(payload): RawBody,
) -> std::result::Result<StatusCode, StatusCode> {
    match produce(kafka_producer, payload).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            log::warn!("{}", e);
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
        axum::Server::bind(&self.address)
            .serve(app.into_make_service())
            .with_graceful_shutdown(async {
                let _ = shutdown_trigger_receiver.recv().await;
            })
            .await?;
        Ok(())
    }
}
