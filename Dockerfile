FROM clux/muslrust:1.65.0-stable as chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin nyisnear

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/nyisnear nyisnear
ENV RUST_LOG="info"
ENTRYPOINT [ "/app/nyisnear" ]
