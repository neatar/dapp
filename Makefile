clean:
	cargo clean
	rm -fr out

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets

fmt:
	cargo fmt

audit-fix:
	cargo audit fix

audit:
	cargo audit

test-contract-integration: out/main.wasm
	cargo test --test integration

test-contract-unit:
	cargo test --lib

test-contract:\
test-contract-integration \
test-contract-unit

test:\
test-contract

qa:\
lint \
test

fix:\
audit-fix\
fmt

rustup:
	rustup component add clippy
	rustup component add rustfmt
	rustup component add rust-src
	rustup target add wasm32-unknown-unknown
	cargo install cargo-audit --features=fix

check:
	cargo check

out/main.wasm:
	cargo build --target wasm32-unknown-unknown --release
	@mkdir -p out
	@cp target/wasm32-unknown-unknown/release/intro.wasm out/main.wasm
	@du -b out/main.wasm
	@sha256sum out/main.wasm

build:\
out/main.wasm
