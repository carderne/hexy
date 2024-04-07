.DEFAULT_GOAL = build

.PHONY: build
build: fmt
	cargo build

.PHONY: lint
lint:
	cargo clippy
