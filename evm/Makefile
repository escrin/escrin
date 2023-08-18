SRC_DIR := src test scripts
SOURCES := $(foreach dir,$(SRC_DIR),$(wildcard $(dir)/*.sol))

.PHONY: all build test

all: build

build:
	forge build

test:
	forge test

lint: $(SOURCES)
	forge fmt --check $^

format: $(SOURCES)
	forge fmt $^
