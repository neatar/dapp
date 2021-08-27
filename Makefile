clean-out:
	rm -fr out
clean: clean-out
	cargo clean

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
# TODO add integration test

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
	@cp target/wasm32-unknown-unknown/release/avatar.wasm out/main.wasm
	@du -b out/main.wasm
	@sha256sum out/main.wasm

build:\
out/main.wasm
rebuild: clean-out build

deploy: rebuild
	near dev-deploy
deploy-new: deploy
	near --account_id $(shell cat neardev/dev-account) call $(shell cat neardev/dev-account) new
deploy-delete: neardev
	near delete $(shell cat neardev/dev-account) ${NEAR_DEV_ACCOUNT}
	rm -fr neardev

view_avatar_of_me:
	near view $(shell cat neardev/dev-account) avatar_of '{"account_id": "ilyar.testnet"}'
view_avatar_of_tb:
	near view $(shell cat neardev/dev-account) avatar_of '{"account_id": "tb.testnet"}'
call_avatar_create_me:
	near --account_id ${NEAR_DEV_ACCOUNT} call $(shell cat neardev/dev-account) avatar_create --amount 1
call_avatar_create_for_tb:
	near --account_id ${NEAR_DEV_ACCOUNT} call $(shell cat neardev/dev-account) avatar_create_for '{"owner_id":"tb.testnet"}' --amount 1
