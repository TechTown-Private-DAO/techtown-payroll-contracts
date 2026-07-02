use soroban_sdk::{contracttype, Address, Env, Vec};
use crate::types::*;
use crate::errors::ContractError;

// ─────────────────────────────────────────────────────────────────────────────
// Storage Keys
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    // ── Counters ─────────────────────────────────────────────────────────────
    DaoCounter,
    EmployeeCounter,
    PayrollCounter,
    ProposalCounter,

    // ── Core entities ────────────────────────────────────────────────────────
    Dao(u64),
    Employee(u64, u64),         // (dao_id, employee_id)
    Payroll(u64),               // payroll_id
    Commitment(u64, u64),       // (dao_id, employee_id)
    Proposal(u64, u64),         // (dao_id, proposal_id)

    // ── Roles / members ──────────────────────────────────────────────────────
    Member(u64, u64),           // (dao_id, member_index) → Member
    MemberIndex(u64),           // dao_id → member count (for iteration)
    MemberByAddr(u64, Address), // (dao_id, address) → member_index  (reverse lookup)

    // ── Token whitelist ──────────────────────────────────────────────────────
    /// (dao_id, token_slot) → bool — true means the token is whitelisted
    TokenWhitelisted(u64, u64),
    /// dao_id → count of whitelisted tokens
    TokenCount(u64),

    // ── Treasury balances ────────────────────────────────────────────────────
    /// (dao_id, token_slot) → i128 available balance
    TreasuryBalance(u64, u64),
    /// (payroll_id, token_slot) → i128 locked balance
    LockedBalance(u64, u64),

    // ── Period double-pay guard ──────────────────────────────────────────────
    /// (dao_id, employee_id, period) → bool — true = already paid this period
    PeriodPaid(u64, u64, u64),

    // ── Per-payroll claim tracking ───────────────────────────────────────────
    /// (payroll_id, employee_id) → bool
    Claimed(u64, u64),

    // ── Employee count per DAO (iteration) ───────────────────────────────────
    DaoEmployeeCount(u64),

    // ── ZK verifying key ─────────────────────────────────────────────────────
    VerifyingKey,
}

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

    // ── Roles / Members ──────────────────────────────────────────────────────

    pub fn add_member(env: &Env, dao_id: u64, member: &Member) {
        let idx_key = DataKey::MemberIndex(dao_id);
        let idx: u64 = env.storage().persistent().get(&idx_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Member(dao_id, idx), member);
        env.storage()
            .persistent()
            .set(&DataKey::MemberByAddr(dao_id, member.address.clone()), &idx);
        env.storage().persistent().set(&idx_key, &(idx + 1));
    }

    pub fn get_member(env: &Env, dao_id: u64, addr: &Address) -> Option<Member> {
        let idx: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::MemberByAddr(dao_id, addr.clone()))?;
        env.storage()
            .persistent()
            .get(&DataKey::Member(dao_id, idx))
    }

    pub fn update_member(env: &Env, dao_id: u64, member: &Member) {
        if let Some(idx) = env
            .storage()
            .persistent()
            .get::<DataKey, u64>(&DataKey::MemberByAddr(dao_id, member.address.clone()))
        {
            env.storage()
                .persistent()
                .set(&DataKey::Member(dao_id, idx), member);
        }
    }

    pub fn remove_member(env: &Env, dao_id: u64, addr: &Address) {
        if let Some(idx) = env
            .storage()
            .persistent()
            .get::<DataKey, u64>(&DataKey::MemberByAddr(dao_id, addr.clone()))
        {
            env.storage()
                .persistent()
                .remove(&DataKey::Member(dao_id, idx));
            env.storage()
                .persistent()
                .remove(&DataKey::MemberByAddr(dao_id, addr.clone()));
        }
    }

    pub fn get_all_members(env: &Env, dao_id: u64) -> Vec<Member> {
        let mut members = Vec::new(env);
        let max: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::MemberIndex(dao_id))
            .unwrap_or(0);
        for i in 0..max {
            if let Some(m) = env
                .storage()
                .persistent()
                .get::<DataKey, Member>(&DataKey::Member(dao_id, i))
            {
                members.push_back(m);
            }
        }
        members
    }

    // ── Token Whitelist ───────────────────────────────────────────────────────

    pub fn whitelist_token(env: &Env, dao_id: u64, token_slot: u64) {
        env.storage()
            .persistent()
            .set(&DataKey::TokenWhitelisted(dao_id, token_slot), &true);
    }

    pub fn remove_whitelisted_token(env: &Env, dao_id: u64, token_slot: u64) {
        env.storage()
            .persistent()
            .remove(&DataKey::TokenWhitelisted(dao_id, token_slot));
    }

    pub fn is_token_whitelisted(env: &Env, dao_id: u64, token_slot: u64) -> bool {
        env.storage()
            .persistent()
            .get::<DataKey, bool>(&DataKey::TokenWhitelisted(dao_id, token_slot))
            .unwrap_or(false)
    }

    // ── Employee ─────────────────────────────────────────────────────────────

    pub fn save_employee(env: &Env, dao_id: u64, employee: &Employee) {
        env.storage()
            .persistent()
            .set(&DataKey::Employee(dao_id, employee.id), employee);
        let count_key = DataKey::DaoEmployeeCount(dao_id);
        let current: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);
        if employee.id >= current {
            env.storage()
                .persistent()
                .set(&count_key, &(employee.id + 1));
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
        let max: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::DaoEmployeeCount(dao_id))
            .unwrap_or(0);
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

    // ── Period double-pay guard ───────────────────────────────────────────────

    pub fn mark_period_paid(env: &Env, dao_id: u64, employee_id: u64, period: u64) {
        env.storage()
            .persistent()
            .set(&DataKey::PeriodPaid(dao_id, employee_id, period), &true);
    }

    pub fn is_period_paid(env: &Env, dao_id: u64, employee_id: u64, period: u64) -> bool {
        env.storage()
            .persistent()
            .get::<DataKey, bool>(&DataKey::PeriodPaid(dao_id, employee_id, period))
            .unwrap_or(false)
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

    // ── Treasury balances ─────────────────────────────────────────────────────

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

    // ── ZK Verifying Key ─────────────────────────────────────────────────────

    pub fn set_verifying_key(env: &Env, vk: &VerifyingKey) {
        env.storage()
            .persistent()
            .set(&DataKey::VerifyingKey, vk);
    }

    pub fn get_verifying_key(env: &Env) -> Option<VerifyingKey> {
        env.storage()
            .persistent()
            .get(&DataKey::VerifyingKey)
    }
}
