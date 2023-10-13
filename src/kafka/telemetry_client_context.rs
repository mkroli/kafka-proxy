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
use opentelemetry::metrics::Meter;
use opentelemetry::KeyValue;
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

impl TelemetryClientContext {
    pub fn new(meter: &Meter) -> Result<TelemetryClientContext> {
        let replyq = meter
            .i64_observable_gauge("kafka.producer.replyq")
            .with_description("Number of ops (callbacks, events, etc) waiting in queue for application to serve with rd_kafka_poll()")
            .init();
        let msgcnt = meter
            .u64_observable_gauge("kafka.producer.msgcnt")
            .with_description("Current number of messages in producer queues")
            .init();
        let tx = meter
            .i64_observable_gauge("kafka.producer.tx")
            .with_description("Total number of requests sent to Kafka brokers")
            .init();
        let tx_bytes = meter
            .i64_observable_gauge("kafka.producer.txbytes")
            .with_description("Total number of bytes transmitted to Kafka brokers")
            .init();
        let rx = meter
            .i64_observable_gauge("kafka.producer.rx")
            .with_description("Total number of responses received from Kafka brokers")
            .init();
        let rx_bytes = meter
            .i64_observable_gauge("kafka.producer.rxbytes")
            .with_description("Total number of bytes received from Kafka brokers")
            .init();
        let txmsgs = meter
            .i64_observable_gauge("kafka.producer.txmsgs")
            .with_description("Total number of messages transmitted (produced) to Kafka brokers")
            .init();
        let txmsg_bytes = meter
            .i64_observable_gauge("kafka.producer.txmsgsbytes")
            .with_description("Total number of message bytes (including framing, such as per-Message framing and MessageSet/batch framing) transmitted to Kafka brokers")
            .init();
        let rxmsgs = meter
            .i64_observable_gauge("kafka.producer.rxmsgs")
            .with_description("Total number of messages consumed, not including ignored messages (due to offset, etc), from Kafka brokers.")
            .init();
        let rxmsg_bytes = meter
            .i64_observable_gauge("kafka.producer.rxmsgsbytes")
            .with_description(
                "Total number of message bytes (including framing) received from Kafka brokers",
            )
            .init();

        let broker_state = meter
            .i64_observable_gauge("kafka.producer.broker.state")
            .with_description("Broker state (INIT, DOWN, CONNECT, AUTH, APIVERSION_QUERY, AUTH_HANDSHAKE, UP, UPDATE)")
            .init();
        let broker_outbuf_cnt = meter
            .i64_observable_gauge("kafka.producer.broker.outbufcnt")
            .with_description("Number of requests awaiting transmission to broker")
            .init();
        let broker_outbuf_msg_cnt = meter
            .i64_observable_gauge("kafka.producer.broker.outbufmsgcnt")
            .with_description("Number of messages awaiting transmission to broker")
            .init();
        let broker_waitresp_cnt = meter
            .i64_observable_gauge("kafka.producer.broker.waitrespcnt")
            .with_description("Number of requests in-flight to broker awaiting response")
            .init();
        let broker_waitresp_msg_cnt = meter
            .i64_observable_gauge("kafka.producer.broker.waitrespmsgcnt")
            .with_description("Number of messages in-flight to broker awaiting response")
            .init();
        let broker_tx = meter
            .u64_observable_gauge("kafka.producer.broker.tx")
            .with_description("Total number of requests sent")
            .init();
        let broker_txbytes = meter
            .u64_observable_gauge("kafka.producer.broker.txbytes")
            .with_description("Total number of bytes sent")
            .init();
        let broker_rx = meter
            .u64_observable_gauge("kafka.producer.broker.rx")
            .with_description("Total number of responses received")
            .init();
        let broker_rxbytes = meter
            .u64_observable_gauge("kafka.producer.broker.rxbytes")
            .with_description("Total number of bytes received")
            .init();

        let latest = Arc::new(RwLock::new(Statistics::default()));
        let context = TelemetryClientContext {
            latest: latest.clone(),
        };
        meter.register_callback(
            &[
                replyq.as_any(),
                msgcnt.as_any(),
                tx.as_any(),
                tx_bytes.as_any(),
                rx.as_any(),
                rx_bytes.as_any(),
                txmsgs.as_any(),
                txmsg_bytes.as_any(),
                rxmsgs.as_any(),
                rxmsg_bytes.as_any(),
                broker_state.as_any(),
                broker_outbuf_cnt.as_any(),
                broker_outbuf_msg_cnt.as_any(),
                broker_waitresp_cnt.as_any(),
                broker_waitresp_msg_cnt.as_any(),
                broker_tx.as_any(),
                broker_txbytes.as_any(),
                broker_rx.as_any(),
                broker_rxbytes.as_any(),
            ],
            move |observer| match latest.read() {
                Ok(stats) => {
                    observer.observe_i64(&replyq, stats.replyq, &[]);
                    observer.observe_u64(&msgcnt, stats.msg_cnt, &[]);
                    observer.observe_i64(&tx, stats.tx, &[]);
                    observer.observe_i64(&tx_bytes, stats.tx_bytes, &[]);
                    observer.observe_i64(&rx, stats.rx, &[]);
                    observer.observe_i64(&rx_bytes, stats.rx_bytes, &[]);
                    observer.observe_i64(&txmsgs, stats.txmsgs, &[]);
                    observer.observe_i64(&txmsg_bytes, stats.txmsg_bytes, &[]);
                    observer.observe_i64(&rxmsgs, stats.rxmsgs, &[]);
                    observer.observe_i64(&rxmsg_bytes, stats.rxmsg_bytes, &[]);

                    for (name, broker) in &stats.brokers {
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
                            observer.observe_i64(
                                &broker_state,
                                v,
                                kv![
                                    "name" => name.clone(),
                                    "nodeid" => broker.nodeid.to_string(),
                                    "nodename" => broker.nodename.clone(),
                                    "state" => state
                                ],
                            );
                        }
                        let attrs = kv![
                            "name" => name.clone(),
                            "nodeid" => broker.nodeid.to_string(),
                            "nodename" => broker.nodename.clone()
                        ];
                        observer.observe_i64(&broker_outbuf_cnt, broker.outbuf_cnt, attrs);
                        observer.observe_i64(&broker_outbuf_msg_cnt, broker.outbuf_msg_cnt, attrs);
                        observer.observe_i64(&broker_waitresp_cnt, broker.waitresp_cnt, attrs);
                        observer.observe_i64(
                            &broker_waitresp_msg_cnt,
                            broker.waitresp_msg_cnt,
                            attrs,
                        );
                        observer.observe_u64(&broker_tx, broker.tx, attrs);
                        observer.observe_u64(&broker_txbytes, broker.txbytes, attrs);
                        observer.observe_u64(&broker_rx, broker.rx, attrs);
                        observer.observe_u64(&broker_rxbytes, broker.rxbytes, attrs);
                    }
                }
                Err(e) => log::warn!("Failed to retrieve statistics: {e}"),
            },
        )?;
        Ok(context)
    }
}
