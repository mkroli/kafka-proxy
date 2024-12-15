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

use std::sync::{Arc, RwLock};

use anyhow::Result;
use opentelemetry::metrics::{AsyncInstrument, Meter};
use opentelemetry::KeyValue;
use rdkafka::statistics::Broker;
use rdkafka::{ClientContext, Statistics};

use crate::kv;

pub struct TelemetryClientContext {
    latest: Arc<RwLock<Statistics>>,
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

fn from_stats<T, C: Fn(&Statistics) -> T>(
    latest: &Arc<RwLock<Statistics>>,
    value_from_stats: C,
) -> impl Fn(&dyn AsyncInstrument<T>) {
    let l = latest.clone();
    move |observer| match l.read() {
        Ok(stats) => observer.observe(value_from_stats(&stats), &[]),
        Err(e) => log::warn!("Failed to retrieve statistics: {e}"),
    }
}

fn broker_callback<T, C: Fn(&dyn AsyncInstrument<T>, &Broker, &[KeyValue])>(
    latest: &Arc<RwLock<Statistics>>,
    callback: C,
) -> impl Fn(&dyn AsyncInstrument<T>) {
    let l = latest.clone();
    move |observer| match l.read() {
        Ok(stats) => {
            for (name, broker) in &stats.brokers {
                let attrs = kv![
                    "name" => name.clone(),
                    "nodeid" => broker.nodeid.to_string(),
                    "nodename" => broker.nodename.clone()
                ];
                callback(observer, broker, attrs)
            }
        }
        Err(e) => log::warn!("Failed to retrieve statistics: {e}"),
    }
}

fn from_broker<T, C: Fn(&Broker) -> T>(
    latest: &Arc<RwLock<Statistics>>,
    value_from_broker: C,
) -> impl Fn(&dyn AsyncInstrument<T>) {
    broker_callback(latest, move |observer, broker, attrs| {
        observer.observe(value_from_broker(broker), attrs);
    })
}

impl TelemetryClientContext {
    pub fn new(meter: &Meter) -> Result<TelemetryClientContext> {
        let latest = Arc::new(RwLock::new(Statistics::default()));

        meter
            .i64_observable_gauge("kafka.producer.replyq")
            .with_description("Number of ops (callbacks, events, etc) waiting in queue for application to serve with rd_kafka_poll()")
            .with_callback(from_stats(&latest, |stats| stats.replyq))
            .build();
        meter
            .u64_observable_gauge("kafka.producer.msgcnt")
            .with_description("Current number of messages in producer queues")
            .with_callback(from_stats(&latest, |stats| stats.msg_cnt))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.tx")
            .with_description("Total number of requests sent to Kafka brokers")
            .with_callback(from_stats(&latest, |stats| stats.tx))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.txbytes")
            .with_description("Total number of bytes transmitted to Kafka brokers")
            .with_callback(from_stats(&latest, |stats| stats.tx_bytes))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.rx")
            .with_description("Total number of responses received from Kafka brokers")
            .with_callback(from_stats(&latest, |stats| stats.rx))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.rxbytes")
            .with_description("Total number of bytes received from Kafka brokers")
            .with_callback(from_stats(&latest, |stats| stats.rx_bytes))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.txmsgs")
            .with_description("Total number of messages transmitted (produced) to Kafka brokers")
            .with_callback(from_stats(&latest, |stats| stats.txmsgs))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.txmsgsbytes")
            .with_description("Total number of message bytes (including framing, such as per-Message framing and MessageSet/batch framing) transmitted to Kafka brokers")
            .with_callback(from_stats(&latest, |stats| stats.txmsg_bytes))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.rxmsgs")
            .with_description("Total number of messages consumed, not including ignored messages (due to offset, etc), from Kafka brokers.")
            .with_callback(from_stats(&latest, |stats| stats.rxmsgs))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.rxmsgsbytes")
            .with_description(
                "Total number of message bytes (including framing) received from Kafka brokers",
            )
            .with_callback(from_stats(&latest, |stats| stats.rxmsg_bytes))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.broker.state")
            .with_description("Broker state (INIT, DOWN, CONNECT, AUTH, APIVERSION_QUERY, AUTH_HANDSHAKE, UP, UPDATE)")
            .with_callback(broker_callback(&latest, |observer, broker, attrs| {
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
                    let mut attrs = Vec::from(attrs);
                    attrs.push(KeyValue::new("state", state));
                    observer.observe(v, &attrs);
                }
            }))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.broker.outbufcnt")
            .with_description("Number of requests awaiting transmission to broker")
            .with_callback(from_broker(&latest, |broker| broker.outbuf_cnt))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.broker.outbufmsgcnt")
            .with_description("Number of messages awaiting transmission to broker")
            .with_callback(from_broker(&latest, |broker| broker.outbuf_msg_cnt))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.broker.waitrespcnt")
            .with_description("Number of requests in-flight to broker awaiting response")
            .with_callback(from_broker(&latest, |broker| broker.waitresp_cnt))
            .build();
        meter
            .i64_observable_gauge("kafka.producer.broker.waitrespmsgcnt")
            .with_description("Number of messages in-flight to broker awaiting response")
            .with_callback(from_broker(&latest, |broker| broker.waitresp_msg_cnt))
            .build();
        meter
            .u64_observable_gauge("kafka.producer.broker.tx")
            .with_description("Total number of requests sent")
            .with_callback(from_broker(&latest, |broker| broker.tx))
            .build();
        meter
            .u64_observable_gauge("kafka.producer.broker.txbytes")
            .with_description("Total number of bytes sent")
            .with_callback(from_broker(&latest, |broker| broker.txbytes))
            .build();
        meter
            .u64_observable_gauge("kafka.producer.broker.rx")
            .with_description("Total number of responses received")
            .with_callback(from_broker(&latest, |broker| broker.rx))
            .build();
        meter
            .u64_observable_gauge("kafka.producer.broker.rxbytes")
            .with_description("Total number of bytes received")
            .with_callback(from_broker(&latest, |broker| broker.rxbytes))
            .build();

        Ok(TelemetryClientContext { latest })
    }
}
