VENV := .venv/bin/activate

.PHONY: setup build-lambdas build-config-api build-etl-trigger build-authorizer \
        plan apply deploy clean smoke-test

##──────────────────────────────────────────────────────────────
## Setup — create venv and install cargo-lambda
##──────────────────────────────────────────────────────────────

setup:
	@echo "▸ Creating Python venv..."
	python3 -m venv .venv
	@echo "▸ Installing cargo-lambda..."
	. $(VENV) && pip install cargo-lambda
	@echo "✓ Setup complete — cargo-lambda installed in .venv"

##──────────────────────────────────────────────────────────────
## Build — cross-compile Rust Lambdas with cargo-lambda
##──────────────────────────────────────────────────────────────

build-lambdas: build-config-api build-etl-trigger build-authorizer

build-config-api:
	@echo "▸ Building config-api Lambda..."
	. $(VENV) && cargo lambda build --release -p api

build-etl-trigger:
	@echo "▸ Building etl-trigger Lambda..."
	. $(VENV) && cargo lambda build --release -p etl-trigger

build-authorizer:
	@echo "▸ Building authorizer Lambda..."
	. $(VENV) && cargo lambda build --release -p authorizer

##──────────────────────────────────────────────────────────────
## Infrastructure — OpenTofu
##──────────────────────────────────────────────────────────────

plan: build-lambdas
	@echo "▸ Initialising OpenTofu..."
	cd infra && tofu init -input=false
	@echo "▸ Planning..."
	cd infra && tofu plan -out=plan.out
	@echo "✓ Plan saved to infra/plan.out — run 'make apply' to deploy"

apply:
	@test -f infra/plan.out || (echo "✗ No plan found. Run 'make plan' first." && exit 1)
	@echo "▸ Applying plan..."
	cd infra && tofu apply plan.out
	@echo "✓ Deployed!"

##──────────────────────────────────────────────────────────────
## All-in-one
##──────────────────────────────────────────────────────────────

deploy: plan apply

##──────────────────────────────────────────────────────────────
## OpenAPI spec — build the API binary and dump the spec to JSON
##──────────────────────────────────────────────────────────────

openapi:
	@echo "▸ Building config-api…"
	cargo build -p api
	@echo "▸ Generating OpenAPI spec…"
	./target/debug/config-api --openapi > openapi.json
	@echo "✓ Wrote openapi.json"
	@echo "▸ Generating TypeScript clients…"
	cd onboard-you-frontend && pnpm openapi-ts
	cd test/smoke-test && npx openapi-ts
	@echo "✓ TypeScript clients generated"

##──────────────────────────────────────────────────────────────
## Smoke tests — sync credentials from tofu output, then run
##──────────────────────────────────────────────────────────────

smoke-test:
	@echo "▸ Syncing smoke-test .env from tofu output…"
	cd test/smoke-test && bash ./sync-env.sh
	@echo "▸ Running smoke tests…"
	cd test/smoke-test && pnpm test

clean:
	cargo clean
	rm -rf infra/.build infra/plan.out
