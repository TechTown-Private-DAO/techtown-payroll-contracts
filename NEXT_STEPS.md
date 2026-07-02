# TechTown-Private-DAO — Next Steps

> Current state: contracts build, 20/20 tests pass, release WASM compiles.

---

## Immediate (before any deployment)

### 1. Real ZK Proof Verification
The verifier is a structural stub. You need an actual circuit. Recommended path for Stellar/Soroban:

- Write the payroll circuit in [circom](https://docs.circom.io/) — inputs: salary array, randomness array, employee IDs, total sum constraint
- Compile with snarkjs to get a Groth16 verifying key
- Embed the VK as a contract constant or store it via a one-time admin `set_verifying_key()` function
- Replace the placeholder body in `verify_payroll_proof` with the pairing check against that VK

This is the hardest piece and the one that makes the "confidential" part real.

### 2. Merkle Tree Generation
The backend needs to build the Merkle tree off-chain and produce inclusion proofs for each employee's claim. You need a consistent leaf hashing scheme — the contract currently uses `SHA-256(employee_id || amount)`. Make sure the backend matches that exactly.

### 3. Salary Commitment Flow
`add_employee` takes a commitment hash but there is no on-chain verification before payroll. Wire `verify_salary_commitment` into `employee_claim` so employees prove knowledge of their salary before withdrawing.

---

## Contract Hardening (before open source launch)

### 4. Multi-admin / Role-based Access
Right now there is one admin per DAO. For a real DAO you want:

- A `members` map with roles: `Admin`, `Treasurer`, `Viewer`
- Treasury operations gated to `Treasurer` role, not just `admin`
- `add_member` / `remove_member` functions

### 5. Contract Upgradeability
Add an upgrade path via Soroban's `update_current_contract_wasm()`. Gate it behind multisig so no single key can upgrade unilaterally.

### 6. Payroll Scheduling / Periods
`period` is currently just a stored number. Add validation that an employee cannot be paid twice for the same period — check `last_payroll` period, not just `last_payroll` ID.

### 7. `release_budget` Wiring
`release_budget` exists but is never called. Wire it into `cancel_payroll` after approval. Currently you can only cancel pending payrolls — approved payrolls with locked funds have no escape hatch.

### 8. Token Whitelist
The treasury accepts any token address. Add a DAO-level whitelist of approved tokens so arbitrary tokens cannot be deposited.

---

## Repository Setup (for Drips / Gitcoin)

### 9. Add Missing Files

```
LICENSE
CONTRIBUTING.md
CHANGELOG.md
.github/
  ISSUE_TEMPLATE/
    bug_report.md
    feature_request.md
  workflows/
    ci.yml
  pull_request_template.md
```

### 10. CI Pipeline (`.github/workflows/ci.yml`)

```yaml
- cargo fmt --check
- cargo clippy -- -D warnings
- cargo test
- cargo build --release --target wasm32-unknown-unknown
```

The WASM target build is the real deployment artifact — confirm it compiles cleanly.

### 11. Contract README
Replace the current generic Soroban scaffold README with:

- What the contract does
- Deployed contract addresses (testnet first)
- How to run tests
- How to invoke functions with the Stellar CLI
- The ZK proof format specification (public inputs format, Merkle tree construction)

---

## After Contracts: The Other Two Repos

### 12. `techtown-payroll-backend` (Rust / Axum)
The backend is the ZK proof generator and relayer. Without it, `execute_payroll` cannot be called in production because nobody can produce the ZK proof. This is the **critical path dependency** for the entire confidential payroll feature.

Stack: Rust · Axum · PostgreSQL · Redis · Stellar SDK · Soroban SDK · Docker

### 13. `techtown-payroll-web` (Next.js)
Blocked on the backend for ZK features. Can start with wallet connection and DAO dashboard which do not require ZK proofs.

Stack: Next.js · React · TypeScript · Tailwind CSS · shadcn/ui · Stellar Wallet Kit

---

## Suggested Order (solo developer)

| Priority | Task |
|---|---|
| 1 | Confirm WASM build (`--target wasm32-unknown-unknown`) |
| 2 | Add LICENSE + CONTRIBUTING + CI workflow (needed for Drips listing) |
| 3 | Testnet deployment + Stellar CLI invoke scripts |
| 4 | Multi-admin roles |
| 5 | Upgrade path via multisig |
| 6 | Backend proof generator (unblocks all confidential features) |
| 7 | Frontend |
