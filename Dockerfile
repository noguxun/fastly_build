FROM rust:1.49.0 as builder
WORKDIR /build
COPY src src
COPY Cargo.toml Cargo.toml
RUN cargo install --path .


FROM rust:1.49.0
RUN rustup target add wasm32-wasi --toolchain 1.49.0
WORKDIR /gu/
COPY build-test build-test
COPY --from=builder /usr/local/cargo/bin/fastlybuild /usr/local/bin/fastlybuild
RUN chmod 777 /gu
CMD ["fastlybuild"]
