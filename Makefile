VENV := .venv/bin/activate

# defaults for building Rust lambdas with OpenSSL
#
# You can override any of these by exporting them in your shell, e.g.: 
#
#   export OPENSSL_DIR=/home/mathew/tmp/openssl-headers
#   export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
#   export OPENSSL_STATIC=1
#   export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
#
# or prepend them to the make command:
#
#   OPENSSL_DIR=/foo OPENSSL_LIB_DIR=/bar make deploy
#


.PHONY: setup build-lambdas build-config-api build-etl-trigger build-authorizer \
        plan apply deploy clean smoke-test assemble-pages openapi-ts setup-hooks \
        build-mcp sync-env-mcp snapshots

##──────────────────────────────────────────────────────────────
## Setup — create venv and install cargo-lambda
##──────────────────────────────────────────────────────────────

setup:
	@echo "▸ Creating Python venv..."
	python3 -m venv .venv
	@echo "▸ Installing cargo-lambda..."
	. $(VENV) && pip install cargo-lambda
	@echo "▸ Configuring git hooks..."
	git config core.hooksPath .githooks
	@echo "✓ Setup complete — cargo-lambda installed, git hooks configured"

##──────────────────────────────────────────────────────────────
## Build — cross-compile Rust Lambdas with cargo-lambda
##──────────────────────────────────────────────────────────────

build-lambdas: build-config-api build-etl-trigger build-authorizer

build-config-api:
	@echo "▸ Building config-api Lambda..."
	. $(VENV) && \
		OPENSSL_DIR=$(OPENSSL_DIR) \
		OPENSSL_LIB_DIR=$(OPENSSL_LIB_DIR) \
		OPENSSL_STATIC=$(OPENSSL_STATIC) \
		RUSTFLAGS="$(RUSTFLAGS)" \
		cargo lambda build --release -p api

build-etl-trigger:
	@echo "▸ Building etl-trigger Lambda..."
	. $(VENV) && \
		OPENSSL_DIR=$(OPENSSL_DIR) \
		OPENSSL_LIB_DIR=$(OPENSSL_LIB_DIR) \
		OPENSSL_STATIC=$(OPENSSL_STATIC) \
		RUSTFLAGS="$(RUSTFLAGS)" \
		cargo lambda build --release -p etl-trigger

build-authorizer:
	@echo "▸ Building authorizer Lambda..."
	. $(VENV) && cargo lambda build --release -p authorizer

snapshots:
	cargo insta test --all
##──────────────────────────────────────────────────────────────
## MCP server — native binary (not a Lambda)
##──────────────────────────────────────────────────────────────

build-mcp:
	@echo "▸ Building onboardyou-mcp..."
	cargo build --release -p onboardyou-mcp
	@echo "✓ Binary at target/release/onboardyou-mcp"

sync-env-mcp:
	@echo "▸ Syncing MCP .env from tofu output…"
	cd onboard-you-backend/mcp && bash ./sync-env.sh

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
## Frontend build & deploy
##──────────────────────────────────────────────────────────────
sync-env:
	@echo "▸ Syncing frontend .env from Terraform outputs…"
	cd onboard-you-frontend && npm run sync-env
	@echo "✓ Frontend .env synced"
	
# build the monorepo and deploy frontend artifacts
# prod → S3 + CloudFront  |  staging → GitHub Pages (via CI)  |  local → skip
upload-frontend: sync-env
	
	@echo "▸ Building frontend packages…"
	cd onboard-you-frontend && pnpm build

	@mode=$$(cd infra && tofu output -raw frontend_hosting_mode) && \
	if [ "$$mode" = "s3-cloudfront" ]; then \
		echo "▸ [prod] Looking up bucket and CloudFront distribution…" && \
		bucket=$$(cd infra && tofu output -raw frontend_bucket_name) && \
		distro=$$(cd infra && tofu output -raw frontend_cloudfront_id) && \
		echo "▸ Syncing platform app → s3://$$bucket/ …" && \
		aws s3 sync onboard-you-frontend/packages/platform/dist \
				s3://$$bucket/ --delete && \
		echo "▸ Syncing config bundle → s3://$$bucket/config/ …" && \
		aws s3 sync onboard-you-frontend/packages/config/dist \
				s3://$$bucket/config/ --delete && \
		echo "✓ Frontend artifacts uploaded" && \
		aws cloudfront create-invalidation --distribution-id $$distro --paths "/*" && \
		echo "✓ Invalidation submitted" ; \
	else \
		echo "▸ [staging/local] Frontend deployed via GitHub Pages CI — skipping upload" ; \
	fi

# Assemble build artifacts into _site/ for GitHub Pages deployment
# Copies 404.html from index.html for SPA client-side routing on GH Pages
assemble-pages:
	@echo "▸ Assembling GitHub Pages artifact…"
	rm -rf _site
	cp -r onboard-you-frontend/packages/platform/dist _site
	cp -r onboard-you-frontend/packages/config/dist _site/config
	cp _site/index.html _site/404.html
	@echo "✓ Artifact assembled in _site/"

frontend-url:
	@echo "▸ Fetching frontend URL from Terraform outputs…"
	cd infra && tofu output -raw frontend_url

##──────────────────────────────────────────────────────────────
## All-in-one
##──────────────────────────────────────────────────────────────

deploy: plan apply build-mcp smoke-test openapi sync-env-mcp upload-frontend

##──────────────────────────────────────────────────────────────
## OpenAPI spec — build the API binary and dump the spec to JSON
##──────────────────────────────────────────────────────────────

openapi: build-config-api
	@echo "▸ Generating OpenAPI spec…"
	./target/debug/config-api --openapi > openapi.json
	@echo "✓ Wrote openapi.json"
	@$(MAKE) openapi-ts

# Generate TypeScript clients from the existing openapi.json (no Rust build)
openapi-ts:
	@echo "▸ Generating TypeScript clients…"
	cd onboard-you-frontend && pnpm openapi-ts
	@echo "✓ TypeScript clients generated"

##──────────────────────────────────────────────────────────────
## Git hooks
##──────────────────────────────────────────────────────────────

setup-hooks:
	@echo "▸ Configuring git hooks path…"
	git config core.hooksPath .githooks
	@echo "✓ Git hooks configured — pre-commit will run 'make sync-env'"

##──────────────────────────────────────────────────────────────
## Smoke tests — sync credentials from tofu output, then run
##──────────────────────────────────────────────────────────────

smoke-test:
	@echo "▸ Syncing smoke-test .env from tofu output…"
	cd onboard-you-backend/test/smoke-test && bash ./sync-env.sh
	@echo "▸ Running smoke tests…"
	cd onboard-you-backend/test/smoke-test && pnpm test

clean:
	cargo clean
	rm -rf infra/.build infra/plan.out _site
