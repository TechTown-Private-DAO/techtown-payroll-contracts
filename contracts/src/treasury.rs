use soroban_sdk::{token, Address, BytesN, Env};
use crate::storage::Storage;
use crate::errors::ContractError;
use crate::event::Events;
use crate::dao::DAOContract;

pub struct TreasuryContract;

impl TreasuryContract {
    /// Deposit tokens into the DAO treasury.
    ///
    /// The caller must have pre-authorised the transfer (standard SEP-41 flow).
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

        // Transfer from caller → contract
        token::Client::new(env, &token_address)
            .transfer(&from, &env.current_contract_address(), &amount);

        // Update internal book-keeping
        let slot = Self::token_slot(env, &token_address);
        let balance = Storage::get_treasury_balance(env, dao_id, slot);
        Storage::set_treasury_balance(env, dao_id, slot, balance + amount);

        Events::treasury_deposit(env, dao_id, &token_address, amount);
        Ok(())
    }

    /// Withdraw tokens from the DAO treasury. Only the DAO admin may call.
    pub fn withdraw(
        env: &Env,
        dao_id: u64,
        token_address: Address,
        admin: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        DAOContract::require_admin(env, dao_id, &admin)?;

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let slot = Self::token_slot(env, &token_address);
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

    /// Lock a portion of the treasury into a payroll escrow.
    ///
    /// Called internally during payroll execution. Not exposed publicly to
    /// prevent arbitrary locking — only the payroll module calls this.
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

    /// Release locked budget back to the treasury (used when payroll is cancelled).
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

    /// Pay an employee directly from locked payroll funds.
    ///
    /// Called once per employee during payroll execution or claim.
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

    /// Available treasury balance for a given token.
    pub fn balance(env: &Env, dao_id: u64, token_address: Address) -> i128 {
        let slot = Self::token_slot(env, &token_address);
        Storage::get_treasury_balance(env, dao_id, slot)
    }

    /// Locked balance for a specific payroll.
    pub fn locked_balance(env: &Env, payroll_id: u64, token_address: Address) -> i128 {
        let slot = Self::token_slot(env, &token_address);
        Storage::get_locked_balance(env, payroll_id, slot)
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Derive a stable u64 "slot" from a token address so we can use it as
    /// a DataKey component without needing Address in the enum variant.
    ///
    /// We use the first 8 bytes of SHA-256(address XDR bytes) for determinism.
    /// Collision probability across distinct token addresses is negligible.
    pub fn token_slot(env: &Env, token_address: &Address) -> u64 {
        use soroban_sdk::xdr::ToXdr;
        let addr_bytes = token_address.to_xdr(env);
        let hash: soroban_sdk::crypto::Hash<32> = env.crypto().sha256(&addr_bytes);
        let hash_bytes_n: BytesN<32> = hash.into();
        let mut slot_bytes = [0u8; 8];
        for i in 0..8u32 {
            slot_bytes[i as usize] = hash_bytes_n.get(i).unwrap();
        }
        u64::from_be_bytes(slot_bytes)
    }
}
