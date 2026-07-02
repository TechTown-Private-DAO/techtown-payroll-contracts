# techtown-payroll-contracts — Code Flow

This repo contains the **Soroban smart contract** that runs on-chain on the Stellar network. It is the source of truth for all on-chain state: DAOs, employees, treasury, payroll, and multisig governance.

---

## Module Map

```
contracts/src/
├── lib.rs          ← Contract entry point — exposes every public function
├── types.rs        ← All shared data types (DAOConfig, Employee, Payroll, …)
├── errors.rs       ← ContractError enum used across all modules
├── storage.rs      ← DataKey enum + raw read/write helpers (Soroban storage)
├── dao.rs          ← DAO creation, settings, pause/unpause, admin transfer
├── employee.rs     ← Add/freeze/activate/remove employees, commitment hashes
├── treasury.rs     ← Deposit, withdraw, balance queries
├── payroll.rs      ← Create/approve/execute payroll, Merkle claim
├── zk_verifier.rs  ← ZK proof verification (Groth16/PLONK stub, future)
├── multisig.rs     ← Proposal creation, approval, execution
├── upgrade.rs      ← WASM contract upgrade guarded by admin
└── event.rs        ← Emit Soroban events consumed by the backend indexer
```

---

## Key Data Types (`types.rs`)

| Type | Purpose |
|---|---|
| `DAOConfig` | name, symbol, admin address, multisig threshold, pause flag |
| `Member` | Address + `Role` (Admin / Treasurer / Viewer) |
| `Employee` | Wallet, department, status, commitment_hash, join/pay timestamps |
| `SalaryCommitment` | `H(salary ‖ randomness ‖ employee_id)` — salary is never stored plaintext |
| `Payroll` | period, total_amount, merkle_root, status lifecycle |
| `ZKProof` | Groth16 proof bytes + public inputs |
| `MultisigProposal` | Target, function, args, approvals list, status lifecycle |

---

## On-Chain Flow

### 1 — DAO Creation
```
lib.rs::create_dao(admin, name, symbol, threshold)
  └─► dao.rs::DAOContract::create_dao
        ├─ storage.rs: write DAOConfig to persistent storage
        └─ event.rs:   emit dao_created event
```

### 2 — Add Employee
```
lib.rs::add_employee(dao_id, admin, wallet, department, commitment_hash)
  └─► employee.rs: validate admin auth → write Employee → emit employee_added
        └─ commitment_hash = H(salary ‖ randomness ‖ employee_id)
           (salary stays off-chain in the backend)
```

### 3 — Treasury Deposit
```
lib.rs::deposit(dao_id, treasurer, token, amount)
  └─► treasury.rs: auth check → Soroban token transfer → update balance → emit deposit
```

### 4 — Payroll Lifecycle
```
lib.rs::create_payroll(dao_id, admin, period, zk_proof)
  └─► payroll.rs: verify ZK proof → build Merkle root → write Payroll{Pending}
                                                         └─ emit payroll_created

lib.rs::approve_payroll(dao_id, approver, payroll_id)
  └─► payroll.rs + multisig.rs: collect approvals until threshold → Payroll{Approved}

lib.rs::execute_payroll(dao_id, executor, payroll_id, zk_proof)
  └─► payroll.rs: re-verify proof → lock treasury funds → Payroll{Executed}
                                                          └─ emit payroll_executed

lib.rs::claim_payroll(dao_id, employee, payroll_id, merkle_proof, amount)
  └─► payroll.rs: verify Merkle leaf → release token to employee wallet
                                       └─ emit payroll_claimed
```

### 5 — Multisig Governance
```
lib.rs::create_proposal(dao_id, proposer, target, function, args)
  └─► multisig.rs: write MultisigProposal{Active}

lib.rs::approve_proposal(dao_id, approver, proposal_id)
  └─► multisig.rs: append approval → if count >= threshold → execute call
                                                              └─ Proposal{Executed}
```

---

## Storage Layout (`storage.rs` — `DataKey` enum)

```
DataKey::DaoCounter             → u64 (auto-increment)
DataKey::Dao(dao_id)            → DAOConfig
DataKey::Member(dao_id, addr)   → Member
DataKey::EmpCounter(dao_id)     → u64
DataKey::Employee(dao_id, id)   → Employee
DataKey::Commitment(dao_id, id) → SalaryCommitment
DataKey::Payroll(dao_id, id)    → Payroll
DataKey::Proposal(dao_id, id)   → MultisigProposal
DataKey::TreasuryBalance(dao_id)→ i128
DataKey::VerifyingKey(dao_id)   → VerifyingKey
```

---

## How this repo connects to the rest of the system

```
techtown-payroll-contracts
        │
        │  compiled to WASM → deployed to Stellar Testnet/Mainnet
        │
        ▼
techtown-payroll-backend
  ├─ StellarService calls RPC to invoke contract functions
  ├─ Listens to on-chain events (dao_created, payroll_executed, …)
  └─ Mirrors on-chain state into PostgreSQL for fast querying
        │
        ▼
techtown-payroll-web
  └─ Reads backend REST API; signs & submits Stellar txns via Freighter
```

See `../flow.md` (root) for the full end-to-end picture.
