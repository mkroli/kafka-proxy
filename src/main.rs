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

#![forbid(unsafe_code)]

use std::process::exit;

use anyhow::{Error, Result};
use base64::alphabet;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use clap::Parser;
use log::SetLoggerError;

use crate::cli::{Cli, ServerCommand};
use crate::kafka::KafkaProducer;
use crate::metrics::Metrics;
use crate::server::Server;

mod cli;
mod kafka;
mod metrics;
mod server;

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

const ENGINE: GeneralPurpose =
    GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::new());

async fn shutdown_signal() -> Result<()> {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?
            .recv()
            .await
            .ok_or(Error::msg("failed to install signal handler"))
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<Result<()>>();

    tokio::select! {
        result = ctrl_c => Ok(result?),
        result = terminate => Ok(result?),
    }
}

fn server(server: ServerCommand) -> Box<dyn Server + Send> {
    match server {
        ServerCommand::Rest(server) => Box::new(server),
        #[cfg(feature = "coap")]
        ServerCommand::Coap(server) => Box::new(server),
        ServerCommand::UnixDatagram(server) => Box::new(server),
        ServerCommand::StdIn(server) => Box::new(server),
        ServerCommand::File(server) => Box::new(server),
        ServerCommand::UnixSocket(server) => Box::new(server),
        ServerCommand::TcpSocket(server) => Box::new(server),
        ServerCommand::UdpSocket(server) => Box::new(server),
        #[cfg(feature = "posixmq")]
        ServerCommand::PosixMQ(server) => Box::new(server),
        #[cfg(feature = "nng")]
        ServerCommand::Nng(server) => Box::new(server),
    }
}

async fn run() -> Result<()> {
    configure_logging()?;

    let (shutdown_trigger_send, shutdown_trigger_recv) = tokio::sync::broadcast::channel(2);
    let (shutdown_send, mut shutdown_recv) = tokio::sync::mpsc::channel(1);

    let cli = Cli::parse();

    let metrics = Metrics::new()?;
    let meter = metrics.meter_provider();
    let prometheus = if let Some(addr) = cli.prometheus_address {
        let prometheus = metrics.run(addr, shutdown_trigger_send.subscribe());
        tokio::spawn(prometheus)
    } else {
        let mut r = shutdown_trigger_send.subscribe();
        tokio::spawn(async move {
            r.recv().await?;
            Ok(())
        })
    };

    let producer = KafkaProducer::new(cli.producer, &meter).await?;

    let server = tokio::spawn(async move {
        let server = server(cli.server);
        let result = server.run(producer, shutdown_trigger_recv, shutdown_send);
        match result.await {
            Ok(()) => (),
            Err(e) => {
                log::error!("{}", e)
            }
        }
    });

    tokio::select! {
        _ = shutdown_recv.recv() => (),
        Ok(Err(e)) = prometheus => {
            log::error!("{}", e);
        },
        result = shutdown_signal() => result?,
    }
    shutdown_trigger_send.send(())?;
    server.await?;
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
