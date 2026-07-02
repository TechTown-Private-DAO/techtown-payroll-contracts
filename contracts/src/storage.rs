use soroban_sdk::{contracttype, Env, Vec};
use crate::types::*;
use crate::errors::ContractError;

// ─────────────────────────────────────────────────────────────────────────────
// Storage Keys
//
// Using a #[contracttype] enum as the key type is the idiomatic Soroban
// pattern. Each variant encodes both the entity type and its ID(s), so every
// record gets a unique, compact, deterministic key — no format! needed.
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    // Counters
    DaoCounter,
    EmployeeCounter,
    PayrollCounter,
    ProposalCounter,
    // Entities
    Dao(u64),
    Employee(u64, u64),         // (dao_id, employee_id)
    Payroll(u64),               // payroll_id
    Commitment(u64, u64),       // (dao_id, employee_id)
    Proposal(u64, u64),         // (dao_id, proposal_id)
    // Per-payroll claim tracking
    Claimed(u64, u64),          // (payroll_id, employee_id) → bool
    // Treasury balances
    TreasuryBalance(u64, u64),  // (dao_id, token_sym) — see note below
    LockedBalance(u64, u64),    // (payroll_id, token_sym)
    // Employee count per DAO (used for iteration)
    DaoEmployeeCount(u64),
}

// ─────────────────────────────────────────────────────────────────────────────
// Note on token keys:
//   Soroban Address is not a primitive, so we can't use it directly in an
//   enum variant that must derive contracttype without it also being
//   contracttype. Instead, callers pass a u64 "token slot" index managed by
//   the treasury — see treasury.rs. The treasury maps (dao_id, token_address)
//   to a slot integer stored in TreasuryBalance(dao_id, slot).
// ─────────────────────────────────────────────────────────────────────────────

pub struct Storage;

impl Storage {
    // ── Counters ─────────────────────────────────────────────────────────────

    pub fn next_dao_id(env: &Env) -> u64 {
        let k = DataKey::DaoCounter;
        let v: u64 = env.storage().persistent().get(&k).unwrap_or(0) + 1;
        env.storage().persistent().set(&k, &v);
        v
    }

    pub fn next_employee_id(env: &Env) -> u64 {
        let k = DataKey::EmployeeCounter;
        let v: u64 = env.storage().persistent().get(&k).unwrap_or(0) + 1;
        env.storage().persistent().set(&k, &v);
        v
    }

    pub fn next_payroll_id(env: &Env) -> u64 {
        let k = DataKey::PayrollCounter;
        let v: u64 = env.storage().persistent().get(&k).unwrap_or(0) + 1;
        env.storage().persistent().set(&k, &v);
        v
    }

    pub fn next_proposal_id(env: &Env) -> u64 {
        let k = DataKey::ProposalCounter;
        let v: u64 = env.storage().persistent().get(&k).unwrap_or(0) + 1;
        env.storage().persistent().set(&k, &v);
        v
    }

    // ── DAO ──────────────────────────────────────────────────────────────────

    pub fn save_dao(env: &Env, id: u64, config: &DAOConfig) {
        env.storage().persistent().set(&DataKey::Dao(id), config);
    }

    pub fn get_dao(env: &Env, id: u64) -> Result<DAOConfig, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Dao(id))
            .ok_or(ContractError::DAONotFound)
    }

    // ── Employee ─────────────────────────────────────────────────────────────

    pub fn save_employee(env: &Env, dao_id: u64, employee: &Employee) {
        env.storage()
            .persistent()
            .set(&DataKey::Employee(dao_id, employee.id), employee);
        // Track max employee id for iteration
        let count_key = DataKey::DaoEmployeeCount(dao_id);
        let current: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);
        if employee.id >= current {
            env.storage().persistent().set(&count_key, &(employee.id + 1));
        }
    }

    pub fn get_employee(env: &Env, dao_id: u64, employee_id: u64) -> Result<Employee, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Employee(dao_id, employee_id))
            .ok_or(ContractError::EmployeeNotFound)
    }

    pub fn get_all_employees(env: &Env, dao_id: u64) -> Vec<Employee> {
        let mut employees = Vec::new(env);
        let count_key = DataKey::DaoEmployeeCount(dao_id);
        let max: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);
        for id in 0..max {
            if let Some(emp) = env
                .storage()
                .persistent()
                .get::<DataKey, Employee>(&DataKey::Employee(dao_id, id))
            {
                employees.push_back(emp);
            }
        }
        employees
    }

    // ── Payroll ──────────────────────────────────────────────────────────────

    pub fn save_payroll(env: &Env, payroll: &Payroll) {
        env.storage()
            .persistent()
            .set(&DataKey::Payroll(payroll.id), payroll);
    }

    pub fn get_payroll(env: &Env, payroll_id: u64) -> Result<Payroll, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Payroll(payroll_id))
            .ok_or(ContractError::PayrollNotFound)
    }

    // ── Salary Commitment ────────────────────────────────────────────────────

    pub fn save_commitment(env: &Env, dao_id: u64, employee_id: u64, commitment: &SalaryCommitment) {
        env.storage()
            .persistent()
            .set(&DataKey::Commitment(dao_id, employee_id), commitment);
    }

    pub fn get_commitment(env: &Env, dao_id: u64, employee_id: u64) -> Result<SalaryCommitment, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Commitment(dao_id, employee_id))
            .ok_or(ContractError::InvalidCommitment)
    }

    // ── Multisig Proposal ────────────────────────────────────────────────────

    pub fn save_proposal(env: &Env, dao_id: u64, proposal: &MultisigProposal) {
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(dao_id, proposal.id), proposal);
    }

    pub fn get_proposal(env: &Env, dao_id: u64, proposal_id: u64) -> Result<MultisigProposal, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(dao_id, proposal_id))
            .ok_or(ContractError::ProposalNotFound)
    }

    // ── Claim tracking ───────────────────────────────────────────────────────

    pub fn mark_claimed(env: &Env, payroll_id: u64, employee_id: u64) {
        env.storage()
            .persistent()
            .set(&DataKey::Claimed(payroll_id, employee_id), &true);
    }

    pub fn is_claimed(env: &Env, payroll_id: u64, employee_id: u64) -> bool {
        env.storage()
            .persistent()
            .get::<DataKey, bool>(&DataKey::Claimed(payroll_id, employee_id))
            .unwrap_or(false)
    }

    // ── Treasury balances ────────────────────────────────────────────────────
    //
    // token_slot: a u64 that uniquely identifies (dao_id, token_address).
    // The treasury module derives it deterministically from the token address.

    pub fn get_treasury_balance(env: &Env, dao_id: u64, token_slot: u64) -> i128 {
        env.storage()
            .persistent()
            .get::<DataKey, i128>(&DataKey::TreasuryBalance(dao_id, token_slot))
            .unwrap_or(0)
    }

    pub fn set_treasury_balance(env: &Env, dao_id: u64, token_slot: u64, amount: i128) {
        env.storage()
            .persistent()
            .set(&DataKey::TreasuryBalance(dao_id, token_slot), &amount);
    }

    pub fn get_locked_balance(env: &Env, payroll_id: u64, token_slot: u64) -> i128 {
        env.storage()
            .persistent()
            .get::<DataKey, i128>(&DataKey::LockedBalance(payroll_id, token_slot))
            .unwrap_or(0)
    }

    pub fn set_locked_balance(env: &Env, payroll_id: u64, token_slot: u64, amount: i128) {
        env.storage()
            .persistent()
            .set(&DataKey::LockedBalance(payroll_id, token_slot), &amount);
    }
}
