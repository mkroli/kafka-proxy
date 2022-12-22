mod cli;
mod kafka_producer;
mod metrics;
mod server;

use crate::cli::{Cli, ServerCommand};
use crate::kafka_producer::KafkaProducer;
use crate::metrics::Metrics;
use crate::server::Server;
use anyhow::Result;
use clap::Parser;
use log::SetLoggerError;
use std::process::exit;

fn configure_logging() -> std::result::Result<(), SetLoggerError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S.%3f]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(std::io::stderr())
        .level(log::LevelFilter::Info)
        .apply()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
async fn run() -> Result<()> {
    configure_logging()?;

    let (shutdown_trigger_send, shutdown_trigger_recv) = tokio::sync::broadcast::channel(2);
    let (shutdown_send, mut shutdown_recv) = tokio::sync::mpsc::channel(1);

    let cli = Cli::parse();

    let metrics = Metrics::new();
    let meter = metrics.meter_provider()?;
    let prometheus = if let Some(addr) = cli.prometheus_address {
        let prometheus = metrics.run(
            addr,
            shutdown_trigger_send.subscribe(),
            shutdown_send.clone(),
        );
        tokio::spawn(prometheus)
    } else {
        let s = shutdown_send.clone();
        let mut r = shutdown_trigger_send.subscribe();
        tokio::spawn(async move {
            let _ = s;
            r.recv().await?;
            Ok(())
        })
    };

    let producer = KafkaProducer::new(cli.producer, meter.clone()).await?;

    let server = tokio::spawn(async move {
        let server: Box<dyn Server + Send> = match cli.server {
            ServerCommand::Rest(server) => Box::new(server),
            ServerCommand::Coap(server) => Box::new(server),
            ServerCommand::UnixDatagram(server) => Box::new(server),
            ServerCommand::StdIn(server) => Box::new(server),
            ServerCommand::File(server) => Box::new(server),
            ServerCommand::UnixSocket(server) => Box::new(server),
            ServerCommand::TcpSocket(server) => Box::new(server),
            ServerCommand::UdpSocket(server) => Box::new(server),
        };
        let result = server.run(producer, shutdown_trigger_recv, shutdown_send);
        match result.await {
            Ok(()) => (),
            Err(e) => {
                log::error!("{}", e)
            }
        }
    });

    tokio::select! {
        _ = server => (),
        _ = prometheus => (),
        _ = shutdown_signal() => (),
    }
    shutdown_trigger_send.send(())?;
    shutdown_recv.recv().await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    match run().await {
        Ok(_) => exit(0),
        Err(e) => {
            log::error!("{}", e);
            exit(1)
        }
    }
}
