FROM rust:1.48.0 as deps

WORKDIR /app

COPY . .

RUN cargo build --release

ARG RUST_LOG=trace
ENV RUST_LOG=${RUST_LOG}

CMD ["/app/target/release/nyisnear"]
