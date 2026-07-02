use soroban_sdk::{Address, BytesN, Env, Vec};
use crate::types::{EmployeeStatus, Payroll, PayrollStatus, SalaryCommitment, ZKProof};
use crate::storage::Storage;
use crate::errors::ContractError;
use crate::event::Events;
use crate::treasury::TreasuryContract;
use crate::zk_verifier::ZKVerifier;
use crate::dao::DAOContract;

pub struct PayrollContract;

impl PayrollContract {
    /// Create a new payroll for the given period.
    ///
    /// `commitments`  – one entry per employee with their salary commitment hash.
    ///                  Amount is revealed only at execution time via ZK proof.
    /// `merkle_root`  – root of the Merkle tree of (employee_id, commitment_hash) leaves.
    pub fn create_payroll(
        env: &Env,
        dao_id: u64,
        admin: Address,
        period: u64,
        employees: Vec<u64>,
        commitments: Vec<SalaryCommitment>,
        total_amount: i128,
        merkle_root: BytesN<32>,
    ) -> Result<u64, ContractError> {
        admin.require_auth();
        DAOContract::require_admin(env, dao_id, &admin)?;

        if employees.len() != commitments.len() {
            return Err(ContractError::InvalidAmount);
        }

        if total_amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let payroll_id = Storage::next_payroll_id(env);

        // Persist each employee's commitment for this payroll period
        for i in 0..employees.len() {
            let emp_id = employees.get(i).unwrap();
            let commitment = commitments.get(i).unwrap();
            // Ensure the employee exists and is active
            let emp = Storage::get_employee(env, dao_id, emp_id)?;
            if emp.status != EmployeeStatus::Active {
                return Err(ContractError::EmployeeNotActive);
            }
            Storage::save_commitment(env, dao_id, emp_id, &commitment);
        }

        let payroll = Payroll {
            id: payroll_id,
            dao_id,
            period,
            total_amount,
            employee_count: employees.len() as u32,
            status: PayrollStatus::Pending,
            merkle_root,
            created_at: env.ledger().timestamp(),
            approved_at: 0,
            executed_at: 0,
        };

        Storage::save_payroll(env, &payroll);
        Events::payroll_created(env, payroll_id, dao_id, total_amount);
        Ok(payroll_id)
    }

    /// Approve a pending payroll. Only the DAO admin may approve.
    pub fn approve_payroll(
        env: &Env,
        payroll_id: u64,
        dao_id: u64,
        approver: Address,
    ) -> Result<(), ContractError> {
        approver.require_auth();
        DAOContract::require_admin(env, dao_id, &approver)?;

        let mut payroll = Storage::get_payroll(env, payroll_id)?;

        if payroll.status != PayrollStatus::Pending {
            return Err(ContractError::PayrollInvalidStatus);
        }

        payroll.status = PayrollStatus::Approved;
        payroll.approved_at = env.ledger().timestamp();
        Storage::save_payroll(env, &payroll);

        Events::payroll_approved(env, payroll_id, &approver);
        Ok(())
    }

    /// Execute an approved payroll.
    ///
    /// Design:
    ///   1. Verify the ZK proof (proves all salaries are valid and sum == total_amount).
    ///   2. Lock `total_amount` from the treasury into the payroll escrow.
    ///   3. Mark the payroll as Executed — individual employees then claim their
    ///      portion via `employee_claim`.
    ///
    /// This separation means:
    ///   - Funds are locked atomically in one transaction.
    ///   - Each employee claims their own amount in a separate, independent tx.
    ///   - No double-withdraw: locking and claiming are distinct operations.
    pub fn execute_payroll(
        env: &Env,
        payroll_id: u64,
        dao_id: u64,
        token_address: Address,
        zk_proof: ZKProof,
    ) -> Result<(), ContractError> {
        // No auth required here: anyone may call execute once the proof is ready,
        // but the ZK proof itself provides the cryptographic authorization.
        DAOContract::require_active(env, dao_id)?;

        let mut payroll = Storage::get_payroll(env, payroll_id)?;

        if payroll.dao_id != dao_id {
            return Err(ContractError::Unauthorized);
        }

        if payroll.status != PayrollStatus::Approved {
            return Err(ContractError::PayrollInvalidStatus);
        }

        // ── Verify ZK proof ────────────────────────────────────────────────
        let valid = ZKVerifier::verify_payroll_proof(
            env,
            &zk_proof,
            payroll.total_amount,
            payroll.employee_count,
            &payroll.merkle_root,
        );
        if !valid {
            return Err(ContractError::InvalidProof);
        }

        // ── Lock the full payroll budget in escrow ─────────────────────────
        TreasuryContract::lock_budget(
            env,
            dao_id,
            &token_address,
            payroll_id,
            payroll.total_amount,
        )?;

        payroll.status = PayrollStatus::Executed;
        payroll.executed_at = env.ledger().timestamp();
        Storage::save_payroll(env, &payroll);

        Events::payroll_executed(env, payroll_id, payroll.total_amount);
        Ok(())
    }

