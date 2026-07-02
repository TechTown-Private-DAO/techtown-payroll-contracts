use soroban_sdk::{Address, Bytes, BytesN, Env, Vec};
use crate::types::{EmployeeStatus, Payroll, PayrollStatus, SalaryCommitment, ZKProof};
use crate::storage::Storage;
use crate::errors::ContractError;
use crate::event::Events;
use crate::treasury::TreasuryContract;
use crate::zk_verifier::ZKVerifier;
use crate::dao::DAOContract;
use crate::types::Role;

pub struct PayrollContract;

impl PayrollContract {
    // ─────────────────────────────────────────────────────────────────────────
    // Create
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a new payroll proposal for a given period.
    ///
    /// - `period`       – payroll period identifier (e.g. YYYYMM as u64).
    /// - `employees`    – ordered list of employee IDs included in this payroll.
    /// - `commitments`  – matching commitment records (same order as employees).
    /// - `total_amount` – sum of all salaries, confirmed later by ZK proof.
    /// - `merkle_root`  – root of the Merkle tree of (employee_id ‖ amount) leaves.
    /// - `token_address`– the payment token (must be whitelisted).
    pub fn create_payroll(
        env: &Env,
        dao_id: u64,
        admin: Address,
        period: u64,
        employees: Vec<u64>,
        commitments: Vec<SalaryCommitment>,
        total_amount: i128,
        merkle_root: BytesN<32>,
        token_address: Address,
    ) -> Result<u64, ContractError> {
        admin.require_auth();
        DAOContract::require_role(env, dao_id, &admin, &Role::Admin)?;

        if employees.len() != commitments.len() {
            return Err(ContractError::InvalidAmount);
        }
        if total_amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Validate every employee and check for double-pay this period
        for i in 0..employees.len() {
            let emp_id = employees.get(i).unwrap();
            let emp = Storage::get_employee(env, dao_id, emp_id)?;

            if emp.status != EmployeeStatus::Active {
                return Err(ContractError::EmployeeNotActive);
            }

            // ── Period double-pay guard ───────────────────────────────────
            if Storage::is_period_paid(env, dao_id, emp_id, period) {
                return Err(ContractError::AlreadyPaidThisPeriod);
            }

            let commitment = commitments.get(i).unwrap();
            Storage::save_commitment(env, dao_id, emp_id, &commitment);
        }

        let slot = TreasuryContract::token_slot(env, &token_address);
        let payroll_id = Storage::next_payroll_id(env);

        let payroll = Payroll {
            id: payroll_id,
            dao_id,
            period,
            total_amount,
            employee_count: employees.len() as u32,
            status: PayrollStatus::Pending,
            merkle_root,
            token_slot: slot,
            created_at: env.ledger().timestamp(),
            approved_at: 0,
            executed_at: 0,
        };

        Storage::save_payroll(env, &payroll);
        Events::payroll_created(env, payroll_id, dao_id, total_amount);
        Ok(payroll_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Approve
    // ─────────────────────────────────────────────────────────────────────────

    /// Approve a pending payroll. Requires Admin role.
    pub fn approve_payroll(
        env: &Env,
        payroll_id: u64,
        dao_id: u64,
        approver: Address,
    ) -> Result<(), ContractError> {
        approver.require_auth();
        DAOContract::require_role(env, dao_id, &approver, &Role::Admin)?;

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

    // ─────────────────────────────────────────────────────────────────────────
    // Execute
    // ─────────────────────────────────────────────────────────────────────────

    /// Execute an approved payroll.
    ///
    /// Flow:
    ///   1. Verify the ZK proof against the payroll's public parameters.
    ///   2. Lock `total_amount` from the treasury into a per-payroll escrow.
    ///   3. Mark the payroll Executed — employees then claim individually.
    pub fn execute_payroll(
        env: &Env,
        payroll_id: u64,
        dao_id: u64,
        token_address: Address,
        zk_proof: ZKProof,
    ) -> Result<(), ContractError> {
        DAOContract::require_active(env, dao_id)?;

        let mut payroll = Storage::get_payroll(env, payroll_id)?;

        if payroll.dao_id != dao_id {
            return Err(ContractError::Unauthorized);
        }
        if payroll.status != PayrollStatus::Approved {
            return Err(ContractError::PayrollInvalidStatus);
        }

        // Verify token matches what was used at creation
        let expected_slot = TreasuryContract::token_slot(env, &token_address);
        if payroll.token_slot != expected_slot {
            return Err(ContractError::InvalidAmount);
        }

        // Verify ZK proof
        if !ZKVerifier::verify_payroll_proof(
            env,
            &zk_proof,
            payroll.total_amount,
            payroll.employee_count,
            &payroll.merkle_root,
        ) {
            return Err(ContractError::InvalidProof);
        }

        // Lock the full budget atomically
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

    // ─────────────────────────────────────────────────────────────────────────
    // Employee Claim
    // ─────────────────────────────────────────────────────────────────────────

    /// Employee claims their salary for an executed payroll.
    ///
    /// Checks (in order):
    ///   1. Payroll is Executed.
    ///   2. Employee has not already claimed.
    ///   3. Employee is Active (not Frozen / Removed).
    ///   4. Employee's wallet signs the transaction.
    ///   5. Salary commitment verification: H(salary ‖ randomness ‖ employee_id)
    ///      matches the on-chain commitment hash.
    ///   6. Merkle inclusion proof: (employee_id ‖ amount) leaf is in the payroll tree.
    ///   7. Period double-pay guard: this period is not already marked paid.
    pub fn employee_claim(
        env: &Env,
        payroll_id: u64,
        dao_id: u64,
        employee_id: u64,
        token_address: Address,
        amount: i128,
        salary: i128,
        randomness: Bytes,
        leaf_index: u64,
        merkle_proof: Vec<BytesN<32>>,
    ) -> Result<(), ContractError> {
        let payroll = Storage::get_payroll(env, payroll_id)?;

        if payroll.status != PayrollStatus::Executed {
            return Err(ContractError::PayrollInvalidStatus);
        }

        // Double-claim guard
        if Storage::is_claimed(env, payroll_id, employee_id) {
            return Err(ContractError::AlreadyClaimed);
        }

        let employee = Storage::get_employee(env, dao_id, employee_id)?;
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

        // ── Salary commitment verification ────────────────────────────────
        // Proves the employee knows their salary without revealing it on-chain.
        let commitment = Storage::get_commitment(env, dao_id, employee_id)?;
        if !ZKVerifier::verify_salary_commitment(
            env,
            &commitment.commitment_hash,
            employee_id,
            salary,
            &randomness,
        ) {
            return Err(ContractError::InvalidCommitment);
        }

        // ── Merkle inclusion proof ────────────────────────────────────────
        let leaf = Self::compute_claim_leaf(env, employee_id, amount);
        if !ZKVerifier::verify_merkle_proof(
            env,
            &leaf,
            &merkle_proof,
            leaf_index,
            &payroll.merkle_root,
        ) {
            return Err(ContractError::InvalidMerkleProof);
        }

        // ── Period double-pay guard ───────────────────────────────────────
        if Storage::is_period_paid(env, dao_id, employee_id, payroll.period) {
            return Err(ContractError::AlreadyPaidThisPeriod);
        }

        // ── Pay ───────────────────────────────────────────────────────────
        TreasuryContract::pay_employee(
            env,
            payroll_id,
            &token_address,
            &employee.wallet,
            amount,
        )?;

        // Mark claimed + update employee state
        Storage::mark_claimed(env, payroll_id, employee_id);
        Storage::mark_period_paid(env, dao_id, employee_id, payroll.period);

        let mut emp = employee;
        emp.last_payroll = payroll_id;
        emp.last_paid_period = payroll.period;
        Storage::save_employee(env, dao_id, &emp);

        Events::salary_claimed(env, payroll_id, employee_id, amount);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Cancel
    // ─────────────────────────────────────────────────────────────────────────

    /// Cancel a payroll. Requires Admin role.
    ///
    /// - Pending → Cancelled immediately, no funds to release.
    /// - Approved → Cancelled AND locked budget is released back to treasury.
    /// - Executed / Cancelled → error.
    pub fn cancel_payroll(
        env: &Env,
        payroll_id: u64,
        dao_id: u64,
        admin: Address,
        token_address: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        DAOContract::require_role(env, dao_id, &admin, &Role::Admin)?;

        let mut payroll = Storage::get_payroll(env, payroll_id)?;

        match payroll.status {
            PayrollStatus::Pending => {
                // Nothing locked yet — just cancel
            }
            PayrollStatus::Approved => {
                // Budget was NOT locked yet (locking happens at execute), just cancel
                // If somehow partial lock occurred, release it
                let slot = TreasuryContract::token_slot(env, &token_address);
                let locked = Storage::get_locked_balance(env, payroll_id, slot);
                if locked > 0 {
                    TreasuryContract::release_budget(
                        env,
                        dao_id,
                        &token_address,
                        payroll_id,
                        locked,
                    )?;
                }
            }
            _ => return Err(ContractError::PayrollInvalidStatus),
        }

        payroll.status = PayrollStatus::Cancelled;
        Storage::save_payroll(env, &payroll);

        Events::payroll_cancelled(env, payroll_id);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Queries
    // ─────────────────────────────────────────────────────────────────────────

    pub fn get_payroll(env: &Env, payroll_id: u64) -> Result<Payroll, ContractError> {
        Storage::get_payroll(env, payroll_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Leaf = SHA-256(employee_id_be8 ‖ amount_be16)
    fn compute_claim_leaf(env: &Env, employee_id: u64, amount: i128) -> BytesN<32> {
        let mut preimage = Bytes::new(env);
        for b in employee_id.to_be_bytes().iter() {
            preimage.push_back(*b);
        }
        for b in amount.to_be_bytes().iter() {
            preimage.push_back(*b);
        }
        env.crypto().sha256(&preimage).into()
    }
}
