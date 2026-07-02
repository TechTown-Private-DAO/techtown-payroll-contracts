use soroban_sdk::{Address, Bytes, BytesN, Env};
use crate::types::{Role, VerifyingKey};
use crate::storage::Storage;
use crate::errors::ContractError;
use crate::event::Events;
use crate::dao::DAOContract;

pub struct UpgradeContract;

impl UpgradeContract {
    // ─────────────────────────────────────────────────────────────────────────
    // ZK Verifying Key
    // ─────────────────────────────────────────────────────────────────────────

    /// Store the Groth16 verifying key for the payroll circuit.
    ///
    /// This is a privileged operation — only an Admin may call it.
    /// In production, gating this behind a multisig proposal is recommended
    /// so no single key can swap the VK unilaterally.
    pub fn set_verifying_key(
        env: &Env,
        dao_id: u64,
        caller: Address,
        vk_bytes: Bytes,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        DAOContract::require_role(env, dao_id, &caller, &Role::Admin)?;

        let vk = VerifyingKey {
            vk_bytes,
            set_at: env.ledger().timestamp(),
        };
        Storage::set_verifying_key(env, &vk);
        Events::verifying_key_set(env, dao_id);
        Ok(())
    }

    /// Retrieve the currently stored verifying key, if any.
    pub fn get_verifying_key(env: &Env) -> Option<VerifyingKey> {
        Storage::get_verifying_key(env)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Contract Upgrade
    // ─────────────────────────────────────────────────────────────────────────

    /// Upgrade the contract WASM.
    ///
    /// Requires Admin role. In production this should additionally require a
    /// multisig proposal to have been Executed (i.e. threshold approvals).
    ///
    /// `new_wasm_hash` is the SHA-256 hash of the new WASM blob, which must
    /// have been uploaded to the ledger beforehand via
    /// `stellar contract upload --wasm <file>`.
    pub fn upgrade_contract(
        env: &Env,
        dao_id: u64,
        caller: Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        DAOContract::require_role(env, dao_id, &caller, &Role::Admin)?;

        env.deployer()
            .update_current_contract_wasm(new_wasm_hash);

        Events::contract_upgraded(env, dao_id);
        Ok(())
    }
}
