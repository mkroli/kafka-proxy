# kafka-proxy

[![Build](https://github.com/mkroli/kafka-proxy/actions/workflows/build.yml/badge.svg)](https://github.com/mkroli/kafka-proxy/actions/workflows/build.yml)

## Installation

### Download Binaries
[Latest Release](https://github.com/mkroli/kafka-proxy/releases/latest)

### Using Docker
```bash
docker run --rm -it ghcr.io/mkroli/kafka-proxy --help
```

### Using Cargo
```bash
cargo install --git https://github.com/mkroli/kafka-proxy
```

## Usage
```
Service to proxy from various protocols to Kafka

Usage: kafka-proxy [OPTIONS] --topic <TOPIC> <COMMAND>

Commands:
  stdin       
  file        
  unix-dgram  
  unix        
  udp         
  tcp         
  coap        
  rest        
  posixmq     
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Kafka Options:
  -b, --bootstrap-server <ADDRESS_LIST>
          [env: KAFKA_PROXY_BOOTSTRAP_SERVER=] [default: 127.0.0.1:9092]
  -t, --topic <TOPIC>
          [env: KAFKA_PROXY_TOPIC=]
      --schema-registry-url <SCHEMA_REGISTRY_URL>
          [env: KAFKA_PROXY_SCHEMA_REGISTRY_URL=]
      --producer-config <KEY=VALUE>
          [env: KAFKA_PROXY_PRODUCER_<KEY>=]
      --dead-letters <FILENAME>
          [env: KAFKA_PROXY_DEAD_LETTERS=]
      --prometheus <ADDRESS>
          [env: KAFKA_PROXY_PROMETHEUS_ADDRESS=]
```
