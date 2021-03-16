FROM rust:1.49.0 as builder
WORKDIR /gu
RUN rustup target add wasm32-wasi --toolchain 1.49.0
COPY build-test build-test
WORKDIR /gu/build-test


