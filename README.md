# cfddns

Simple dynamic DNS utility for Cloudflare

## Installation

Compile with Cargo:

```
cargo install --path .
```

If no errors, and installatiion was successful, you should be able to see help output here:

```
cfddns -h
```

## Usage

`cfddns` uses the `clap` package for command-line arguments, so either environment variables or flags are supported. An example of the former is provided below:

```
CFDDNS_AUTH_TOKEN=aaaaaaaaaaaaaaaaaaaaaaaaaaaa \
CFDDNS_ZONE_ID=aaaaaaaaaaaaaaaaaaaaaaaaaaaa \
CFDDNS_RECORD_ID=aaaaaaaaaaaaaaaaaaaaaaaaaaaa \
CFDDNS_RECORD_NAME=vpn \
./cfddns
```

By default, this will output nothing. You can use the `RUST_LOG` environment variable to adjust the logging level:

```
CFDDNS_AUTH_TOKEN=aaaaaaaaaaaaaaaaaaaaaaaaaaaa \
CFDDNS_ZONE_ID=aaaaaaaaaaaaaaaaaaaaaaaaaaaa \
CFDDNS_RECORD_ID=aaaaaaaaaaaaaaaaaaaaaaaaaaaa \
CFDDNS_RECORD_NAME=vpn \
RUST_LOG=info \
./cfddns
```
