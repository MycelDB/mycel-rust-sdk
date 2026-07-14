.PHONY: fmt test build ci

fmt:
	cargo fmt --check

test:
	cargo test

build:
	cargo build

ci: fmt test build
