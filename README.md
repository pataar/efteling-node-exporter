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

Requires Rust 1.100 or newer. The exact toolchain is pinned in [`rust-toolchain.toml`](./rust-toolchain.toml), so `rustup` installs it on first use.

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
cargo test                  # run tests
cargo clippy --all-targets  # lint (clippy pedantic + Cargo lints, see [lints] in Cargo.toml)
cargo fmt                   # format
```

CI runs with `CARGO_BUILD_WARNINGS=deny`, so any rustc, clippy or Cargo lint warning (e.g. `cargo::unused_dependencies`) fails the build. Set the same variable locally to reproduce.

New dependency versions are only picked up after they have been on crates.io for 7 days (`global-min-publish-age` in [`.cargo/config.toml`](./.cargo/config.toml); Dependabot uses the same cooldown).
