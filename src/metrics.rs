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

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::headers::ContentType;
use axum_extra::TypedHeader;
use opentelemetry::metrics::{Counter, Meter};
use opentelemetry::{metrics::MeterProvider as _, KeyValue};
use opentelemetry_sdk::metrics::MeterProvider;
use opentelemetry_sdk::Resource;
use prometheus::{Encoder, Registry, TextEncoder};
use tokio::net::TcpListener;
use tokio::sync::broadcast::Receiver;

pub const COLLECT_PERIOD_MS: u64 = 10000;

pub struct Metrics {
    registry: Registry,
    provider: MeterProvider,
}

impl IntoResponse for &Metrics {
    fn into_response(self) -> Response {
        let metric_families = self.registry.gather();
        let encoder = TextEncoder::new();
        let mut result = Vec::new();
        let result = match encoder.encode(&metric_families, &mut result) {
            Ok(()) => Ok((TypedHeader(ContentType::text_utf8()), result)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
        result.into_response()
    }
}

impl Metrics {
    async fn metrics_handler(State(metrics): State<Arc<Metrics>>) -> Response {
        metrics.into_response()
    }

    pub fn new() -> Result<Metrics> {
        let registry = Registry::new();
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()?;
        let provider = MeterProvider::builder()
            .with_reader(exporter)
            .with_resource(Resource::new([KeyValue::new(
                "service.name",
                env!("CARGO_PKG_NAME"),
            )]))
            .build();
        Ok(Metrics { registry, provider })
    }

    pub async fn run(
        self,
        bind_address: SocketAddr,
        mut shutdown_trigger_receiver: Receiver<()>,
    ) -> Result<()> {
        let app = Router::new()
            .route("/metrics", get(Metrics::metrics_handler))
            .with_state(Arc::new(self));
        let listener = TcpListener::bind(bind_address).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_trigger_receiver.recv().await;
            })
            .await?;
        Ok(())
    }

    pub fn meter_provider(&self) -> Meter {
        self.provider.meter(env!("CARGO_PKG_NAME"))
    }
}

#[macro_export]
macro_rules! kv {
    ($($key:expr => $value:expr),*) => {
        &[$(KeyValue::new($key, $value)),*]
    };
}

pub fn counter_inc(counter: &Counter<u64>, attributes: &[KeyValue]) {
    counter.add(1, attributes)
}
