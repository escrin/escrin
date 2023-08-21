SRC_DIR := src test scripts
SOURCES := $(foreach dir,$(SRC_DIR),$(wildcard $(dir)/*.sol))

.PHONY: all build test

all: build generate

build:
	forge build

generate: build
	node export-abis.js

test:
	forge test

lint: $(SOURCES)
	forge fmt --check $^

format: $(SOURCES)
	forge fmt $^
