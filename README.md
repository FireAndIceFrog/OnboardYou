# OnboardYou Tooling & "Run Everything" Scripts

This project uses a mix of Rust, Python, JavaScript, and Infrastructure-as-Code tooling. The Makefile (`Makefile`) is the canonical build entrypoint; these docs summarize the required tools and provide cross-platform scripts to "run everything" end-to-end.

---

## ✅ Supported Environments

- **Linux**: Native Linux (Ubuntu, Debian, etc.) is the primary supported platform.
- **Windows**: **Strongly recommended to use WSL 2** (Windows Subsystem for Linux). The project relies on Unix tooling (`make`, shell scripts, etc.) and assumes a POSIX-like environment.

---

## 🧰 Required Tools (Linux & WSL)

### Core Toolchain
- `git` (version control)
- `make` (build orchestrator)
- `python3` (3.8+ recommended)
- `pip` (Python package installer)
- `rustup` + `cargo` (Rust toolchain)
- `cargo-lambda` (installed via `make setup`)
- `node` (Node.js, 18+ recommended)
- `npm` (Node package manager)
- `pnpm` (monorepo package manager)
- `tofu` (OpenTofu CLI; equivalent to Terraform)
- `aws` (AWS CLI v2)
- `psql` (PostgreSQL client)

### Optional / Commonly Useful
- `openssl` development libraries (often required for building Rust crates that use OpenSSL)
- `jq` (JSON CLI tool, handy for debugging outputs)

---

## 🧩 Installing System Dependencies

### Linux / WSL

Run the helper script to install system packages needed to build and run OnboardYou:

```bash
sudo ./scripts/install-deps.sh
```

### Windows (native PowerShell)

Run this script in an elevated PowerShell window:

```powershell
./scripts/install-deps.ps1
```

> ⚠️ The PowerShell installer uses `winget` and installs Git, Node.js, Python, Rust, PostgreSQL client, AWS CLI, and make.

---

## 🧩 What the "Run Everything" Workflow Looks Like

The scripts perform the following high-level steps:

1. Create a Python virtualenv and install `cargo-lambda` (`make setup`).
2. Build all Rust Lambda binaries (`make build-lambdas`).
3. Plan and apply infrastructure via OpenTofu (`make plan` + `make apply`).
4. Build the frontend packages and upload them (S3/CloudFront or GitHub Pages depending on infra mode).
5. Build the MCP server binary and sync environment files.
6. Run smoke tests.

---

## 🧪 If You Want to Run Only Part of the Workflow

Use the Makefile targets directly:

- `make setup` — create venv, install cargo-lambda, configure git hooks
- `make build-lambdas` — build all Lambda binaries
- `make plan` — terraform planning (OpenTofu)
- `make apply` — deploy the current plan
- `make deploy` — full end-to-end deploy (plan + apply + build-mcp + smoke-test + openapi + sync-env-mcp + upload-frontend)

---

## 📌 Notes

- The project expects `aws` credentials in the environment (e.g., `AWS_PROFILE`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, etc.).
- If you hit OpenSSL build errors while compiling Rust, set `OPENSSL_DIR` and `OPENSSL_LIB_DIR` (see `Makefile` comments).
- Running `make deploy` will incur AWS resource creation; be sure you want to deploy before running it in a production account.
