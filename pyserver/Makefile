.PHONY: all ci clean build test

ci: clean build test

all: build test

clean:
	cargo clean

build:
	maturin develop

test: build
	cargo test
	poetry run pytest
