use soroban_sdk::{Address, Env, String, Vec};
use crate::types::{DAOConfig, Member, Role};
use crate::storage::Storage;
use crate::errors::ContractError;
use crate::event::Events;

pub struct DAOContract;

impl DAOContract {
    // ─────────────────────────────────────────────────────────────────────────
    // DAO lifecycle
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a new DAO. The creator becomes the first Admin member.
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

        // Register founder as first Admin member
        let founder = Member {
            address: admin.clone(),
            role: Role::Admin,
            added_at: env.ledger().timestamp(),
        };
        Storage::add_member(env, id, &founder);

        Events::dao_created(env, id, &admin);
        Ok(id)
    }

    /// Update mutable DAO settings. Requires Admin role.
    pub fn update_settings(
        env: &Env,
        dao_id: u64,
        admin: Address,
        new_name: Option<String>,
        new_symbol: Option<String>,
        new_threshold: Option<u32>,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        Self::require_role(env, dao_id, &admin, &Role::Admin)?;

        let mut config = Storage::get_dao(env, dao_id)?;
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

    /// Transfer the primary admin address. Requires current Admin role.
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

        config.admin = new_admin.clone();
        Storage::save_dao(env, dao_id, &config);

        // Update role in members map
        let member = Member {
            address: new_admin,
            role: Role::Admin,
            added_at: env.ledger().timestamp(),
        };
        Storage::add_member(env, dao_id, &member);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Members / Roles
    // ─────────────────────────────────────────────────────────────────────────

    /// Add a new member with a given role. Requires Admin.
    pub fn add_member(
        env: &Env,
        dao_id: u64,
        caller: Address,
        new_member: Address,
        role: Role,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        Self::require_role(env, dao_id, &caller, &Role::Admin)?;

        // Prevent adding someone who is already a member
        if Storage::get_member(env, dao_id, &new_member).is_some() {
            return Err(ContractError::AlreadyMember);
        }

        let member = Member {
            address: new_member.clone(),
            role,
            added_at: env.ledger().timestamp(),
        };
        Storage::add_member(env, dao_id, &member);

        let mut config = Storage::get_dao(env, dao_id)?;
        config.total_members += 1;
        Storage::save_dao(env, dao_id, &config);

        Events::member_added(env, dao_id, &new_member);
        Ok(())
    }

    /// Remove a member. Requires Admin. Cannot remove the primary admin.
    pub fn remove_member(
        env: &Env,
        dao_id: u64,
        caller: Address,
        member_addr: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        Self::require_role(env, dao_id, &caller, &Role::Admin)?;

        let config = Storage::get_dao(env, dao_id)?;
        if config.admin == member_addr {
            return Err(ContractError::Unauthorized); // cannot remove primary admin
        }

        if Storage::get_member(env, dao_id, &member_addr).is_none() {
            return Err(ContractError::MemberNotFound);
        }

        Storage::remove_member(env, dao_id, &member_addr);

        let mut config = Storage::get_dao(env, dao_id)?;
        if config.total_members > 1 {
            config.total_members -= 1;
        }
        Storage::save_dao(env, dao_id, &config);

        Events::member_removed(env, dao_id, &member_addr);
        Ok(())
    }

    /// Update a member's role. Requires Admin.
    pub fn update_member_role(
        env: &Env,
        dao_id: u64,
        caller: Address,
        member_addr: Address,
        new_role: Role,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        Self::require_role(env, dao_id, &caller, &Role::Admin)?;

        let mut member = Storage::get_member(env, dao_id, &member_addr)
            .ok_or(ContractError::MemberNotFound)?;
        member.role = new_role;
        Storage::update_member(env, dao_id, &member);
        Ok(())
    }

    /// List all members of a DAO.
    pub fn get_members(env: &Env, dao_id: u64) -> Vec<Member> {
        Storage::get_all_members(env, dao_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Pause / unpause
    // ─────────────────────────────────────────────────────────────────────────

    pub fn pause(env: &Env, dao_id: u64, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();
        Self::require_role(env, dao_id, &admin, &Role::Admin)?;

        let mut config = Storage::get_dao(env, dao_id)?;
        if config.paused {
            return Err(ContractError::DAOPaused);
        }
        config.paused = true;
        Storage::save_dao(env, dao_id, &config);
        Events::dao_paused(env, dao_id, &admin);
        Ok(())
    }

    pub fn unpause(env: &Env, dao_id: u64, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();
        Self::require_role(env, dao_id, &admin, &Role::Admin)?;

        let mut config = Storage::get_dao(env, dao_id)?;
        if !config.paused {
            return Err(ContractError::DAONotPaused);
        }
        config.paused = false;
        Storage::save_dao(env, dao_id, &config);
        Events::dao_unpaused(env, dao_id, &admin);
        Ok(())
    }

    pub fn get_dao_info(env: &Env, dao_id: u64) -> Result<DAOConfig, ContractError> {
        Storage::get_dao(env, dao_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal guard helpers used by all modules
    // ─────────────────────────────────────────────────────────────────────────

    /// Returns `Ok(config)` if the DAO exists and is not paused.
    pub fn require_active(env: &Env, dao_id: u64) -> Result<DAOConfig, ContractError> {
        let config = Storage::get_dao(env, dao_id)?;
        if config.paused {
            return Err(ContractError::DAOPaused);
        }
        Ok(config)
    }

    /// Returns `Ok(config)` if caller is the primary admin and DAO is active.
    pub fn require_admin(env: &Env, dao_id: u64, caller: &Address) -> Result<DAOConfig, ContractError> {
        let config = Self::require_active(env, dao_id)?;
        if config.admin != *caller {
            return Err(ContractError::Unauthorized);
        }
        Ok(config)
    }

    /// Returns `Ok(())` if the caller has at least the required role.
    ///
    /// Role hierarchy: Admin > Treasurer > Viewer
    /// `require_role(env, id, addr, Role::Treasurer)` passes for both
    /// Treasurers and Admins.
    pub fn require_role(
        env: &Env,
        dao_id: u64,
        caller: &Address,
        required: &Role,
    ) -> Result<(), ContractError> {
        // Primary admin always passes
        let config = Storage::get_dao(env, dao_id)?;
        if config.admin == *caller {
            return Ok(());
        }

        let member = Storage::get_member(env, dao_id, caller)
            .ok_or(ContractError::Unauthorized)?;

        let ok = match required {
            Role::Viewer => true, // any role satisfies Viewer
            Role::Treasurer => matches!(member.role, Role::Admin | Role::Treasurer),
            Role::Admin => matches!(member.role, Role::Admin),
        };

        if ok { Ok(()) } else { Err(ContractError::Unauthorized) }
    }
}
