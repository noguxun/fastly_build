FROM rust:1.49.0 as builder
WORKDIR /build
RUN rustup target add wasm32-wasi --toolchain 1.49.0
COPY src src
COPY Cargo.toml Cargo.toml
RUN cargo install --path .
WORKDIR /gu/
COPY build-test build-test
RUN chmod 777 /gu
