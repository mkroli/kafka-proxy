/*
 * Copyright 2023 Michael Krolikowski
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

use std::{
    hash::Hash,
    sync::{
        Arc, RwLock,
        atomic::{AtomicI64, AtomicU64},
    },
};

use anyhow::Result;
use prometheus_client::{
    collector::Collector,
    encoding::{EncodeGaugeValue, EncodeLabelSet, EncodeMetric},
    metrics::{
        family::Family,
        gauge::{Atomic, ConstGauge, Gauge},
    },
};
use rdkafka::{ClientContext, Statistics};

#[derive(Debug, Clone)]
pub struct TelemetryClientContext {
    latest: Arc<RwLock<Statistics>>,
}

impl TelemetryClientContext {
    pub fn new() -> Result<TelemetryClientContext> {
        let latest = Arc::new(RwLock::new(Statistics::default()));
        Ok(TelemetryClientContext { latest })
    }
}

impl ClientContext for TelemetryClientContext {
    fn stats(&self, statistics: Statistics) {
        match self.latest.write() {
            Ok(mut latest) => {
                *latest = statistics;
            }
            Err(e) => log::warn!("Failed to update statistics: {e}"),
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, EncodeLabelSet)]
struct BrokerLabels {
    name: String,
    nodeid: String,
    nodename: String,
    state: String,
}

fn encode_metric<M: EncodeMetric>(
    encoder: &mut prometheus_client::encoding::DescriptorEncoder,
    name: &str,
    help: &str,
    metric: M,
) -> std::result::Result<(), std::fmt::Error> {
    let metric_encoder = encoder.encode_descriptor(name, help, None, metric.metric_type())?;
    metric.encode(metric_encoder)?;
    Ok(())
}

fn encode_gauge<T: EncodeGaugeValue>(
    encoder: &mut prometheus_client::encoding::DescriptorEncoder,
    name: &str,
    help: &str,
    value: T,
) -> std::result::Result<(), std::fmt::Error> {
    encode_metric(encoder, name, help, ConstGauge::new(value))
}

trait GaugeAtomicType<T> {
    type A: Atomic<T> + Default;
}

impl GaugeAtomicType<i64> for i64 {
    type A = AtomicI64;
}

impl GaugeAtomicType<u64> for u64 {
    type A = AtomicU64;
}

fn encode_single_gauge_family<S, V>(
    encoder: &mut prometheus_client::encoding::DescriptorEncoder,
    name: &str,
    help: &str,
    labels: &S,
    value: V,
) -> std::result::Result<(), std::fmt::Error>
where
    S: Clone + Eq + Hash + EncodeLabelSet,
    V: EncodeGaugeValue + GaugeAtomicType<V>,
{
    let family = Family::<S, Gauge<V, V::A>>::default();
    let metric = family.get_or_create_owned(labels);
    metric.set(value);
    encode_metric(encoder, name, help, family)
}

impl Collector for TelemetryClientContext {
    fn encode(
        &self,
        mut encoder: prometheus_client::encoding::DescriptorEncoder,
    ) -> std::result::Result<(), std::fmt::Error> {
        let lock = self.latest.clone();
        let stats = lock.read().map_err(|_| std::fmt::Error)?;

        encode_gauge(
            &mut encoder,
            "replyq",
            "Number of ops (callbacks, events, etc) waiting in queue for application to serve with rd_kafka_poll()",
            stats.replyq,
        )?;
        encode_gauge(
            &mut encoder,
            "msgcnt",
            "Current number of messages in producer queues",
            stats.msg_cnt,
        )?;
        encode_gauge(
            &mut encoder,
            "tx",
            "Total number of requests sent to Kafka brokers",
            stats.tx,
        )?;
        encode_gauge(
            &mut encoder,
            "txbytes",
            "Total number of bytes transmitted to Kafka brokers",
            stats.tx_bytes,
        )?;
        encode_gauge(
            &mut encoder,
            "rx",
            "Total number of responses received from Kafka brokers",
            stats.rx,
        )?;
        encode_gauge(
            &mut encoder,
            "rxbytes",
            "Total number of bytes received from Kafka brokers",
            stats.rx_bytes,
        )?;
        encode_gauge(
            &mut encoder,
            "txmsgs",
            "Total number of messages transmitted (produced) to Kafka brokers",
            stats.txmsgs,
        )?;
        encode_gauge(
            &mut encoder,
            "txmsgsbytes",
            "Total number of message bytes (including framing, such as per-Message framing and MessageSet/batch framing) transmitted to Kafka brokers",
            stats.txmsg_bytes,
        )?;
        encode_gauge(
            &mut encoder,
            "rxmsgs",
            "Total number of messages consumed, not including ignored messages (due to offset, etc), from Kafka brokers",
            stats.rxmsgs,
        )?;
        encode_gauge(
            &mut encoder,
            "rxmsgsbytes",
            "Total number of message bytes (including framing) received from Kafka brokers",
            stats.rxmsg_bytes,
        )?;

        for (name, broker) in &stats.brokers {
            let labels = BrokerLabels {
                name: name.to_string(),
                nodeid: broker.nodeid.to_string(),
                nodename: broker.nodename.to_string(),
                state: broker.state.to_string(),
            };

            let broker_state_family = Family::<BrokerLabels, Gauge>::default();
            for state in [
                "INIT",
                "DOWN",
                "CONNECT",
                "AUTH",
                "APIVERSION_QUERY",
                "AUTH_HANDSHAKE",
                "UP",
                "UPDATE",
            ] {
                let v = (broker.state == state) as i64;
                let mut labels = labels.clone();
                labels.state = state.to_string();
                let metric = broker_state_family.get_or_create(&labels);
                metric.set(v);
            }
            encode_metric(
                &mut encoder,
                "broker_state",
                "Broker state (INIT, DOWN, CONNECT, AUTH, APIVERSION_QUERY, AUTH_HANDSHAKE, UP, UPDATE)",
                broker_state_family,
            )?;

            encode_single_gauge_family(
                &mut encoder,
                "broker_outbufcnt",
                "Number of requests awaiting transmission to broker",
                &labels,
                broker.outbuf_cnt,
            )?;
            encode_single_gauge_family(
                &mut encoder,
                "broker_outbufmsgcnt",
                "Number of messages awaiting transmission to broker",
                &labels,
                broker.outbuf_msg_cnt,
            )?;
            encode_single_gauge_family(
                &mut encoder,
                "broker_waitrespcnt",
                "Number of requests in-flight to broker awaiting response",
                &labels,
                broker.waitresp_cnt,
            )?;
            encode_single_gauge_family(
                &mut encoder,
                "broker_waitrespmsgcnt",
                "Number of messages in-flight to broker awaiting response",
                &labels,
                broker.waitresp_msg_cnt,
            )?;
            encode_single_gauge_family(
                &mut encoder,
                "broker_tx",
                "Total number of requests sent",
                &labels,
                broker.tx,
            )?;
            encode_single_gauge_family(
                &mut encoder,
                "broker_txbytes",
                "Total number of bytes sent",
                &labels,
                broker.txbytes,
            )?;
            encode_single_gauge_family(
                &mut encoder,
                "broker_rx",
                "Total number of responses received",
                &labels,
                broker.rx,
            )?;
            encode_single_gauge_family(
                &mut encoder,
                "broker_rxbytes",
                "Total number of bytes received",
                &labels,
                broker.rxbytes,
            )?;
        }

        Ok(())
    }
}
