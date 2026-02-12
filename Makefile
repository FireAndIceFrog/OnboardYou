.PHONY: build-lambdas build-config-api build-etl-trigger tf-init tf-plan tf-apply deploy clean

##──────────────────────────────────────────────────────────────
## Build — cross-compile Rust Lambdas with cargo-lambda
##──────────────────────────────────────────────────────────────

build-lambdas: build-config-api build-etl-trigger

build-config-api:
	@echo "▸ Building config-api Lambda..."
	cargo lambda build --release -p api --output-format zip

build-etl-trigger:
	@echo "▸ Building etl-trigger Lambda..."
	cargo lambda build --release -p etl-trigger --output-format zip

##──────────────────────────────────────────────────────────────
## OpenTofu — infra provisioning
##──────────────────────────────────────────────────────────────

tf-init:
	cd infra && tofu init

tf-plan: build-lambdas
	cd infra && tofu plan -out=plan.out

tf-apply:
	cd infra && tofu apply plan.out

##──────────────────────────────────────────────────────────────
## All-in-one
##──────────────────────────────────────────────────────────────

deploy: build-lambdas tf-plan tf-apply
	@echo "✓ Deployed!"

clean:
	cargo clean
	rm -rf infra/.build infra/plan.out
