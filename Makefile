.PHONY: generate fmt test build ci

generate:
	MYCEL_GENERATE_PROTO=1 cargo build -p myceldb-proto

fmt:
	cargo fmt --check

test:
	cargo test

build:
	cargo build

ci: fmt test build
