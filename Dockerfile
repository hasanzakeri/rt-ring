FROM rust:1.85

RUN rustup component add clippy rustfmt
RUN cargo install cargo-fuzz || true

WORKDIR /work
