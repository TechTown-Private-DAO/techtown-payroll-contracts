#![no_std]

mod types;
mod errors;
mod storage;
mod event;
mod dao;
mod employee;
mod treasury;
mod payroll;
mod zk_verifier;
mod multisig;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec};

// Re-export types so tests and callers can import them cleanly
pub use types::*;
pub use errors::ContractError;
pub use storage::DataKey;

#[contract]
pub struct TechTownPayroll;

#[contractimpl]
impl TechTownPayroll {
    // ─────────────────────────────────────────────────────────────────────────
    // DAO
    // ─────────────────────────────────────────────────────────────────────────

    pub fn create_dao(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        multisig_threshold: u32,
    ) -> Result<u64, ContractError> {
        dao::DAOContract::create_dao(&env, admin, name, symbol, multisig_threshold)
    }

    pub fn update_dao_settings(
        env: Env,
        dao_id: u64,
        admin: Address,
        new_name: Option<String>,
        new_symbol: Option<String>,
        new_threshold: Option<u32>,
    ) -> Result<(), ContractError> {
        dao::DAOContract::update_settings(&env, dao_id, admin, new_name, new_symbol, new_threshold)
    }

    pub fn transfer_dao_admin(
        env: Env,
        dao_id: u64,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), ContractError> {
        dao::DAOContract::transfer_admin(&env, dao_id, current_admin, new_admin)
    }

    pub fn pause_dao(env: Env, dao_id: u64, admin: Address) -> Result<(), ContractError> {
        dao::DAOContract::pause(&env, dao_id, admin)
    }

    pub fn unpause_dao(env: Env, dao_id: u64, admin: Address) -> Result<(), ContractError> {
        dao::DAOContract::unpause(&env, dao_id, admin)
    }

