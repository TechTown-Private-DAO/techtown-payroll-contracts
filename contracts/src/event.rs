use soroban_sdk::{symbol_short, Address, Env};

pub struct Events;

impl Events {
    // ── DAO ──────────────────────────────────────────────────────────────────

    pub fn dao_created(env: &Env, dao_id: u64, admin: &Address) {
        env.events().publish((symbol_short!("dao_creat"), dao_id), admin);
    }

    pub fn dao_paused(env: &Env, dao_id: u64, admin: &Address) {
        env.events().publish((symbol_short!("dao_pause"), dao_id), admin);
    }

    pub fn dao_unpaused(env: &Env, dao_id: u64, admin: &Address) {
        env.events().publish((symbol_short!("dao_unpse"), dao_id), admin);
    }

    // ── Members / Roles ──────────────────────────────────────────────────────

    pub fn member_added(env: &Env, dao_id: u64, member: &Address) {
        env.events().publish((symbol_short!("mbr_added"), dao_id), member);
    }

    pub fn member_removed(env: &Env, dao_id: u64, member: &Address) {
        env.events().publish((symbol_short!("mbr_rmvd"), dao_id), member);
    }

    pub fn role_updated(env: &Env, dao_id: u64, member: &Address) {
        env.events().publish((symbol_short!("rol_updt"), dao_id), member);
    }

    // ── Token Whitelist ───────────────────────────────────────────────────────

    pub fn token_whitelisted(env: &Env, dao_id: u64, token: &Address) {
        env.events().publish((symbol_short!("tok_whtl"), dao_id), token);
    }

    pub fn token_removed(env: &Env, dao_id: u64, token: &Address) {
        env.events().publish((symbol_short!("tok_rmvd"), dao_id), token);
    }

    // ── Employee ─────────────────────────────────────────────────────────────

    pub fn employee_added(env: &Env, dao_id: u64, employee_id: u64, wallet: &Address) {
        env.events()
            .publish((symbol_short!("emp_added"), dao_id), (employee_id, wallet));
    }

    pub fn employee_removed(env: &Env, dao_id: u64, employee_id: u64) {
        env.events().publish((symbol_short!("emp_rmvd"), dao_id), employee_id);
    }

    pub fn employee_frozen(env: &Env, dao_id: u64, employee_id: u64) {
        env.events().publish((symbol_short!("emp_frzn"), dao_id), employee_id);
    }

    pub fn employee_activated(env: &Env, dao_id: u64, employee_id: u64) {
        env.events().publish((symbol_short!("emp_actv"), dao_id), employee_id);
    }

    pub fn wallet_updated(env: &Env, dao_id: u64, employee_id: u64, new_wallet: &Address) {
        env.events()
            .publish((symbol_short!("wlt_updt"), dao_id), (employee_id, new_wallet));
    }

    // ── Treasury ─────────────────────────────────────────────────────────────

    pub fn treasury_deposit(env: &Env, dao_id: u64, token: &Address, amount: i128) {
        env.events()
            .publish((symbol_short!("trs_depo"), dao_id), (token, amount));
    }

    pub fn treasury_withdraw(env: &Env, dao_id: u64, token: &Address, amount: i128) {
        env.events()
            .publish((symbol_short!("trs_wdrl"), dao_id), (token, amount));
    }

    pub fn budget_locked(env: &Env, dao_id: u64, payroll_id: u64, amount: i128) {
        env.events()
            .publish((symbol_short!("bdg_lock"), dao_id), (payroll_id, amount));
    }

    pub fn budget_released(env: &Env, dao_id: u64, payroll_id: u64, amount: i128) {
        env.events()
            .publish((symbol_short!("bdg_rels"), dao_id), (payroll_id, amount));
    }

    // ── Payroll ──────────────────────────────────────────────────────────────

    pub fn payroll_created(env: &Env, payroll_id: u64, dao_id: u64, total_amount: i128) {
        env.events()
            .publish((symbol_short!("pay_crtd"), payroll_id), (dao_id, total_amount));
    }

    pub fn payroll_approved(env: &Env, payroll_id: u64, approver: &Address) {
        env.events()
            .publish((symbol_short!("pay_appd"), payroll_id), approver);
    }

    pub fn payroll_executed(env: &Env, payroll_id: u64, total_amount: i128) {
        env.events()
            .publish((symbol_short!("pay_exec"), payroll_id), total_amount);
    }

    pub fn payroll_cancelled(env: &Env, payroll_id: u64) {
        env.events().publish((symbol_short!("pay_cncl"), payroll_id), ());
    }

    pub fn salary_claimed(env: &Env, payroll_id: u64, employee_id: u64, amount: i128) {
        env.events()
            .publish((symbol_short!("sal_clmd"), payroll_id), (employee_id, amount));
    }

    // ── Multisig ─────────────────────────────────────────────────────────────

    pub fn proposal_created(env: &Env, proposal_id: u64, dao_id: u64, proposer: &Address) {
        env.events()
            .publish((symbol_short!("prp_crtd"), proposal_id), (dao_id, proposer));
    }

    pub fn proposal_approved(env: &Env, proposal_id: u64, approver: &Address) {
        env.events()
            .publish((symbol_short!("prp_appd"), proposal_id), approver);
    }

    pub fn proposal_executed(env: &Env, proposal_id: u64) {
        env.events().publish((symbol_short!("prp_exec"), proposal_id), ());
    }

    pub fn proposal_rejected(env: &Env, proposal_id: u64, admin: &Address) {
        env.events()
            .publish((symbol_short!("prp_rjct"), proposal_id), admin);
    }

    // ── Upgradeability ────────────────────────────────────────────────────────

    pub fn contract_upgraded(env: &Env, dao_id: u64) {
        env.events().publish((symbol_short!("upgraded"), dao_id), ());
    }

    pub fn verifying_key_set(env: &Env, dao_id: u64) {
        env.events().publish((symbol_short!("vk_set"), dao_id), ());
    }
}
