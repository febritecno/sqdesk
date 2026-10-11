.DEFAULT_GOAL := dev

PNPM ?= pnpm
TAURI_DEV_PORT ?= 1420

.PHONY: help install check-tauri-dev-port dev dev-fast build package clean check test cargo-check

node_modules/.modules.yaml: package.json pnpm-lock.yaml
	$(PNPM) install --frozen-lockfile

help:
	@printf '%s\n' 'SQDesk development targets:'
	@printf '  %-18s %s\n' 'make / make dev' 'Start the desktop app in development mode'
	@printf '  %-18s %s\n' 'make dev-fast' 'Same, without DuckDB'
	@printf '  %-18s %s\n' 'make build' 'Type-check and build the frontend'
	@printf '  %-18s %s\n' 'make package' 'Build the desktop app package'
	@printf '  %-18s %s\n' 'make check' 'Format check, lint, type-check and tests'
	@printf '  %-18s %s\n' 'make test' 'Run frontend tests'
	@printf '  %-18s %s\n' 'make cargo-check' 'Check the Rust workspace'
	@printf '  %-18s %s\n' 'make clean' 'Remove Rust build artifacts'
	@printf '  %-18s %s\n' 'make install' 'Install dependencies'

install:
	$(PNPM) install --frozen-lockfile

check-tauri-dev-port:
	@if lsof -nP -iTCP:$(TAURI_DEV_PORT) -sTCP:LISTEN >/dev/null 2>&1; then \
		echo "Port $(TAURI_DEV_PORT) is already in use. SQDesk dev needs http://localhost:$(TAURI_DEV_PORT)."; \
		lsof -nP -iTCP:$(TAURI_DEV_PORT) -sTCP:LISTEN; \
		echo "Stop the process above, then run make dev again."; \
		exit 1; \
	fi

dev: node_modules/.modules.yaml check-tauri-dev-port
	$(PNPM) dev:tauri

dev-fast: node_modules/.modules.yaml check-tauri-dev-port
	# os-keyring must stay in sync with src-tauri defaults: without it the keychain
	# key is unreadable and the secret-store migration wizard reappears on every launch.
	RUST_MIN_STACK=16777216 $(PNPM) dev:tauri -- --no-default-features --features dynamodb,sqlite-bundled,os-keyring

build: node_modules/.modules.yaml
	$(PNPM) build:checked

package: node_modules/.modules.yaml
	$(PNPM) tauri build

clean:
	cargo clean

check: node_modules/.modules.yaml
	$(PNPM) check

test: node_modules/.modules.yaml
	$(PNPM) test

cargo-check:
	cargo check -p sqdesk
