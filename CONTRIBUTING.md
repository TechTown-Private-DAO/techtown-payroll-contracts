# Contributing to TechTown Payroll Contracts

Thanks for helping improve the TechTown Payroll on-chain contracts. This
repository contains the Soroban smart contract layer for confidential payroll on
Stellar, so small, well-tested pull requests are easier to review than broad
changes.

## Local development setup

### Prerequisites

Install:

- Rust stable: <https://rustup.rs/>
- Stellar CLI: <https://developers.stellar.org/docs/tools/developer-tools/cli/install-stellar-cli>
- Git

Add the WebAssembly target used for Soroban contract builds:

```bash
rustup target add wasm32-unknown-unknown
```

### Clone the repository

```bash
git clone https://github.com/TechTown-Private-DAO/techtown-payroll-contracts.git
cd techtown-payroll-contracts
```

### Build the workspace

```bash
cargo build
```

### Run tests

```bash
cargo test
```

The repository CI currently runs `cargo test` and the release WASM build.
Please run both locally before opening a pull request when your change touches
contract behavior.

### Build the release WASM

```bash
cargo build --release --target wasm32-unknown-unknown
```

The compiled artifact is written to:

```text
target/wasm32-unknown-unknown/release/techtown_payroll_contracts.wasm
```

## Project structure

```text
contracts/src/lib.rs         contract entry point
contracts/src/types.rs       shared Soroban contract types
contracts/src/errors.rs      contract error enum
contracts/src/storage.rs     storage keys and helper functions
contracts/src/dao.rs         DAO lifecycle and role logic
contracts/src/employee.rs    employee registry
contracts/src/treasury.rs    treasury deposits, withdrawals, and balances
contracts/src/payroll.rs     payroll lifecycle and salary claims
contracts/src/zk_verifier.rs ZK proof and Merkle helper logic
contracts/src/multisig.rs    governance proposal logic
contracts/src/upgrade.rs     verifying key and upgrade helpers
contracts/src/event.rs       emitted contract events
contracts/src/test.rs        contract tests
```

Before changing contract behavior, read the matching module and the relevant
section in `README.md`. The current roadmap and known hardening tasks are in
`NEXT_STEPS.md`.

## Development workflow

1. Pick one issue and keep the pull request focused on that issue.
2. Create a branch from `main`.
3. Make the smallest useful change.
4. Run formatting, linting, tests, and the WASM build when relevant.
5. Open a pull request with a short summary and the checks you ran.

Useful local checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release --target wasm32-unknown-unknown
```

If a check is not available in your local environment, mention that in the pull
request and explain what you did run.

## Branch naming

Use short branch names with a prefix that describes the type of work:

```text
docs/add-contributing-guide
fix/payroll-claim-guard
feat/multisig-upgrade-path
test/treasury-withdrawal-coverage
chore/ci-clippy-check
```

## Commit messages

Use concise commit messages that explain the change. Conventional-style
prefixes are welcome:

```text
docs: add contributor guide
fix: prevent duplicate salary claims
feat: add DAO member role update
test: cover payroll cancellation
chore: add clippy to CI
```

Keep each commit scoped. Avoid mixing formatting-only changes with behavior
changes unless formatting is the entire purpose of the commit.

## Pull request checklist

Before opening a pull request, check:

- [ ] The change is tied to one issue or one clear improvement.
- [ ] `cargo fmt --check` passes, or formatting changes are intentional.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes when
      code changes are included.
- [ ] `cargo test` passes.
- [ ] `cargo build --release --target wasm32-unknown-unknown` passes when the
      contract build output may be affected.
- [ ] Documentation was updated when behavior, commands, or public interfaces
      changed.
- [ ] No private keys, funded accounts, RPC secrets, or deployment credentials
      are committed.

In the pull request description, include:

- What changed
- Why it changed
- Which checks you ran
- Any known limitations or follow-up work

## Review notes

Contract changes can affect payroll safety, treasury behavior, and upgrade
paths. Please be explicit when changing storage keys, authorization checks,
claim guards, token handling, or ZK/Merkle verification logic.

For documentation-only changes, keep the language direct and runnable. Prefer
commands that already appear in this repository over new tooling unless the new
tooling is part of the change.
