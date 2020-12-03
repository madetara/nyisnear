FROM rust:1.48.0 as deps

WORKDIR /app
# creating dummy main file to restore dependencies without actual compiling
RUN mkdir src && echo "fn main() {}" > src/main.rs

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --release


FROM deps as build

WORKDIR /dist
COPY --from=deps /app .
# removing dummy main file prior to copying actual program
RUN rm src/main.rs
COPY . .
RUN cargo build --release --frozen


FROM debian:buster-slim as run

COPY --from=build /dist/target/release/nyisnear /usr/local/bin/nyisnear

ARG RUST_LOG=trace
ENV RUST_LOG=${RUST_LOG}}

CMD ["nyisnear"]
