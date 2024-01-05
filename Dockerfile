FROM rust:1.75 as builder

WORKDIR /efteling-node-exporter
RUN cargo new --bin efteling-node-exporter

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

COPY src src
RUN cargo build --release


FROM debian:bookworm-slim AS app

RUN apt-get update && apt install -y openssl ca-certificates

RUN update-ca-certificates

COPY --from=builder /efteling-node-exporter/target/release/efteling-node-exporter /efteling-node-exporter

CMD ["/efteling-node-exporter"]