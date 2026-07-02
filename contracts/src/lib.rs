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
mod upgrade;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec};

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
    // Members / Roles
    // ─────────────────────────────────────────────────────────────────────────

    pub fn add_member(
        env: Env,
        dao_id: u64,
        caller: Address,
        new_member: Address,
        role: Role,
    ) -> Result<(), ContractError> {
        dao::DAOContract::add_member(&env, dao_id, caller, new_member, role)
    }

    pub fn remove_member(
        env: Env,
        dao_id: u64,
        caller: Address,
        member_addr: Address,
    ) -> Result<(), ContractError> {
        dao::DAOContract::remove_member(&env, dao_id, caller, member_addr)
    }

    pub fn update_member_role(
        env: Env,
        dao_id: u64,
        caller: Address,
        member_addr: Address,
        new_role: Role,
    ) -> Result<(), ContractError> {
        dao::DAOContract::update_member_role(&env, dao_id, caller, member_addr, new_role)
    }

    pub fn get_members(env: Env, dao_id: u64) -> Vec<Member> {
        dao::DAOContract::get_members(&env, dao_id)
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

    pub fn get_employee(env: Env, dao_id: u64, employee_id: u64) -> Result<Employee, ContractError> {
        employee::EmployeeContract::get_employee(&env, dao_id, employee_id)
    }

    pub fn get_all_employees(env: Env, dao_id: u64) -> Vec<Employee> {
        employee::EmployeeContract::get_all_employees(&env, dao_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Treasury — token whitelist
    // ─────────────────────────────────────────────────────────────────────────

    pub fn add_token(
        env: Env,
        dao_id: u64,
        caller: Address,
        token_address: Address,
    ) -> Result<(), ContractError> {
        treasury::TreasuryContract::add_token(&env, dao_id, caller, token_address)
    }

    pub fn remove_token(
        env: Env,
        dao_id: u64,
        caller: Address,
        token_address: Address,
    ) -> Result<(), ContractError> {
        treasury::TreasuryContract::remove_token(&env, dao_id, caller, token_address)
    }

    pub fn is_token_whitelisted(env: Env, dao_id: u64, token_address: Address) -> bool {
        treasury::TreasuryContract::is_whitelisted(&env, dao_id, token_address)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Treasury — balances
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
        caller: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        treasury::TreasuryContract::withdraw(&env, dao_id, token_address, caller, to, amount)
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
        token_address: Address,
    ) -> Result<u64, ContractError> {
        payroll::PayrollContract::create_payroll(
            &env, dao_id, admin, period, employees, commitments,
            total_amount, merkle_root, token_address,
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
        salary: i128,
        randomness: Bytes,
        leaf_index: u64,
        merkle_proof: Vec<BytesN<32>>,
    ) -> Result<(), ContractError> {
        payroll::PayrollContract::employee_claim(
            &env, payroll_id, dao_id, employee_id, token_address,
            amount, salary, randomness, leaf_index, merkle_proof,
        )
    }

    pub fn cancel_payroll(
        env: Env,
        payroll_id: u64,
        dao_id: u64,
        admin: Address,
        token_address: Address,
    ) -> Result<(), ContractError> {
        payroll::PayrollContract::cancel_payroll(&env, payroll_id, dao_id, admin, token_address)
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
    // Upgradeability
    // ─────────────────────────────────────────────────────────────────────────

    pub fn set_verifying_key(
        env: Env,
        dao_id: u64,
        caller: Address,
        vk_bytes: Bytes,
    ) -> Result<(), ContractError> {
        upgrade::UpgradeContract::set_verifying_key(&env, dao_id, caller, vk_bytes)
    }

    pub fn get_verifying_key(env: Env) -> Option<VerifyingKey> {
        upgrade::UpgradeContract::get_verifying_key(&env)
    }

    pub fn upgrade_contract(
        env: Env,
        dao_id: u64,
        caller: Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<(), ContractError> {
        upgrade::UpgradeContract::upgrade_contract(&env, dao_id, caller, new_wasm_hash)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ZK utilities
    // ─────────────────────────────────────────────────────────────────────────

    pub fn compute_commitment(
        env: Env,
        employee_id: u64,
        salary: i128,
        randomness: Bytes,
    ) -> BytesN<32> {
        zk_verifier::ZKVerifier::compute_commitment(&env, employee_id, salary, &randomness)
    }
}
