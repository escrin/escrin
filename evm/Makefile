.PHONY: all build test generate lint format

SRC_DIR := src test scripts

all: build generate

build:
	FOUNDRY_PROFILE=release forge build

generate: build | abi
	node export-abis.js

abi:
	mkdir abi

test:
	forge test

lint:
	forge fmt --check

format:
	forge fmt $^
