FROM rust:1.48.0-alpine

WORKDIR /app

RUN cargo install cargo-chef
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin nyisnear

ARG RUST_LOG=info
ENV RUST_LOG=${RUST_LOG}
ENV TZ=Asia/Yekaterinburg
ENTRYPOINT ["/app/target/release/nyisnear"]
