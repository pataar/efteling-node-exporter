# efteling-node-exporter

A [Prometheus](https://prometheus.io/) exporter that exposes [Efteling](https://www.efteling.com/) attraction waiting times as metrics.

It fetches the park's public API, converts the response into Prometheus text format, and serves it on `/metrics`.

## Metrics

A single gauge is exported:

| Metric | Description | Labels |
| --- | --- | --- |
| `efteling_waiting_time` | Current waiting time for an attraction, in minutes | `id`, `name`, `empire`, `type` |

Attractions with no reported waiting time (e.g. closed) are omitted from the output.

Example:

```
# HELP efteling_waiting_time Waiting time for attractions
# TYPE efteling_waiting_time gauge
efteling_waiting_time{id="1", name="Baron 1898", empire="Ruigrijk", type="Attraction"} 25
```

Responses are cached in-memory for 30 seconds to avoid hammering the upstream API.

## Running

### With Docker

```sh
docker run -p 1337:1337 ghcr.io/pataar/efteling-node-exporter
```

### From source

Requires a recent Rust toolchain (see [`mise.toml`](./mise.toml)).

```sh
cargo run
```

The exporter listens on `0.0.0.0:1337`. Metrics are available at `http://localhost:1337/metrics`.

## Configuration

| Variable | Default | Description |
| --- | --- | --- |
| `RUST_LOG` | `efteling_node_exporter=info` | Log level / filter, per [`tracing-subscriber`](https://docs.rs/tracing-subscriber/) `EnvFilter` syntax |

## Prometheus scrape config

```yaml
scrape_configs:
  - job_name: efteling
    static_configs:
      - targets: ["localhost:1337"]
```

## Development

```sh
cargo test    # run tests
cargo clippy  # lint
cargo fmt     # format
```
