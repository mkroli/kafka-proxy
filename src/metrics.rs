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

use anyhow::Result;
use axum::extract::State;
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Server;
use axum::{Router, TypedHeader};
use opentelemetry::metrics::{Counter, Meter, MeterProvider};
use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::{controllers, processors, selectors};
use opentelemetry::sdk::Resource;
use opentelemetry::{Context, KeyValue};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;

pub const COLLECT_PERIOD_MS: u64 = 10000;

pub struct Metrics {
    exporter: PrometheusExporter,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for &Metrics {
    fn into_response(self) -> Response {
        let metric_families = self.exporter.registry().gather();
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

    pub fn new() -> Metrics {
        let controller = controllers::basic(processors::factory(
            selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
            aggregation::cumulative_temporality_selector(),
        ))
        .with_collect_period(Duration::from_millis(COLLECT_PERIOD_MS))
        .with_resource(Resource::new([KeyValue::new(
            "service.name",
            env!("CARGO_PKG_NAME"),
        )]))
        .build();

        let exporter = opentelemetry_prometheus::exporter(controller).init();
        Metrics { exporter }
    }

    pub async fn run(
        self,
        bind_address: SocketAddr,
        mut shutdown_trigger_receiver: Receiver<()>,
    ) -> Result<()> {
        let app = Router::new()
            .route("/metrics", get(Metrics::metrics_handler))
            .with_state(Arc::new(self));
        Server::try_bind(&bind_address)?
            .serve(app.into_make_service())
            .with_graceful_shutdown(async {
                let _ = shutdown_trigger_receiver.recv().await;
            })
            .await?;
        Ok(())
    }

    pub fn meter_provider(&self) -> Result<Meter> {
        Ok(self
            .exporter
            .meter_provider()?
            .meter(env!("CARGO_PKG_NAME")))
    }
}

#[macro_export]
macro_rules! kv {
    ($($key:expr => $value:expr),*) => {
        &[$(KeyValue::new($key, $value)),*]
    };
}

pub fn counter_inc(counter: &Counter<u64>, attributes: &[KeyValue]) {
    counter.add(&Context::current(), 1, attributes)
}
