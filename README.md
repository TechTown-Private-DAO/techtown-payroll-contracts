# TechTown Payroll Contracts

> Confidential contributor payroll on Stellar — salary amounts stay private through Zero-Knowledge Proofs.

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![Tests](https://img.shields.io/badge/tests-30%20passing-brightgreen)](#)
[![License](https://img.shields.io/badge/license-MIT-blue)](#license)
[![Soroban SDK](https://img.shields.io/badge/soroban--sdk-22.0.11-blueviolet)](https://docs.rs/soroban-sdk)

---

## What This Is

TechTown Payroll Contracts is the on-chain layer of [TechTown-Private-DAO](https://github.com/TechTown-Private-DAO) — an open-source payroll platform that lets DAOs, startups, and open-source organisations pay contributors on Stellar while keeping every salary amount confidential.

Instead of broadcasting `Alice gets $5,000` to the world, the system proves:

- The payroll follows company rules
- The treasury has sufficient funds
- Each employee receives a valid payment
- No salary figure is ever visible on-chain

This is achieved through **salary commitments** (`hash(salary || randomness || employee_id)`) and **Zero-Knowledge Proofs** that verify correctness without revealing values.

---

## Repository Layout

```
techtown-payroll-contracts/
├── contracts/
│   └── src/
│       ├── lib.rs           # Contract entry point — all public functions
│       ├── types.rs         # Shared types (contracttype structs/enums)
│       ├── errors.rs        # ContractError enum
│       ├── storage.rs       # Typed DataKey enum + all storage helpers
│       ├── dao.rs           # DAO lifecycle and roles system
│       ├── employee.rs      # Employee registry
│       ├── treasury.rs      # Token deposits, withdrawals, budget locking
│       ├── payroll.rs       # Payroll lifecycle + ZK-gated execution
│       ├── zk_verifier.rs   # ZK proof and Merkle proof verification
│       ├── multisig.rs      # Governance proposals
│       ├── upgrade.rs       # Verifying key + contract upgradeability
│       ├── event.rs         # On-chain event emitters
│       └── test.rs          # 30 integration tests
├── Cargo.toml               # Workspace
├── NEXT_STEPS.md            # Development roadmap
└── README.md
```

---

## Smart Contract Overview

### DAO (`dao.rs`)
Manages organisation creation and access control.

| Function | Role Required | Description |
|---|---|---|
| `create_dao` | — | Create a DAO; caller becomes first Admin |
| `update_dao_settings` | Admin | Change name, symbol, multisig threshold |
| `transfer_dao_admin` | Admin | Hand off primary admin address |
| `pause_dao` / `unpause_dao` | Admin | Emergency circuit breaker |
| `add_member` | Admin | Add a member with role (Admin/Treasurer/Viewer) |
| `remove_member` | Admin | Remove a member (primary admin is protected) |
| `update_member_role` | Admin | Reassign a member's role |
| `get_members` | — | List all DAO members |

**Role hierarchy:** `Admin > Treasurer > Viewer`

### Employee (`employee.rs`)
On-chain registry of contributors. Salary amounts are never stored — only commitment hashes.

| Function | Role Required | Description |
|---|---|---|
| `add_employee` | Admin | Register employee with wallet + commitment hash |
| `remove_employee` | Admin | Soft-delete (data preserved, status = Removed) |
| `freeze_employee` | Admin | Block salary claims without removing |
| `activate_employee` | Admin | Re-enable a frozen employee |
| `update_employee_wallet` | Employee | Self-service wallet rotation |
| `update_employee_commitment` | Admin | Update salary commitment (on pay change) |

### Treasury (`treasury.rs`)
Holds DAO funds. Only whitelisted tokens accepted.

| Function | Role Required | Description |
|---|---|---|
| `add_token` | Admin | Whitelist a token (SEP-41) for this DAO |
| `remove_token` | Admin | Remove a token from the whitelist |
| `deposit` | — | Deposit whitelisted tokens into the treasury |
| `withdraw` | Treasurer / Admin | Withdraw free treasury balance |
| `treasury_balance` | — | Query available balance for a token |
| `locked_balance` | — | Query escrowed balance for a payroll |

### Payroll (`payroll.rs`)
Full payroll lifecycle. Funds are locked atomically at execution; employees claim individually.

| Function | Role Required | Description |
|---|---|---|
| `create_payroll` | Admin | Propose a payroll with Merkle root + commitments |
| `approve_payroll` | Admin | Approve a pending payroll for execution |
| `execute_payroll` | — | Submit ZK proof → lock budget atomically |
| `claim_salary` | Employee | Prove salary + Merkle inclusion → receive payment |
| `cancel_payroll` | Admin | Cancel pending/approved payroll; releases locked funds |

**Guards on `claim_salary`:**
1. Payroll must be `Executed`
2. Employee has not already claimed (double-claim guard)
3. Employee is `Active` (not Frozen or Removed)
4. Salary commitment verified: `H(salary || randomness || employee_id) == commitment_hash`
5. Merkle inclusion proof validates `(employee_id || amount)` leaf against payroll root
6. Period double-pay guard: employee cannot claim twice for the same period

### Multisig (`multisig.rs`)
Governance proposals with configurable approval thresholds (e.g. 3-of-5).

| Function | Description |
|---|---|
| `create_proposal` | Create proposal; proposer auto-approves |
| `approve_proposal` | Add approval; auto-executes when threshold is met |
| `reject_proposal` | Admin veto on active proposals |
| `get_proposal` / `get_all_proposals` | Query proposals |

### Upgrade (`upgrade.rs`)
Controlled contract evolution, gated to Admin role.

| Function | Description |
|---|---|
| `set_verifying_key` | Store the Groth16 verifying key for the ZK circuit |
| `get_verifying_key` | Query the current verifying key |
| `upgrade_contract` | Upgrade contract WASM (upload hash via Stellar CLI first) |

### ZK Verifier (`zk_verifier.rs`)
All hashing uses the Soroban host's native SHA-256 — no external crates.

| Function | Description |
|---|---|
| `verify_payroll_proof` | Verify ZK proof against public inputs + Merkle root |
| `verify_salary_commitment` | Check `H(salary \|\| randomness \|\| id) == commitment_hash` |
| `verify_merkle_proof` | Binary Merkle inclusion proof |
| `compute_commitment` | Compute a commitment hash (callable via simulate) |

> **Note:** The pairing check in `verify_payroll_proof` is currently a structural stub. Replace the placeholder body with a real Groth16 verification once the circuit and verifying key are available. See `NEXT_STEPS.md`.

---

## Payroll Flow

```
Admin: create_dao()
Admin: add_token()           ← whitelist USDC / XLM
Admin: add_employee()        ← stores H(salary || rand || id)

Off-chain:
  Backend builds Merkle tree of (employee_id, amount) leaves
  Backend generates ZK proof (snarkjs / circom)

Admin: create_payroll()      ← submits Merkle root + commitments
Admin: approve_payroll()

Anyone: execute_payroll()    ← submits ZK proof → locks funds in escrow

Employee: claim_salary()     ← proves salary knowledge + Merkle path
                               receives payment from escrow
```

---

## ZK Proof Format

The contract expects `ZKProof { proof: Bytes, public_inputs: Vec<Bytes> }` where:

| Index | Content | Encoding |
|---|---|---|
| `public_inputs[0]` | `total_amount` | 16-byte big-endian `i128` |
| `public_inputs[1]` | `employee_count` | 4-byte big-endian `u32` |
| `public_inputs[2]` | `merkle_root` | 32 bytes |

**Merkle leaf:** `SHA-256(employee_id_be8 || amount_be16)`

**Commitment hash:** `SHA-256(salary_be16 || randomness || employee_id_be8)`

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) stable toolchain
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-stellar-cli)

```bash
rustup target add wasm32-unknown-unknown
```

### Clone and build

```bash
git clone https://github.com/TechTown-Private-DAO/techtown-payroll-contracts
cd techtown-payroll-contracts
cargo build
```

### Run tests

```bash
cargo test
```

### Build release WASM

```bash
cargo build --release --target wasm32-unknown-unknown
```

The compiled artifact will be at:
```
target/wasm32-unknown-unknown/release/techtown_payroll_contracts.wasm
```

### Deploy to Stellar testnet

```bash
# Upload the WASM
stellar contract upload \
  --wasm target/wasm32-unknown-unknown/release/techtown_payroll_contracts.wasm \
  --source <YOUR_KEYPAIR> \
  --network testnet

# Deploy the contract
stellar contract deploy \
  --wasm-hash <WASM_HASH_FROM_ABOVE> \
  --source <YOUR_KEYPAIR> \
  --network testnet
```

### Example invocation

```bash
# Create a DAO
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_KEYPAIR> \
  --network testnet \
  -- create_dao \
  --admin <ADMIN_ADDRESS> \
  --name "TechTown" \
  --symbol "TT" \
  --multisig_threshold 3

# Whitelist a token
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_KEYPAIR> \
  --network testnet \
  -- add_token \
  --dao_id 1 \
  --caller <ADMIN_ADDRESS> \
  --token_address <USDC_CONTRACT_ID>
```

---

## Architecture Decisions

**Why one contract instead of many?**
Soroban cross-contract calls add latency and instruction cost. Keeping DAO, treasury, payroll, and multisig in one contract reduces round-trips and simplifies the call graph for the backend relayer.

**Why typed `DataKey` enum instead of string keys?**
String keys built with `format!` are impossible in `no_std`. A `#[contracttype]` enum gives compact, deterministic, collision-free storage keys with zero runtime allocation.

**Why separate `execute_payroll` and `claim_salary`?**
Separating lock from disbursement means the full budget is locked atomically in one transaction (easy to verify on-chain), while each employee claims independently. This eliminates the double-withdraw bug common in single-transaction payroll designs.

**Why `token_slot` instead of storing `Address` in `DataKey`?**
`Address` cannot appear directly in a `#[contracttype]` enum variant used as a storage key. We derive a stable `u64` slot from `SHA-256(address XDR)` — deterministic, collision-resistant, and XDR-serialisable.

---

## Project Structure (Full Organisation)

This repository is one of three:

| Repo | Stack | Status |
|---|---|---|
| `techtown-payroll-contracts` | Rust · Soroban SDK | ✅ Active |
| `techtown-payroll-backend` | Rust · Axum · PostgreSQL · Redis | 🚧 Planned |
| `techtown-payroll-web` | Next.js · React · Tailwind · Stellar Wallet Kit | 🚧 Planned |

---

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a PR.

Good first issues are labelled [`good first issue`](https://github.com/TechTown-Private-DAO/techtown-payroll-contracts/issues?q=label%3A%22good+first+issue%22) on GitHub.

---

## License

MIT — see [LICENSE](LICENSE).