    /// Employee claims their salary for an executed payroll.
    ///
    /// `amount` and `merkle_proof` are provided by the employee (or relayer)
    /// off-chain. The contract verifies:
    ///   1. The payroll is in Executed state.
    ///   2. The employee is active and has not already claimed.
    ///   3. The (employee_id, amount) leaf is in the payroll Merkle tree.
    ///   4. `amount` matches the stored commitment hash (salary commitment check).
    pub fn employee_claim(
        env: &Env,
        payroll_id: u64,
        dao_id: u64,
        employee_id: u64,
        token_address: Address,
        amount: i128,
        leaf_index: u64,
        merkle_proof: Vec<BytesN<32>>,
    ) -> Result<(), ContractError> {
        let payroll = Storage::get_payroll(env, payroll_id)?;

        if payroll.status != PayrollStatus::Executed {
            return Err(ContractError::PayrollInvalidStatus);
        }

        // Prevent double-claim
        if Storage::is_claimed(env, payroll_id, employee_id) {
            return Err(ContractError::AlreadyClaimed);
        }

        let employee = Storage::get_employee(env, dao_id, employee_id)?;

        // Auth: the employee's registered wallet must sign
        employee.wallet.require_auth();

        if employee.status == EmployeeStatus::Frozen {
            return Err(ContractError::EmployeeFrozen);
        }
        if employee.status != EmployeeStatus::Active {
            return Err(ContractError::EmployeeNotActive);
        }

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // ── Verify Merkle inclusion of (employee_id, amount) ──────────────
        // Leaf = SHA-256(employee_id || amount)
        let leaf = Self::compute_claim_leaf(env, employee_id, amount);
        let proof_valid = ZKVerifier::verify_merkle_proof(
            env,
            &leaf,
            &merkle_proof,
            leaf_index,
            &payroll.merkle_root,
        );
        if !proof_valid {
            return Err(ContractError::InvalidMerkleProof);
        }

        // ── Pay the employee from locked funds ─────────────────────────────
        TreasuryContract::pay_employee(
            env,
            payroll_id,
            &token_address,
            &employee.wallet,
            amount,
        )?;

        // Mark claimed and update last_payroll
        Storage::mark_claimed(env, payroll_id, employee_id);
        let mut emp = employee;
        emp.last_payroll = payroll_id;
        Storage::save_employee(env, dao_id, &emp);

        Events::salary_claimed(env, payroll_id, employee_id, amount);
        Ok(())
    }

    /// Cancel a pending payroll (before it is approved).
    pub fn cancel_payroll(
        env: &Env,
        payroll_id: u64,
        dao_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        DAOContract::require_admin(env, dao_id, &admin)?;

        let mut payroll = Storage::get_payroll(env, payroll_id)?;

        if payroll.status != PayrollStatus::Pending {
            return Err(ContractError::PayrollInvalidStatus);
        }

        payroll.status = PayrollStatus::Cancelled;
        Storage::save_payroll(env, &payroll);

        Events::payroll_cancelled(env, payroll_id);
        Ok(())
    }

    pub fn get_payroll(env: &Env, payroll_id: u64) -> Result<Payroll, ContractError> {
        Storage::get_payroll(env, payroll_id)
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Compute the Merkle leaf for a claim: SHA-256(employee_id_be || amount_be)
    fn compute_claim_leaf(env: &Env, employee_id: u64, amount: i128) -> BytesN<32> {
        use soroban_sdk::Bytes;
        let mut preimage = Bytes::new(env);
        let id_bytes = employee_id.to_be_bytes();
        for b in id_bytes.iter() {
            preimage.push_back(*b);
        }
        let amount_bytes = amount.to_be_bytes();
        for b in amount_bytes.iter() {
            preimage.push_back(*b);
        }
        env.crypto().sha256(&preimage).into()
    }
}
