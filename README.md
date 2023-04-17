# kafka-proxy

[![Build](https://github.com/mkroli/kafka-proxy/actions/workflows/build.yml/badge.svg)](https://github.com/mkroli/kafka-proxy/actions/workflows/build.yml)

Sidecar service to proxy from various protocols to Kafka.
It supports proxying text messages, binary messages (on stream based protocols via newline terminated base64 strings) and [Avro](https://avro.apache.org/) with [Schema Registry](https://docs.confluent.io/platform/current/schema-registry/index.html).
For Avro support, JSON messages are expected and converted according to the schema.

## Installation

### Download Binaries
[Latest Release](https://github.com/mkroli/kafka-proxy/releases/latest)

### Using Docker
```bash
docker run --rm -it ghcr.io/mkroli/kafka-proxy --help
```

### Using Cargo
```bash
cargo install --locked --bin kafka-proxy --git https://github.com/mkroli/kafka-proxy
```

## Usage
```
Service to proxy from various protocols to Kafka

Usage: kafka-proxy [OPTIONS] --topic <TOPIC> <COMMAND>

Commands:
  stdin       Read one message per line from stdin
  file        Read one message per line from a file
  unix-dgram  Receive messages from a Unix Datagram socket
  unix        Receive one message per line from a Unix socket
  udp         Receive messages from a UDP socket
  tcp         Receive one message per line from a TCP socket
  coap        Receive messages via CoAP
  rest        Receive messages via HTTP
  posixmq     Receive messages via Posix MQ
  nng         Receive messages via NNG
  help        Print this message or the help of the given subcommand(s)

Options:
      --prometheus <ADDRESS>  [env: KAFKA_PROXY_PROMETHEUS_ADDRESS=]
  -h, --help                  Print help
  -V, --version               Print version

Kafka Options:
  -b, --bootstrap-server <ADDRESS_LIST>
          [env: KAFKA_PROXY_BOOTSTRAP_SERVER=] [default: 127.0.0.1:9092]
  -t, --topic <TOPIC>
          [env: KAFKA_PROXY_TOPIC=]
      --producer-config <KEY=VALUE>
          [env: KAFKA_PROXY_PRODUCER_<KEY>=]
      --dead-letters <FILENAME>
          [env: KAFKA_PROXY_DEAD_LETTERS=]

Schema Registry Options:
      --schema-registry-url <SCHEMA_REGISTRY_URL>
          [env: KAFKA_PROXY_SCHEMA_REGISTRY_URL=]
      --schema-id <SCHEMA_ID>
          Use a specific schema id rather than the latest version [env: KAFKA_PROXY_SCHEMA_ID=]
      --topic-name
          Use TopicNameStrategy to derive the subject name (default)
      --record-name <RECORD_NAME>
          Use RecordNameStrategy to derive the subject name [env: KAFKA_PROXY_SCHEMA_REGISTRY_RECORD_NAME=]
      --topic-record-name <RECORD_NAME>
          Use TopicRecordNameStrategy to derive the subject name [env: KAFKA_PROXY_SCHEMA_REGISTRY_TOPIC_RECORD_NAME=]
```