    pub fn get_dao(env: Env, dao_id: u64) -> Result<DAOConfig, ContractError> {
        dao::DAOContract::get_dao_info(&env, dao_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Employee
    // ─────────────────────────────────────────────────────────────────────────

    pub fn add_employee(
        env: Env,
        dao_id: u64,
        admin: Address,
        wallet: Address,
        department: String,
        commitment_hash: BytesN<32>,
        period: u64,
    ) -> Result<u64, ContractError> {
        employee::EmployeeContract::add_employee(
            &env, dao_id, admin, wallet, department, commitment_hash, period,
        )
    }

    pub fn remove_employee(
        env: Env,
        dao_id: u64,
        employee_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        employee::EmployeeContract::remove_employee(&env, dao_id, employee_id, admin)
    }

    pub fn update_employee_wallet(
        env: Env,
        dao_id: u64,
        employee_id: u64,
        caller: Address,
        new_wallet: Address,
    ) -> Result<(), ContractError> {
        employee::EmployeeContract::update_wallet(&env, dao_id, employee_id, caller, new_wallet)
    }

    pub fn freeze_employee(
        env: Env,
        dao_id: u64,
        employee_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        employee::EmployeeContract::freeze_employee(&env, dao_id, employee_id, admin)
    }

    pub fn activate_employee(
        env: Env,
        dao_id: u64,
        employee_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        employee::EmployeeContract::activate_employee(&env, dao_id, employee_id, admin)
    }

    pub fn update_employee_commitment(
        env: Env,
        dao_id: u64,
        employee_id: u64,
        admin: Address,
        new_commitment_hash: BytesN<32>,
        period: u64,
    ) -> Result<(), ContractError> {
        employee::EmployeeContract::update_commitment(
            &env, dao_id, employee_id, admin, new_commitment_hash, period,
        )
    }

    pub fn get_employee(
        env: Env,
        dao_id: u64,
        employee_id: u64,
    ) -> Result<Employee, ContractError> {
        employee::EmployeeContract::get_employee(&env, dao_id, employee_id)
    }

    pub fn get_all_employees(env: Env, dao_id: u64) -> Vec<Employee> {
        employee::EmployeeContract::get_all_employees(&env, dao_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Treasury
    // ─────────────────────────────────────────────────────────────────────────

    pub fn deposit(
        env: Env,
        dao_id: u64,
        token_address: Address,
        from: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        treasury::TreasuryContract::deposit(&env, dao_id, token_address, from, amount)
    }

    pub fn withdraw(
        env: Env,
        dao_id: u64,
        token_address: Address,
        admin: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        treasury::TreasuryContract::withdraw(&env, dao_id, token_address, admin, to, amount)
    }

    pub fn treasury_balance(env: Env, dao_id: u64, token_address: Address) -> i128 {
        treasury::TreasuryContract::balance(&env, dao_id, token_address)
    }

    pub fn locked_balance(env: Env, payroll_id: u64, token_address: Address) -> i128 {
        treasury::TreasuryContract::locked_balance(&env, payroll_id, token_address)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Payroll
    // ─────────────────────────────────────────────────────────────────────────

    pub fn create_payroll(
        env: Env,
        dao_id: u64,
        admin: Address,
        period: u64,
        employees: Vec<u64>,
        commitments: Vec<SalaryCommitment>,
        total_amount: i128,
        merkle_root: BytesN<32>,
    ) -> Result<u64, ContractError> {
        payroll::PayrollContract::create_payroll(
            &env, dao_id, admin, period, employees, commitments, total_amount, merkle_root,
        )
    }

    pub fn approve_payroll(
        env: Env,
        payroll_id: u64,
        dao_id: u64,
        approver: Address,
    ) -> Result<(), ContractError> {
        payroll::PayrollContract::approve_payroll(&env, payroll_id, dao_id, approver)
    }

    pub fn execute_payroll(
        env: Env,
        payroll_id: u64,
        dao_id: u64,
        token_address: Address,
        zk_proof: ZKProof,
    ) -> Result<(), ContractError> {
        payroll::PayrollContract::execute_payroll(&env, payroll_id, dao_id, token_address, zk_proof)
    }

    pub fn claim_salary(
        env: Env,
        payroll_id: u64,
        dao_id: u64,
        employee_id: u64,
        token_address: Address,
        amount: i128,
        leaf_index: u64,
        merkle_proof: Vec<BytesN<32>>,
    ) -> Result<(), ContractError> {
        payroll::PayrollContract::employee_claim(
            &env, payroll_id, dao_id, employee_id, token_address,
            amount, leaf_index, merkle_proof,
        )
    }

    pub fn cancel_payroll(
        env: Env,
        payroll_id: u64,
        dao_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        payroll::PayrollContract::cancel_payroll(&env, payroll_id, dao_id, admin)
    }

    pub fn get_payroll(env: Env, payroll_id: u64) -> Result<Payroll, ContractError> {
        payroll::PayrollContract::get_payroll(&env, payroll_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Multisig
    // ─────────────────────────────────────────────────────────────────────────

    pub fn create_proposal(
        env: Env,
        dao_id: u64,
        proposer: Address,
        target: Address,
        function: String,
        args: Bytes,
    ) -> Result<u64, ContractError> {
        multisig::MultisigContract::create_proposal(&env, dao_id, proposer, target, function, args)
    }

    pub fn approve_proposal(
        env: Env,
        dao_id: u64,
        proposal_id: u64,
        approver: Address,
    ) -> Result<(), ContractError> {
        multisig::MultisigContract::approve_proposal(&env, dao_id, proposal_id, approver)
    }

    pub fn reject_proposal(
        env: Env,
        dao_id: u64,
        proposal_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        multisig::MultisigContract::reject_proposal(&env, dao_id, proposal_id, admin)
    }

    pub fn get_proposal(
        env: Env,
        dao_id: u64,
        proposal_id: u64,
    ) -> Result<MultisigProposal, ContractError> {
        multisig::MultisigContract::get_proposal(&env, dao_id, proposal_id)
    }

    pub fn get_all_proposals(env: Env, dao_id: u64) -> Vec<MultisigProposal> {
        multisig::MultisigContract::get_all_proposals(&env, dao_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ZK utility (callable off-chain via simulate, not via live transactions)
    // ─────────────────────────────────────────────────────────────────────────

    /// Compute a salary commitment hash on-chain (useful for backend verification).
    pub fn compute_commitment(
        env: Env,
        employee_id: u64,
        salary: i128,
        randomness: Bytes,
    ) -> BytesN<32> {
        zk_verifier::ZKVerifier::compute_commitment(&env, employee_id, salary, &randomness)
    }
}
