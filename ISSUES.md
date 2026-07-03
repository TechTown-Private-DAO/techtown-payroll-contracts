# 📋 TechTown Payroll Contracts — Open Issues

This document tracks open issues for the `techtown-payroll-contracts` repository.
Each issue includes its difficulty level, labels, and a clear description to help new contributors get started quickly.

> **Want to contribute?** Read [CONTRIBUTING.md](CONTRIBUTING.md), pick an issue, comment on the corresponding GitHub issue to claim it, then open a PR. All skill levels are welcome!

---

## Table of Contents

- [Good First Issues](#good-first-issues)
- [Enhancements](#enhancements)
- [Testing](#testing)
- [Documentation](#documentation)
- [Security & Correctness](#security--correctness)

---

## Good First Issues

---

### #1 — Add `get_employee` query function
**Labels:** `good first issue` · `enhancement` · `smart-contract`
**Difficulty:** ⭐ Beginner

**Description:**
Currently, the contract exposes no single-employee lookup. Callers must fetch all employees and filter off-chain.

**Task:**
Add a `get_employee(dao_id: u32, employee_id: u64) -> Employee` public function in `employee.rs` and wire it up in `lib.rs`. Return an error if the employee doesn't exist.

**Acceptance Criteria:**
- [ ] Function is callable via `stellar contract invoke`
- [ ] Returns `ContractError::EmployeeNotFound` when the ID is invalid
- [ ] A unit test covers both the happy path and the not-found case

---

### #2 — Add `get_payroll` query function
**Labels:** `good first issue` · `enhancement` · `smart-contract`
**Difficulty:** ⭐ Beginner

**Description:**
There is no way to fetch a single payroll by ID without listing all payrolls. A direct lookup reduces off-chain complexity.

**Task:**
Add `get_payroll(dao_id: u32, payroll_id: u64) -> Payroll` in `payroll.rs` and expose it in `lib.rs`.

**Acceptance Criteria:**
- [ ] Function returns a `Payroll` struct
- [ ] Returns `ContractError::PayrollNotFound` on a bad ID
- [ ] Unit test included

---

### #3 — Add `get_proposal` shorthand in `lib.rs`
**Labels:** `good first issue` · `enhancement`
**Difficulty:** ⭐ Beginner

**Description:**
`get_proposal` and `get_all_proposals` exist in `multisig.rs` but are not yet publicly wired in `lib.rs`.

**Task:**
Expose both functions as public contract entry points.

**Acceptance Criteria:**
- [ ] Both functions reachable via the contract ABI
- [ ] Integration test in `test.rs` demonstrates fetching a proposal after creation

---

### #4 — Improve `ContractError` with descriptive messages
**Labels:** `good first issue` · `dx` · `documentation`
**Difficulty:** ⭐ Beginner

**Description:**
`errors.rs` lists error variants but the names alone are not always self-explanatory for contract callers.

**Task:**
Add a `/// <explanation>` doc comment to every variant in `ContractError`. Include what state caused the error and how a caller can fix it.

**Acceptance Criteria:**
- [ ] All variants have doc comments
- [ ] No code logic changes

---

### #5 — Extract magic numbers into named constants
**Labels:** `good first issue` · `refactor`
**Difficulty:** ⭐ Beginner

**Description:**
Values like Merkle tree depth limits, minimum multisig threshold, and max employee count appear as literals scattered across modules.

**Task:**
Define a `constants.rs` module and move every such literal there. Import the constants where needed.

**Acceptance Criteria:**
- [ ] No bare literals for tunable parameters remain in business logic files
- [ ] `constants.rs` is documented with the rationale for each value

---

## Enhancements

---

### #6 — Implement real Groth16 pairing check in `zk_verifier.rs`
**Labels:** `enhancement` · `zk-proofs` · `help wanted`
**Difficulty:** ⭐⭐⭐ Advanced

**Description:**
`verify_payroll_proof` currently contains a placeholder body. A real Groth16 (or PLONK) pairing check is required before mainnet deployment.

**Task:**
Integrate a production-ready ZK verification using the Soroban host's crypto primitives or a `no_std`-compatible crate. Verify against the verifying key stored via `set_verifying_key`.

**References:**
- `arkworks` — https://github.com/arkworks-rs
- Soroban host functions — https://developers.stellar.org/docs/smart-contracts

**Acceptance Criteria:**
- [ ] Placeholder replaced with real elliptic curve pairing check
- [ ] Verifying key parsed from on-chain storage
- [ ] At least one test with a valid proof passes, and one with an invalid proof fails

---

### #7 — Add employee count cap per DAO
**Labels:** `enhancement` · `smart-contract`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
There is currently no upper bound on the number of employees per DAO, which can make Merkle tree construction unbounded.

**Task:**
Add a configurable `max_employees: u32` field to DAO settings. Enforce the cap in `add_employee`. Allow Admins to update this value via `update_dao_settings`.

**Acceptance Criteria:**
- [ ] `add_employee` returns `ContractError::EmployeeLimitReached` when at cap
- [ ] Default cap is reasonable (e.g. 1000)
- [ ] Test covers the boundary condition

---

### #8 — Emit events for all state-changing functions
**Labels:** `enhancement` · `observability`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
`event.rs` exists but several functions (e.g. `freeze_employee`, `cancel_payroll`, `reject_proposal`) do not emit events, making off-chain indexing incomplete.

**Task:**
Audit every state-changing contract function. For each one that has no event emit, add a corresponding event emitter function in `event.rs` and call it.

**Acceptance Criteria:**
- [ ] All state-changing functions emit at least one event
- [ ] Event topic and data are documented in `event.rs`

---

### #9 — Add token balance check before payroll execution
**Labels:** `enhancement` · `smart-contract` · `security`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
`execute_payroll` locks funds but does not verify the DAO treasury holds a sufficient balance at call time, risking a revert mid-execution.

**Task:**
Before locking, assert `treasury_balance >= total_amount` and return a descriptive error if not.

**Acceptance Criteria:**
- [ ] `execute_payroll` fails early with `ContractError::InsufficientFunds` when balance is too low
- [ ] Unit test covers the underfunded case

---

### #10 — Implement payroll expiry / TTL
**Labels:** `enhancement` · `smart-contract`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
Approved payrolls that are never executed can lock treasury funds indefinitely. A TTL allows Admins to reclaim those funds.

**Task:**
Add an optional `expires_at: Option<u64>` (ledger timestamp) to the `Payroll` struct. In `execute_payroll`, reject execution if the ledger timestamp exceeds `expires_at`. Add a separate `expire_payroll` function callable by Admins.

**Acceptance Criteria:**
- [ ] Expired payrolls cannot be executed
- [ ] `expire_payroll` releases locked funds back to treasury
- [ ] Tests for both paths

---

### #11 — Support multi-token payrolls
**Labels:** `enhancement` · `smart-contract` · `help wanted`
**Difficulty:** ⭐⭐⭐ Advanced

**Description:**
Payrolls currently operate on a single token. Some DAOs pay contributors in a mix of stablecoins and governance tokens.

**Task:**
Extend `create_payroll` and `execute_payroll` to accept a `Vec<(token_slot, amount)>` breakdown. Lock each token separately. Adjust `claim_salary` accordingly.

**Acceptance Criteria:**
- [ ] Payroll can reference multiple whitelisted tokens
- [ ] Each token is locked/released independently
- [ ] Existing single-token tests still pass

---

## Testing

---

### #12 — Add fuzz tests for Merkle proof verification
**Labels:** `testing` · `security`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
`verify_merkle_proof` is security-critical. A fuzz harness would catch edge cases like empty proofs, single-leaf trees, and crafted second-preimage inputs.

**Task:**
Using `cargo-fuzz` or `proptest`, write a fuzz harness that generates random Merkle trees and proof paths, and asserts that only valid paths verify successfully.

**Acceptance Criteria:**
- [ ] Fuzz target runs with `cargo fuzz run` or `cargo test` (proptest)
- [ ] No panics found in initial run of at least 10,000 iterations

---

### #13 — Add integration test for the full payroll lifecycle
**Labels:** `testing`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
`test.rs` has individual unit tests, but no end-to-end test that exercises the complete flow: `create_dao → add_employee → create_payroll → approve_payroll → execute_payroll → claim_salary`.

**Task:**
Write a single integration test that walks through the entire lifecycle, asserting balances and statuses at each step.

**Acceptance Criteria:**
- [ ] Test passes with `cargo test`
- [ ] Each step's state is asserted before moving to the next

---

### #14 — Test all `ContractError` variants are reachable
**Labels:** `testing` · `quality`
**Difficulty:** ⭐ Beginner

**Description:**
Some error variants may be dead code if no test exercises the path that triggers them.

**Task:**
Write a test for each error variant in `ContractError` that confirms the variant is returned in the expected scenario.

**Acceptance Criteria:**
- [ ] Every variant has at least one test that asserts it is returned
- [ ] No `#[allow(dead_code)]` needed on any variant

---

### #15 — Add snapshot tests for contract ABI
**Labels:** `testing` · `dx`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
The `test_snapshots/` directory exists. Adding ABI snapshot tests prevents accidental breaking changes to the public contract interface.

**Task:**
Generate an ABI JSON snapshot for the contract and add a test that compares the current ABI against the snapshot, failing the build if they diverge.

**Acceptance Criteria:**
- [ ] Snapshot file checked into `test_snapshots/`
- [ ] Test fails if a public function signature changes without a snapshot update

---

## Documentation

---

### #16 — Write a `CONTRIBUTING.md`
**Labels:** `documentation` · `good first issue`
**Difficulty:** ⭐ Beginner

**Description:**
The README references `CONTRIBUTING.md` but the file does not exist. First-time contributors have no clear onboarding guide.

**Task:**
Create `CONTRIBUTING.md` covering:
- How to set up the dev environment
- How to run tests
- Branch naming and commit message conventions
- PR checklist (tests, docs, clippy clean)

**Acceptance Criteria:**
- [ ] File exists at repo root
- [ ] Covers all four topics above
- [ ] Reviewed by a maintainer

---

### #17 — Document the ZK proof format with worked examples
**Labels:** `documentation`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
The README describes the ZK proof format at a high level, but there is no step-by-step example of how an off-chain prover should construct `ZKProof { proof, public_inputs }`.

**Task:**
Add a `docs/zk-proof-format.md` file that includes:
- Encoding of each `public_inputs` field with byte-layout diagrams
- A worked example using pseudo-code or Python
- How `compute_commitment` output maps to a Merkle leaf

**Acceptance Criteria:**
- [ ] File exists in `docs/`
- [ ] At least one fully worked example with concrete byte values

---

### #18 — Add inline doc comments to all public contract functions
**Labels:** `documentation`
**Difficulty:** ⭐ Beginner

**Description:**
Most public functions in `lib.rs` lack `///` doc comments. Rustdoc output is sparse.

**Task:**
Add a `///` block to every `pub fn` in `lib.rs` describing parameters, return value, required role, and possible errors.

**Acceptance Criteria:**
- [ ] `cargo doc --no-deps` produces a complete HTML reference with no missing-doc warnings
- [ ] Every public function has at least 3 lines of documentation

---

## Security & Correctness

---

### #19 — Prevent double-claim across payroll periods at the storage level
**Labels:** `security` · `smart-contract`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
The period double-pay guard is documented but it is worth auditing whether the storage key used for the claim record is unique across (employee_id, payroll_id, period). A collision in the `DataKey` could allow a duplicate claim.

**Task:**
Audit `storage.rs` to confirm the claim record key includes all three dimensions. Add a test that attempts a double-claim and asserts it fails.

**Acceptance Criteria:**
- [ ] Storage key verified to be collision-free across the three dimensions
- [ ] Test for duplicate claim attempt is present and passes

---

### #20 — Add WASM size budget CI check
**Labels:** `ci` · `performance`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
Soroban contracts have instruction and size limits. There is currently no automated check to catch size regressions before they hit the network.

**Task:**
Add a step to the existing GitHub Actions workflow that builds the release WASM and fails if the artifact exceeds a configured size threshold (e.g. 100 KB). Print the current size on every run.

**Acceptance Criteria:**
- [ ] CI step reports WASM size in bytes on every push
- [ ] Build fails if size exceeds the threshold
- [ ] Threshold is configurable via a workflow input or constant at the top of the YAML

---

---

### #21 — Add `get_dao` query function
**Labels:** `good first issue` · `enhancement` · `smart-contract`
**Difficulty:** ⭐ Beginner

**Description:**
There is no public function to fetch a DAO's details by ID. Off-chain clients have to cache the creation event or store it themselves.

**Task:**
Add `get_dao(dao_id: u32) -> DAO` in `dao.rs` and expose it in `lib.rs`. Return `ContractError::DAONotFound` for an unknown ID.

**Acceptance Criteria:**
- [ ] Function is callable via `stellar contract invoke`
- [ ] Returns full `DAO` struct including name, symbol, and multisig threshold
- [ ] Unit test covers happy path and not-found case

---

### #22 — Enforce minimum multisig threshold of 1
**Labels:** `security` · `smart-contract`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
`create_dao` and `update_dao_settings` accept a `multisig_threshold` parameter but do not validate it is at least 1. A threshold of 0 would make approvals meaningless.

**Task:**
Add a guard in both functions that returns `ContractError::InvalidThreshold` when `multisig_threshold == 0` or exceeds the current member count.

**Acceptance Criteria:**
- [ ] `create_dao` with `threshold = 0` returns `ContractError::InvalidThreshold`
- [ ] `update_dao_settings` with `threshold > member_count` also returns the error
- [ ] Tests cover both invalid cases and a valid boundary value

---

### #23 — Add `list_payrolls` with pagination support
**Labels:** `enhancement` · `smart-contract`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
DAOs with a long payroll history will have unbounded storage reads if all payrolls are returned at once. Pagination reduces instruction cost per call.

**Task:**
Add `list_payrolls(dao_id: u32, page: u32, per_page: u32) -> Vec<Payroll>` in `payroll.rs`. Use a stored payroll counter to compute the offset range. Expose it in `lib.rs`.

**Acceptance Criteria:**
- [ ] Returns at most `per_page` entries per call (max 50)
- [ ] `page` is 1-indexed; page 1 returns the first `per_page` payrolls
- [ ] Empty vec returned (not an error) when page is out of range
- [ ] Unit test covers first page, last page, and out-of-range page

---

### #24 — Write a `SECURITY.md` with vulnerability disclosure policy
**Labels:** `documentation` · `security` · `good first issue`
**Difficulty:** ⭐ Beginner

**Description:**
The repository handles real on-chain funds. There is no documented process for responsible disclosure of security vulnerabilities.

**Task:**
Create `SECURITY.md` at the repo root. Include:
- Scope (what is and is not in scope)
- How to report a vulnerability (email or private GitHub advisory)
- Expected response time
- A note on the ZK stub that is not yet production-ready

**Acceptance Criteria:**
- [ ] File exists and covers all four sections above
- [ ] Reviewed and approved by a maintainer

---

### #25 — Add `clippy` and `rustfmt` CI enforcement
**Labels:** `ci` · `dx` · `good first issue`
**Difficulty:** ⭐ Beginner

**Description:**
The existing GitHub Actions workflow builds and tests the contract but does not enforce code style or lint rules. This allows unformatted or warned code to merge.

**Task:**
Add two new CI steps to the workflow:
1. `cargo fmt --check` — fails if any file is not formatted
2. `cargo clippy -- -D warnings` — fails on any Clippy warning

**Acceptance Criteria:**
- [ ] Both steps run on every push and PR
- [ ] A deliberately unformatted file causes the `fmt` step to fail
- [ ] A Clippy warning causes the `clippy` step to fail

---

*Last updated: 2026-07-03 · Maintainers: TechTown-Private-DAO*
