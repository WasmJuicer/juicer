
MONOREPO_PATH=$(shell pwd)/../..
CONTRACT_PATH=$(shell pwd)

cosmwasm-build:
	docker run --rm \
		-v "${MONOREPO_PATH}":/code \
		--mount type=volume,source="cw-juicer_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/rust-optimizer:0.12.5