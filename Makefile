DC = docker compose run --rm dev

.PHONY: build test clippy fmt fmt-check bench fuzz ci clean shell

# Automatically configure git hooks on first make invocation
-include .hooks-installed
.hooks-installed:
	@git config core.hooksPath .githooks
	@touch .hooks-installed

build: .hooks-installed
	$(DC) cargo build

test: .hooks-installed
	$(DC) cargo test

clippy: .hooks-installed
	$(DC) cargo clippy -- -D warnings

fmt: .hooks-installed
	$(DC) cargo fmt

fmt-check: .hooks-installed
	$(DC) cargo fmt -- --check

bench: .hooks-installed
	$(DC) cargo bench

fuzz: .hooks-installed
	$(DC) cargo +nightly fuzz run push_pop -- -max_total_time=600

ci: clippy fmt-check test

clean:
	docker compose down -v
	@rm -f .hooks-installed

shell: .hooks-installed
	$(DC) bash
