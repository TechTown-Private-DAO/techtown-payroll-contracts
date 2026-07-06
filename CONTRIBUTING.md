# Contributing to TechTown Payroll Contracts

Thanks for your interest in contributing! This guide covers everything you need to set up your environment, run tests, and submit changes.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Full Local Development Stack](#full-local-development-stack)
4. [Smart Contract Development](#smart-contract-development)
5. [Environment Variables](#environment-variables)
6. [Running Tests](#running-tests)
7. [Branching & Commit Conventions](#branching--commit-conventions)
8. [Pull Request Checklist](#pull-request-checklist)
9. [Code Style](#code-style)
10. [Deployment](#deployment)

---

## Prerequisites

- **Rust** stable toolchain (install via [rustup](https://rustup.rs/))
- **Stellar CLI** (install via [stellar.org docs](https://developers.stellar.org/docs/tools/developer-tools/cli/install-stellar-cli))
- **Docker & Docker Compose** (for Postgres, Redis, and local Stellar RPC)
- **Git** (for cloning and branching)

### Install the WASM target

```bash
rustup target add wasm32-unknown-unknown
```

---

## Quick Start

```bash
# Clone the repository
git clone https://github.com/TechTown-Private-DAO/techtown-payroll-contracts.git
cd techtown-payroll-contracts

# Copy environment variables
cp .env.example .env

# Build
cargo build

# Run tests
cargo test

# Build release WASM
cargo build --release --target wasm32-unknown-unknown
```

---

## Full Local Development Stack

The TechTown system spans multiple services. This section explains how to run the **complete local environment** for development.

### Services Overview

| Service | Purpose | Default Port |
|---|---|---|
| **PostgreSQL** | Backend database (payrolls, employees, DAOs, audit logs) | 5432 |
| **Redis** | Caching, session store, proof queue | 6379 |
| **Stellar RPC** | Local Horizon/Stellar Core for testnet simulation | 8000 |
| **Soroban RPC** | Contract simulation and host function access | 8001 |

### Starting Services with Docker Compose

```bash
docker compose up -d postgres redis stellar-rpc soroban-rpc
```

Verify all services are healthy:

```bash
docker compose ps
```

All services should show `healthy` or `running`.

### Database Migrations

Migrations live in `backend/migrations/` (when the backend repo is merged) or can be managed via `sqlx` / `diesel` CLI. For local development:

```bash
# Apply all pending migrations
cargo run --bin migrate

# Rollback last migration
cargo run --bin migrate -- --rollback
```

If migrations are SQL files in a `migrations/` directory:

```bash
# Using sqlx
sqlx migrate run --database-url postgres://localhost:5432/techtown

# Using diesel
diesel migration run
```

### Seeding Development Data

```bash
cargo run --bin seed
```

This creates a default DAO, an admin user, and a whitelisted test token.

### Stopping Services

```bash
docker compose down
```

To wipe all data and start fresh:

```bash
docker compose down -v
```

---

## Smart Contract Development

This repo (`techtown-payroll-contracts`) is the on-chain Soroban layer. It runs **independently** of the backend services, but the backend uses these contracts for payroll execution.

### Project Structure

```
contracts/src/
├── lib.rs           # Contract entry point — all public functions
├── types.rs         # Shared types (contracttype structs/enums)
├── errors.rs        # ContractError enum
├── storage.rs       # Typed DataKey enum + all storage helpers
├── dao.rs           # DAO lifecycle and roles system
├── employee.rs      # Employee registry
├── treasury.rs      # Token deposits, withdrawals, budget locking
├── payroll.rs       # Payroll lifecycle + ZK-gated execution
├── zk_verifier.rs   # ZK proof and Merkle proof verification
├── multisig.rs      # Governance proposals
├── upgrade.rs       # Verifying key + contract upgradeability
├── event.rs         # On-chain event emitters
└── test.rs          # Integration tests
```

### Running a Local Soroban Network

For testing contract deployment outside of unit tests:

```bash
# Start a local Soroban preview network
soroban network start \
  --local \
  --port 8000 \
  --rpc-port 8001
```

Add the local network to your Stellar CLI config:

```bash
stellar config set rpc_url http://localhost:8001
stellar config set network_passphrase "Standalone Network ; February 2017"
```

---

## Environment Variables

All environment variables are defined in `.env.example`. Copy it to `.env` before developing:

```bash
cp .env.example .env
```

### `.env.example` Reference

| Variable | Description | Default | Required |
|---|---|---|---|
| `DATABASE_URL` | PostgreSQL connection string for the backend | `postgres://postgres:postgres@localhost:5432/techtown` | Yes (backend) |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` | Yes (backend) |
| `STELLAR_RPC_URL` | Stellar Horizon RPC endpoint | `http://localhost:8000` | Yes (backend & contracts integration tests) |
| `SOROBAN_RPC_URL` | Soroban-specific RPC endpoint | `http://localhost:8001` | Yes (contracts simulation) |
| `STELLAR_NETWORK_PASSPHRASE` | Network identity string | `Test SDF Future Network ; October 2022` | Yes (testnet) |
| `CONTRACT_ID` | Deployed contract ID on the target network | — | Yes (deployment) |
| `ADMIN_SECRET_KEY` | Stellar secret key for the DAO admin account | — | Yes (deployment & tests) |
| `ADMIN_PUBLIC_KEY` | Stellar public key for the DAO admin account | — | Yes (tests) |
| `TOKEN_ADDRESS` | Whitelisted token contract ID (e.g., USDC) | — | No |
| `PORT` | Backend API server port | `3000` | Yes (backend) |
| `LOG_LEVEL` | Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`) | `info` | No |
| `ENVIRONMENT` | Runtime environment (`development`, `staging`, `production`) | `development` | No |
| `STELLAR_NETWORK` | Target Stellar network | `testnet` | Yes (backend) |
| `STELLAR_HORIZON_URL` | Horizon API endpoint for the target network | `https://horizon-testnet.stellar.org` | Yes (backend) |
| `STELLAR_FRIENDBOT_URL` | Friendbot faucet URL for testnet account funding | `https://friendbot.stellar.org` | No |
| `NEXT_PUBLIC_CONTRACT_ID` | Contract ID exposed to the frontend | — | Yes (web) |
| `NEXT_PUBLIC_STELLAR_NETWORK` | Network name used by the web frontend | `testnet` | Yes (web) |
| `NEXT_PUBLIC_STELLAR_HORIZON_URL` | Horizon URL used by the web frontend | `https://horizon-testnet.stellar.org` | Yes (web) |
| `NEXT_PUBLIC_API_URL` | Backend API base URL for the web frontend | `http://localhost:3000` | Yes (web) |
| `NEXT_PUBLIC_APP_URL` | Public URL of the web frontend | `http://localhost:3001` | Yes (web) |

### Variable Details

#### `DATABASE_URL`
PostgreSQL connection string. The backend uses `sqlx` / `diesel` for migrations and queries. Format: `postgres://<user>:<password>@<host>:<port>/<database>`.

#### `REDIS_URL`
Redis connection string. Used for caching employee commitments and proof job queues. Default `redis://localhost:6379` works with Docker Compose.

#### `STELLAR_RPC_URL` / `SOROBAN_RPC_URL`
- `STELLAR_RPC_URL` points to the Horizon RPC for account operations, transaction submission, and ledger queries.
- `SOROBAN_RPC_URL` points to the Soroban RPC for contract simulations and host function calls.

When running a local network, both typically point to `localhost` on their respective ports.

#### `STELLAR_NETWORK_PASSPHRASE`
Identifies the Stellar network. Use the testnet passphrase for non-local deployments. Never commit real secret keys.

#### `CONTRACT_ID`
The on-chain ID returned by `stellar contract deploy`. Required for `stellar contract invoke` and backend relayer configuration.

#### `ADMIN_SECRET_KEY` / `ADMIN_PUBLIC_KEY`
Stellar keypair for the DAO admin. During tests, the contract mocks auth, so real keys are not always needed, but they are required for CLI deployments and integration tests that exercise auth checks.

#### `TOKEN_ADDRESS`
Contract ID of the SEP-41 token (e.g., USDC on testnet) used for payroll payments. Must be whitelisted per DAO.

#### `PORT`
Backend server listen port. Must be available locally.

#### `LOG_LEVEL`
Controls filtering for the backend Axum logger. `debug` is useful when tracing ZK proof generation.

#### `ENVIRONMENT`
Affects error messages, CORS origins, and rate limiting in the backend. Keep `development` for local work.

#### `STELLAR_NETWORK`
Identifies which Stellar network the backend and frontend target. Use `testnet` for local and staging work; use `public` only for production.

#### `STELLAR_HORIZON_URL`
The Horizon server REST endpoint used by the backend and frontend to query accounts, transactions, and balances.

#### `STELLAR_FRIENDBOT_URL`
Friendbot endpoint used to fund new test accounts automatically. The backend uses this during onboarding tests.

#### `NEXT_PUBLIC_CONTRACT_ID`
The frontend's read-only reference to the deployed payroll contract ID. Required for all wallet interactions.

#### `NEXT_PUBLIC_STELLAR_NETWORK`
Network name the web frontend passes to Stellar Wallet Kit. Must match the backend's `STELLAR_NETWORK`.

#### `NEXT_PUBLIC_STELLAR_HORIZON_URL`
Horizon URL used by the web frontend for read-only queries. Must match `STELLAR_HORIZON_URL`.

#### `NEXT_PUBLIC_API_URL`
Backend REST API base URL consumed by the Next.js frontend. Change this if the backend runs on a non-default port.

#### `NEXT_PUBLIC_APP_URL`
The canonical public URL of the web frontend. Used for OAuth redirects and email links during development.

---

## Running Tests

### Smart Contract Tests

Run the full test suite:

```bash
cargo test
```

Run a specific test:

```bash
cargo test test_create_dao
```

Run tests with output:

```bash
cargo test -- --nocapture
```

### Backend Tests (when available)

```bash
cargo test --package techtown-payroll-backend
```

Integration tests require the full local stack to be running (Postgres, Redis, local Stellar RPC).

### CI Parity

The GitHub Actions workflow runs the following on every push:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release --target wasm32-unknown-unknown
```

Run these locally before opening a PR.

---

## Branching & Commit Conventions

### Branch Naming

Use the following prefixes:

| Type | Example | When to Use |
|---|---|---|
| Feature | `feature/add-get-employee` | New contract functions or backend endpoints |
| Fix | `fix/claim-period-guard` | Bug fixes |
| Refactor | `refactor/extract-constants` | Code cleanup without behavior change |
| Docs | `docs/contributing-guide` | Documentation only |
| Chore | `chore/update-ci` | Tooling, CI, dependencies |

Format: `<type>/<short-description>` with lowercase and hyphens.

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:** `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `perf`, `ci`

**Scope (optional):** `contracts`, `dao`, `treasury`, `payroll`, `zk`, `backend`, `web`

**Examples:**

```
feat(payroll): add employee_count cap validation
fix(dao): enforce minimum multisig threshold of 1
docs(contracts): document ZK proof format with examples
chore(ci): add clippy and fmt enforcement steps
```

**Rules:**
- Use lowercase and imperative mood (`add`, not `added` or `adds`)
- Subject line max 72 characters
- Body explains motivation and approach
- Reference issues: `Closes #19`

---

## Pull Request Checklist

Before opening a PR, verify the following:

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes (contracts)
- [ ] `cargo test` passes (backend, if applicable)
- [ ] Release WASM compiles: `cargo build --release --target wasm32-unknown-unknown`
- [ ] New public functions have `///` doc comments
- [ ] New code paths have corresponding tests
- [ ] `.env.example` updated if new environment variables were introduced
- [ ] `README.md` updated if public API or usage changed
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
- [ ] PR title follows the same convention as commit messages

### PR Title Format

```
feat(contracts): add get_employee query function
fix(payroll): prevent double-claim guard bypass
docs: add CONTRIBUTING.md
chore(deps): bump soroban-sdk to 22.x
```

Squash commits when merging unless the history is individually reviewable.

---

## Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting; treat warnings as errors
- Add doc comments to every public function (`///` blocks with parameters, returns, errors, and required roles)
- Prefer explicit types over `_` in public function signatures
- Keep functions small; extract helpers when a function exceeds ~50 lines
- Use `Result` with `ContractError` for all contract-visible fallible operations

---

## Deployment

### Build WASM Artifact

```bash
cargo build --release --target wasm32-unknown-unknown
```

Artifact location:
```
target/wasm32-unknown-unknown/release/techtown_payroll_contracts.wasm
```

### Upload to Stellar Testnet

```bash
stellar contract upload \
  --wasm target/wasm32-unknown-unknown/release/techtown_payroll_contracts.wasm \
  --source <ADMIN_SECRET_KEY> \
  --network testnet
```

### Deploy Contract

```bash
stellar contract deploy \
  --wasm-hash <WASM_HASH> \
  --source <ADMIN_SECRET_KEY> \
  --network testnet
```

### Example Invocation

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_KEYPAIR> \
  --network testnet \
  -- create_dao \
  --admin <ADMIN_ADDRESS> \
  --name "TechTown" \
  --symbol "TT" \
  --multisig_threshold 3
```

---

## Getting Help

- Open an issue for bugs or feature requests
- Tag good-first-issue for starter tasks
- Read `NEXT_STEPS.md` for the project roadmap
- Check `README.md` for contract architecture and ZK proof formats
