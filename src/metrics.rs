use anyhow::Result;
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum::Server;
use opentelemetry::metrics::{Meter, MeterProvider};
use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::{controllers, processors, selectors};
use opentelemetry::sdk::Resource;
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

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
            Ok(()) => Ok(([(header::CONTENT_TYPE, "text/plain")], result)),
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
        let controller = controllers::basic(
            processors::factory(
                selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
                aggregation::cumulative_temporality_selector(),
            )
            .with_memory(true),
        )
        .with_resource(Resource::empty())
        .build();

        let exporter = opentelemetry_prometheus::exporter(controller).init();
        Metrics { exporter }
    }

    pub async fn run(
        self,
        bind_address: SocketAddr,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> Result<()> {
        let app = Router::new()
            .route("/metrics", get(Metrics::metrics_handler))
            .with_state(Arc::new(self));
        Server::bind(&bind_address)
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
