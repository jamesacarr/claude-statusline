.PHONY: help build release install test fmt lint check clean

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build debug binary
	cargo build

release: ## Build release binary
	cargo build --release

install: ## Build release binary and install to cargo bin
	cargo install --path .

test: ## Run all tests
	cargo test

fmt: ## Format code
	cargo fmt

lint: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

check: ## Check formatting and linting
	cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings

clean: ## Clean build artifacts
	cargo clean
