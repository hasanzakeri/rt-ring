.PHONY: setup test clippy fmt fmt-check bench test-loom fuzz ci

setup:
	@echo "Configuring git hooks..."
	@git config core.hooksPath .githooks
	@echo "Done."

test:
	cargo test

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

bench:
	cargo bench

test-loom:
	cargo test --features loom-tests --test loom_spsc

fuzz:
	cargo +nightly fuzz run push_pop -- -max_total_time=600

ci: clippy fmt-check test
