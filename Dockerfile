FROM rust:1.48.0 as deps

WORKDIR /app

RUN apt-get update
RUN apt-get install -y musl musl-dev musl-tools libssl-dev

RUN rustup target add x86_64-unknown-linux-musl

ENV PKG_CONFIG_ALLOW_CROSS=1

COPY . .

RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM debian:10-slim as run

WORKDIR /app

ARG RUST_LOG=trace
ENV RUST_LOG=${RUST_LOG}

COPY --from=deps /app/target/x86_64-unknown-linux-musl/release/nyisnear /app/nyisnear

CMD ["./nyisnear"]
