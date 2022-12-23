# kafka-proxy

## Installation

### Using Cargo
```bash
cargo install --git https://github.com/mkroli/kafka-proxy
```

## Usage
```
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
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information

Kafka Options:
  -b, --bootstrap-server <ADDRESS_LIST>            [default: 127.0.0.1:9092]
  -t, --topic <TOPIC>                              
      --schema-registry-url <SCHEMA_REGISTRY_URL>  
      --prometheus <ADDRESS>
```
