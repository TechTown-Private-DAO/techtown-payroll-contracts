use soroban_sdk::{contracttype, Address, Bytes, BytesN, String, Vec};

// ─────────────────────────────────────────────────────────────────────────────
// Roles
// ─────────────────────────────────────────────────────────────────────────────

/// Access roles within a DAO.
///
/// - `Admin`     – full control: settings, members, payroll approval, upgrade
/// - `Treasurer` – can deposit/withdraw treasury funds and execute payroll
/// - `Viewer`    – read-only (enforced off-chain; stored for frontend use)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Treasurer,
    Viewer,
}

/// A DAO member with an assigned role.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Member {
    pub address: Address,
    pub role: Role,
    pub added_at: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// DAO
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DAOConfig {
    pub name: String,
    pub symbol: String,
    /// Primary admin — always has Admin role regardless of members map.
    pub admin: Address,
    pub multisig_threshold: u32,
    pub total_members: u32,
    pub paused: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Employee
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmployeeStatus {
    Active,
    Frozen,
    Removed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Employee {
    pub id: u64,
    pub wallet: Address,
    pub department: String,
    pub status: EmployeeStatus,
    pub commitment_hash: BytesN<32>,
    pub joined_at: u64,
    /// ID of the last payroll this employee was paid in (0 = never paid)
    pub last_payroll: u64,
    /// Period of the last payroll (0 = never paid) — used for double-pay guard
    pub last_paid_period: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Salary Commitment
//
// On-chain we store only the commitment hash: H(salary || randomness || employee_id).
// The salary amount is revealed to the contract only at claim time via a
// ZK / Merkle proof — it is never stored in plaintext.
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SalaryCommitment {
    pub employee_id: u64,
    pub commitment_hash: BytesN<32>,
    pub period: u64,
    pub created_at: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Payroll
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PayrollStatus {
    Pending,
    Approved,
    Executed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payroll {
    pub id: u64,
    pub dao_id: u64,
    pub period: u64,
    /// Total locked amount confirmed by ZK proof
    pub total_amount: i128,
    pub employee_count: u32,
    pub status: PayrollStatus,
    /// Merkle root of all (employee_id, amount) claim leaves
    pub merkle_root: BytesN<32>,
    /// Token used for this payroll (slot hash)
    pub token_slot: u64,
    pub created_at: u64,
    pub approved_at: u64,   // 0 = not yet approved
    pub executed_at: u64,   // 0 = not yet executed
}

// ─────────────────────────────────────────────────────────────────────────────
// ZK Proof
//
// In production this carries a Groth16 proof (π_A, π_B, π_C) plus public
// inputs. The verifier checks these against the stored verifying key.
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZKProof {
    /// Serialised proof bytes (Groth16 / PLONK / etc.)
    pub proof: Bytes,
    /// Public inputs: [total_amount_be(16), employee_count_be(4), merkle_root(32), ...]
    pub public_inputs: Vec<Bytes>,
}

/// Stored verifying key for the payroll ZK circuit.
/// Set once by admin via `set_verifying_key`, used in every `execute_payroll`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifyingKey {
    /// Raw serialised Groth16 verifying key bytes
    pub vk_bytes: Bytes,
    pub set_at: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Multisig Proposal
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Active,
    Approved,
    Executed,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultisigProposal {
    pub id: u64,
    pub dao_id: u64,
    pub proposer: Address,
    pub target: Address,
    pub function: String,
    pub args: Bytes,
    pub approvals: Vec<Address>,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub executed_at: u64,   // 0 = not yet executed
}
