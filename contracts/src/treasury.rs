use soroban_sdk::{token, Address, BytesN, Env};
use crate::storage::Storage;
use crate::errors::ContractError;
use crate::event::Events;
use crate::dao::DAOContract;
use crate::types::Role;

pub struct TreasuryContract;

impl TreasuryContract {
    // ─────────────────────────────────────────────────────────────────────────
    // Token Whitelist management
    // ─────────────────────────────────────────────────────────────────────────

    /// Whitelist a token so it can be deposited into this DAO's treasury.
    /// Requires Admin role.
    pub fn add_token(
        env: &Env,
        dao_id: u64,
        caller: Address,
        token_address: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        DAOContract::require_role(env, dao_id, &caller, &Role::Admin)?;

        let slot = Self::token_slot(env, &token_address);
        Storage::whitelist_token(env, dao_id, slot);
        Events::token_whitelisted(env, dao_id, &token_address);
        Ok(())
    }

    /// Remove a token from the whitelist. Requires Admin role.
    pub fn remove_token(
        env: &Env,
        dao_id: u64,
        caller: Address,
        token_address: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        DAOContract::require_role(env, dao_id, &caller, &Role::Admin)?;

        let slot = Self::token_slot(env, &token_address);
        Storage::remove_whitelisted_token(env, dao_id, slot);
        Events::token_removed(env, dao_id, &token_address);
        Ok(())
    }

    pub fn is_whitelisted(env: &Env, dao_id: u64, token_address: Address) -> bool {
        let slot = Self::token_slot(env, &token_address);
        Storage::is_token_whitelisted(env, dao_id, slot)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Deposits and Withdrawals
    // ─────────────────────────────────────────────────────────────────────────

    /// Deposit tokens into the DAO treasury.
    /// Token must be whitelisted. Any address may deposit (open to contributors).
    pub fn deposit(
        env: &Env,
        dao_id: u64,
        token_address: Address,
        from: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        from.require_auth();
        DAOContract::require_active(env, dao_id)?;

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let slot = Self::token_slot(env, &token_address);
        if !Storage::is_token_whitelisted(env, dao_id, slot) {
            return Err(ContractError::TokenNotWhitelisted);
        }

        token::Client::new(env, &token_address)
            .transfer(&from, &env.current_contract_address(), &amount);

        let balance = Storage::get_treasury_balance(env, dao_id, slot);
        Storage::set_treasury_balance(env, dao_id, slot, balance + amount);

        Events::treasury_deposit(env, dao_id, &token_address, amount);
        Ok(())
    }

    /// Withdraw tokens from the DAO treasury.
    /// Requires Treasurer or Admin role.
    pub fn withdraw(
        env: &Env,
        dao_id: u64,
        token_address: Address,
        caller: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        DAOContract::require_active(env, dao_id)?;
        DAOContract::require_role(env, dao_id, &caller, &Role::Treasurer)?;

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let slot = Self::token_slot(env, &token_address);
        if !Storage::is_token_whitelisted(env, dao_id, slot) {
            return Err(ContractError::TokenNotWhitelisted);
        }

        let balance = Storage::get_treasury_balance(env, dao_id, slot);
        if balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        token::Client::new(env, &token_address)
            .transfer(&env.current_contract_address(), &to, &amount);

        Storage::set_treasury_balance(env, dao_id, slot, balance - amount);
        Events::treasury_withdraw(env, dao_id, &token_address, amount);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Budget locking  (called internally by the payroll module only)
    // ─────────────────────────────────────────────────────────────────────────

    pub fn lock_budget(
        env: &Env,
        dao_id: u64,
        token_address: &Address,
        payroll_id: u64,
        amount: i128,
    ) -> Result<(), ContractError> {
        DAOContract::require_active(env, dao_id)?;

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let slot = Self::token_slot(env, token_address);
        let balance = Storage::get_treasury_balance(env, dao_id, slot);
        if balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        let locked = Storage::get_locked_balance(env, payroll_id, slot);
        Storage::set_treasury_balance(env, dao_id, slot, balance - amount);
        Storage::set_locked_balance(env, payroll_id, slot, locked + amount);

        Events::budget_locked(env, dao_id, payroll_id, amount);
        Ok(())
    }

    /// Release locked budget back to the free treasury balance.
    /// Called when an Approved payroll is cancelled before execution.
    pub fn release_budget(
        env: &Env,
        dao_id: u64,
        token_address: &Address,
        payroll_id: u64,
        amount: i128,
    ) -> Result<(), ContractError> {
        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let slot = Self::token_slot(env, token_address);
        let locked = Storage::get_locked_balance(env, payroll_id, slot);
        if locked < amount {
            return Err(ContractError::InvalidAmount);
        }

        let balance = Storage::get_treasury_balance(env, dao_id, slot);
        Storage::set_locked_balance(env, payroll_id, slot, locked - amount);
        Storage::set_treasury_balance(env, dao_id, slot, balance + amount);

        Events::budget_released(env, dao_id, payroll_id, amount);
        Ok(())
    }

    /// Transfer salary from locked escrow directly to an employee wallet.
    pub fn pay_employee(
        env: &Env,
        payroll_id: u64,
        token_address: &Address,
        employee_wallet: &Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let slot = Self::token_slot(env, token_address);
        let locked = Storage::get_locked_balance(env, payroll_id, slot);
        if locked < amount {
            return Err(ContractError::InsufficientBalance);
        }

        token::Client::new(env, token_address)
            .transfer(&env.current_contract_address(), employee_wallet, &amount);

        Storage::set_locked_balance(env, payroll_id, slot, locked - amount);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Queries
    // ─────────────────────────────────────────────────────────────────────────

    pub fn balance(env: &Env, dao_id: u64, token_address: Address) -> i128 {
        let slot = Self::token_slot(env, &token_address);
        Storage::get_treasury_balance(env, dao_id, slot)
    }

    pub fn locked_balance(env: &Env, payroll_id: u64, token_address: Address) -> i128 {
        let slot = Self::token_slot(env, &token_address);
        Storage::get_locked_balance(env, payroll_id, slot)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Derive a stable u64 slot from a token address via SHA-256 of its XDR.
    pub fn token_slot(env: &Env, token_address: &Address) -> u64 {
        use soroban_sdk::xdr::ToXdr;
        let addr_bytes = token_address.to_xdr(env);
        let hash: soroban_sdk::crypto::Hash<32> = env.crypto().sha256(&addr_bytes);
        let hash_n: BytesN<32> = hash.into();
        let mut slot_bytes = [0u8; 8];
        for i in 0..8u32 {
            slot_bytes[i as usize] = hash_n.get(i).unwrap();
        }
        u64::from_be_bytes(slot_bytes)
    }
}
