use soroban_sdk::{contracttype, Address, Bytes, BytesN, String, Vec};

// ─────────────────────────────────────────────────────────────────────────────
// DAO
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DAOConfig {
    pub name: String,
    pub symbol: String,
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
    pub last_payroll: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Salary Commitment
//
// The amount is kept off-chain. On-chain we only store the commitment hash
// hash(salary || randomness || employee_id). The `amount` field here is used
// only inside execute_payroll where the admin has already proven the amount via
// ZK proof. It is NOT part of the commitment hash derivation.
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
    /// Merkle root of all (employee_id, commitment_hash) leaves
    pub merkle_root: BytesN<32>,
    pub created_at: u64,
    pub approved_at: u64,   // 0 = not yet approved
    pub executed_at: u64,   // 0 = not yet executed
}

// ─────────────────────────────────────────────────────────────────────────────
// ZK Proof
//
// In production this carries a Groth16 proof (π_A, π_B, π_C) plus public
// inputs. For now the verifier checks structural validity; a real verifier
// would use a pairing-based check against the circuit's verifying key.
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZKProof {
    /// Serialised proof bytes (Groth16 / PLONK / etc.)
    pub proof: Bytes,
    /// Public inputs: [total_amount_hi, total_amount_lo, employee_count, merkle_root_hash…]
    pub public_inputs: Vec<Bytes>,
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
