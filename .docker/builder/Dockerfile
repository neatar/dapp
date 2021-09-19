FROM rust:1.54.0

RUN apt-get update
RUN apt-get install -y git
RUN apt-get install -y binaryen wabt

RUN rustup default 1.54.0
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add clippy
RUN rustup component add rustfmt
RUN rustup component add rust-src
RUN cargo install cargo-audit --features=fix
RUN cargo install wasm-snip wasm-gc
