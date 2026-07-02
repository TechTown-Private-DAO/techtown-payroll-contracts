use soroban_sdk::{Address, Env, String};
use crate::types::DAOConfig;
use crate::storage::Storage;
use crate::errors::ContractError;
use crate::event::Events;

pub struct DAOContract;

impl DAOContract {
    /// Create a new DAO. Returns the assigned dao_id.
    pub fn create_dao(
        env: &Env,
        admin: Address,
        name: String,
        symbol: String,
        multisig_threshold: u32,
    ) -> Result<u64, ContractError> {
        admin.require_auth();

        if multisig_threshold < 1 {
            return Err(ContractError::InvalidThreshold);
        }

        let id = Storage::next_dao_id(env);
        let config = DAOConfig {
            name,
            symbol,
            admin: admin.clone(),
            multisig_threshold,
            total_members: 1,
            paused: false,
        };

        Storage::save_dao(env, id, &config);
        Events::dao_created(env, id, &admin);
        Ok(id)
    }

    /// Update mutable settings of a DAO. Only the current admin may call this.
    pub fn update_settings(
        env: &Env,
        dao_id: u64,
        admin: Address,
        new_name: Option<String>,
        new_symbol: Option<String>,
        new_threshold: Option<u32>,
    ) -> Result<(), ContractError> {
        admin.require_auth();

        let mut config = Storage::get_dao(env, dao_id)?;

        if config.admin != admin {
            return Err(ContractError::Unauthorized);
        }

        if config.paused {
            return Err(ContractError::DAOPaused);
        }

        if let Some(name) = new_name {
            config.name = name;
        }
        if let Some(symbol) = new_symbol {
            config.symbol = symbol;
        }
        if let Some(threshold) = new_threshold {
            if threshold < 1 {
                return Err(ContractError::InvalidThreshold);
            }
            config.multisig_threshold = threshold;
        }

        Storage::save_dao(env, dao_id, &config);
        Ok(())
    }

    /// Transfer admin role to a new address.
    pub fn transfer_admin(
        env: &Env,
        dao_id: u64,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), ContractError> {
        current_admin.require_auth();

        let mut config = Storage::get_dao(env, dao_id)?;

        if config.admin != current_admin {
            return Err(ContractError::Unauthorized);
        }

        config.admin = new_admin;
        Storage::save_dao(env, dao_id, &config);
        Ok(())
    }

    /// Pause all operations for the DAO.
    pub fn pause(
        env: &Env,
        dao_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();

        let mut config = Storage::get_dao(env, dao_id)?;

        if config.admin != admin {
            return Err(ContractError::Unauthorized);
        }

        if config.paused {
            return Err(ContractError::DAOPaused);
        }

        config.paused = true;
        Storage::save_dao(env, dao_id, &config);
        Events::dao_paused(env, dao_id, &admin);
        Ok(())
    }

    /// Resume operations for the DAO.
    pub fn unpause(
        env: &Env,
        dao_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();

        let mut config = Storage::get_dao(env, dao_id)?;

        if config.admin != admin {
            return Err(ContractError::Unauthorized);
        }

        if !config.paused {
            return Err(ContractError::DAONotPaused);
        }

        config.paused = false;
        Storage::save_dao(env, dao_id, &config);
        Events::dao_unpaused(env, dao_id, &admin);
        Ok(())
    }

    /// Fetch DAO configuration.
    pub fn get_dao_info(env: &Env, dao_id: u64) -> Result<DAOConfig, ContractError> {
        Storage::get_dao(env, dao_id)
    }

    // ── Internal guard helpers used by other modules ──────────────────────────

    /// Returns Ok(config) if the DAO exists and is not paused.
    pub fn require_active(env: &Env, dao_id: u64) -> Result<DAOConfig, ContractError> {
        let config = Storage::get_dao(env, dao_id)?;
        if config.paused {
            return Err(ContractError::DAOPaused);
        }
        Ok(config)
    }

    /// Returns Ok(config) if the DAO exists, is not paused, and `caller` is the admin.
    pub fn require_admin(env: &Env, dao_id: u64, caller: &Address) -> Result<DAOConfig, ContractError> {
        let config = Self::require_active(env, dao_id)?;
        if config.admin != *caller {
            return Err(ContractError::Unauthorized);
        }
        Ok(config)
    }
}
