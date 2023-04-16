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

use crate::kv;
use opentelemetry::metrics::{Meter, ObservableGauge};
use opentelemetry::Context;
use opentelemetry::KeyValue;
use rdkafka::{ClientContext, Statistics};

pub struct TelemetryClientContext {
    replyq: ObservableGauge<i64>,
    msgcnt: ObservableGauge<u64>,
    tx: ObservableGauge<i64>,
    tx_bytes: ObservableGauge<i64>,
    rx: ObservableGauge<i64>,
    rx_bytes: ObservableGauge<i64>,
    txmsgs: ObservableGauge<i64>,
    txmsg_bytes: ObservableGauge<i64>,
    rxmsgs: ObservableGauge<i64>,
    rxmsg_bytes: ObservableGauge<i64>,
    broker_state: ObservableGauge<i64>,
    broker_outbuf_cnt: ObservableGauge<i64>,
    broker_outbuf_msg_cnt: ObservableGauge<i64>,
    broker_waitresp_cnt: ObservableGauge<i64>,
    broker_waitresp_msg_cnt: ObservableGauge<i64>,
    broker_tx: ObservableGauge<u64>,
    broker_txbytes: ObservableGauge<u64>,
    broker_rx: ObservableGauge<u64>,
    broker_rxbytes: ObservableGauge<u64>,
}

impl ClientContext for TelemetryClientContext {
    fn stats(&self, statistics: Statistics) {
        let context = Context::current();
        self.replyq.observe(&context, statistics.replyq, &[]);
        self.msgcnt.observe(&context, statistics.msg_cnt, &[]);
        self.tx.observe(&context, statistics.tx, &[]);
        self.tx_bytes.observe(&context, statistics.tx_bytes, &[]);
        self.rx.observe(&context, statistics.rx, &[]);
        self.rx_bytes.observe(&context, statistics.rx_bytes, &[]);
        self.txmsgs.observe(&context, statistics.txmsgs, &[]);
        self.txmsg_bytes
            .observe(&context, statistics.txmsg_bytes, &[]);
        self.rxmsgs.observe(&context, statistics.rxmsgs, &[]);
        self.rxmsg_bytes
            .observe(&context, statistics.rxmsg_bytes, &[]);

        for (name, broker) in statistics.brokers {
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
                self.broker_state.observe(
                    &context,
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
            self.broker_outbuf_cnt
                .observe(&context, broker.outbuf_cnt, attrs);
            self.broker_outbuf_msg_cnt
                .observe(&context, broker.outbuf_msg_cnt, attrs);
            self.broker_waitresp_cnt
                .observe(&context, broker.waitresp_cnt, attrs);
            self.broker_waitresp_msg_cnt
                .observe(&context, broker.waitresp_msg_cnt, attrs);
            self.broker_tx.observe(&context, broker.tx, attrs);
            self.broker_txbytes.observe(&context, broker.txbytes, attrs);
            self.broker_rx.observe(&context, broker.rx, attrs);
            self.broker_rxbytes.observe(&context, broker.rxbytes, attrs);
        }
    }
}

impl TelemetryClientContext {
    pub fn new(meter: &Meter) -> TelemetryClientContext {
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

        TelemetryClientContext {
            replyq,
            msgcnt,
            tx,
            tx_bytes,
            rx,
            rx_bytes,
            txmsgs,
            txmsg_bytes,
            rxmsgs,
            rxmsg_bytes,
            broker_state,
            broker_outbuf_cnt,
            broker_outbuf_msg_cnt,
            broker_waitresp_cnt,
            broker_waitresp_msg_cnt,
            broker_tx,
            broker_txbytes,
            broker_rx,
            broker_rxbytes,
        }
    }
}
