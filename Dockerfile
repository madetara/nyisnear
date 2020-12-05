FROM rust:1.48.0

WORKDIR /app

COPY . .

RUN cargo build --release

WORKDIR /app/target/release

ARG RUST_LOG=info
ENV RUST_LOG=${RUST_LOG}

CMD ["./nyisnear"]
