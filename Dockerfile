FROM rust:1.48.0 as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.48.0 as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.48.0 as builder
WORKDIR /app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
RUN cargo build --release --bin nyisnear

FROM rust:1.48.0 as runtime
WORKDIR /app
COPY --from=builder /app/target/release/nyisnear /usr/local/bin
ARG RUST_LOG=info
ENV RUST_LOG=${RUST_LOG}
ENTRYPOINT ["/usr/local/bin/nyisnear"]
