SHELL=bash
-include neardev/dev-account.env
-include .env

in-docker-%:
	bash .docker/builder/builder.sh make $*

clean-build-contract:
	rm -fr build/contract

clean: clean-build-contract
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

test-contract-integration: build/contract
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

check:
	cargo check

build-contract:
	bash src/contract/build.sh
rebuild-contract: clean-build-contract build-contract
redeploy-contract: deploy-delete-contract deploy-contract-init
deploy-contract: rebuild-contract
	near dev-deploy build/contract/neatar.wasm
deploy-contract-init: deploy-contract
	# near --account_id ${CONTRACT_NAME} call ${CONTRACT_NAME} init
	near --account_id $(shell cat neardev/dev-account) call $(shell cat neardev/dev-account) init
deploy-delete-contract: neardev
	near delete ${CONTRACT_NAME} ${NEAR_DEV_ACCOUNT}
	rm -fr neardev
migrate-contract: deploy-contract
	near --account_id ${CONTRACT_NAME} call ${CONTRACT_NAME} migrate
nft_metadata:
	near view ${CONTRACT_NAME} nft_metadata
nft_tokens:
	near view ${CONTRACT_NAME} nft_tokens
nft_tokens_for_owner:
	near view ${CONTRACT_NAME} nft_tokens_for_owner '{"account_id": "ilyar.testnet", "from_index": "0", "limit": 50}'
view_avatar_of_me:
	near view ${CONTRACT_NAME} avatar_of '{"account_id": "ilyar.testnet"}'
view_avatar_of_tb:
	near view ${CONTRACT_NAME} avatar_of '{"account_id": "tb.testnet"}'
call_avatar_create: call_avatar_create_me call_avatar_create_for_beta_tester
call_avatar_create_me:
	near --account_id ${NEAR_DEV_ACCOUNT} call ${CONTRACT_NAME} avatar_create --depositYocto 26370000000000000000000
# 0.02401721 - 0.0016 = 0.02241721
call_avatar_create_for_beta_tester: call_avatar_create_me
	near --account_id ${NEAR_DEV_ACCOUNT} call ${CONTRACT_NAME} avatar_create_for '{"owner_id":"tb.testnet"}' --amount 1
	near --account_id ${NEAR_DEV_ACCOUNT} call ${CONTRACT_NAME} avatar_create_for '{"owner_id":"jondou42.testnet"}' --amount 1
	near --account_id ${NEAR_DEV_ACCOUNT} call ${CONTRACT_NAME} avatar_create_for '{"owner_id":"anftimatter.testnet"}' --amount 1
#	near --account_id ${NEAR_DEV_ACCOUNT} call $(cat neardev/dev-account) nft_transfer '{ "token_id": "bafkreieyck4x2tujwtvmdu4dltjmff67khqviaewzixidj5zoa2sjrc62y", "receiver_id": "dev-1630244685532-93937831175122"}' --amount 0.000000000000000000000001
#	near --account_id $(cat neardev/dev-account) call $(cat neardev/dev-account) nft_transfer '{ "token_id": "bafkreieyck4x2tujwtvmdu4dltjmff67khqviaewzixidj5zoa2sjrc62y", "receiver_id": "ilyar.testnet"}' --amount 0.000000000000000000000001
