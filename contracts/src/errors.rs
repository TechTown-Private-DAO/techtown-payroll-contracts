use soroban_sdk::contracterror;

/// All errors that the TechTown Payroll contract can return.
///
/// Variants are numbered starting at 1 (0 is reserved by the SDK to mean
/// "success" in the XDR encoding).
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    // ── Auth / Access ────────────────────────────────────────────────────────
    Unauthorized = 1,
    InvalidThreshold = 2,

    // ── DAO ──────────────────────────────────────────────────────────────────
    DAONotFound = 3,
    DAOPaused = 12,
    DAONotPaused = 13,

    // ── Employee ─────────────────────────────────────────────────────────────
    EmployeeNotFound = 4,
    EmployeeAlreadyExists = 5,
    EmployeeFrozen = 22,
    EmployeeNotActive = 23,

    // ── Commitment / ZK ──────────────────────────────────────────────────────
    InvalidCommitment = 6,
    InvalidProof = 7,
    InvalidMerkleProof = 21,

    // ── Treasury ─────────────────────────────────────────────────────────────
    InsufficientBalance = 8,
    InvalidAmount = 14,

    // ── Payroll ──────────────────────────────────────────────────────────────
    PayrollNotFound = 9,
    PayrollAlreadyExecuted = 10,
    /// Operation requires a different payroll status than current
    PayrollInvalidStatus = 11,
    PeriodMismatch = 15,
    AlreadyClaimed = 24,

    // ── Multisig ─────────────────────────────────────────────────────────────
    AlreadyApproved = 16,
    NotEnoughApprovals = 17,
    ProposalNotFound = 18,
    ProposalExpired = 19,
    ProposalAlreadyExecuted = 20,
}
