#![cfg(test)]

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, Bytes, BytesN, Env, String, Vec,
};

use crate::{
    EmployeeStatus, Member, PayrollStatus, ProposalStatus, Role,
    SalaryCommitment, TechTownPayroll, TechTownPayrollClient, ZKProof,
};

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

fn setup() -> (Env, Address) {
    let env = Env::default();
    let addr = env.register(TechTownPayroll, ());
    (env, addr)
}

fn client<'a>(env: &'a Env, addr: &Address) -> TechTownPayrollClient<'a> {
    TechTownPayrollClient::new(env, addr)
}

fn rand_hash(env: &Env) -> BytesN<32> {
    BytesN::random(env)
}

/// Create a DAO and return (dao_id).
fn make_dao(env: &Env, c: &TechTownPayrollClient, admin: &Address, threshold: u32) -> u64 {
    c.create_dao(
        admin,
        &String::from_str(env, "TechTown"),
        &String::from_str(env, "TT"),
        &threshold,
    )
}

/// Add an employee and return employee_id.
fn make_employee(
    env: &Env,
    c: &TechTownPayrollClient,
    dao_id: u64,
    admin: &Address,
    wallet: &Address,
) -> u64 {
    c.add_employee(
        &dao_id,
        admin,
        wallet,
        &String::from_str(env, "Engineering"),
        &rand_hash(env),
        &1u64,
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// DAO tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_create_dao() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 2);
    assert_eq!(dao_id, 1u64);

    let cfg = c.get_dao(&dao_id);
    assert_eq!(cfg.admin, admin);
    assert!(!cfg.paused);
    assert_eq!(cfg.multisig_threshold, 2u32);
    assert_eq!(cfg.total_members, 1u32);
}

#[test]
#[should_panic]
fn test_zero_threshold_panics() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    make_dao(&env, &c, &admin, 0);
}

#[test]
fn test_pause_unpause() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    c.pause_dao(&dao_id, &admin);
    assert!(c.get_dao(&dao_id).paused);
    c.unpause_dao(&dao_id, &admin);
    assert!(!c.get_dao(&dao_id).paused);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_pause() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let rando = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    c.pause_dao(&dao_id, &rando); // should panic — Unauthorized
}

#[test]
fn test_update_settings() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 2);
    c.update_dao_settings(
        &dao_id,
        &admin,
        &Some(String::from_str(&env, "NewName")),
        &None,
        &Some(3u32),
    );
    let cfg = c.get_dao(&dao_id);
    assert_eq!(cfg.name, String::from_str(&env, "NewName"));
    assert_eq!(cfg.multisig_threshold, 3u32);
}

// ─────────────────────────────────────────────────────────────────────────────
// Roles / Members tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_add_member_treasurer() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let treasurer = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    c.add_member(&dao_id, &admin, &treasurer, &Role::Treasurer);

    let members = c.get_members(&dao_id);
    // founder (Admin) + new Treasurer = 2
    assert_eq!(members.len(), 2u32);

    let has_treasurer = members.iter().any(|m: Member| m.address == treasurer && m.role == Role::Treasurer);
    assert!(has_treasurer);
}

#[test]
fn test_add_member_viewer() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let viewer = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    c.add_member(&dao_id, &admin, &viewer, &Role::Viewer);

    let members = c.get_members(&dao_id);
    assert_eq!(members.len(), 2u32);
}

#[test]
#[should_panic]
fn test_duplicate_member_panics() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    c.add_member(&dao_id, &admin, &member, &Role::Viewer);
    c.add_member(&dao_id, &admin, &member, &Role::Viewer); // AlreadyMember
}

#[test]
fn test_remove_member() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    c.add_member(&dao_id, &admin, &member, &Role::Viewer);
    assert_eq!(c.get_members(&dao_id).len(), 2u32);

    c.remove_member(&dao_id, &admin, &member);
    // removed member is gone from the map; active count reflects removal
    let members = c.get_members(&dao_id);
    let still_there = members.iter().any(|m: Member| m.address == member);
    assert!(!still_there);
}

#[test]
fn test_update_member_role() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    c.add_member(&dao_id, &admin, &member, &Role::Viewer);
    c.update_member_role(&dao_id, &admin, &member, &Role::Treasurer);

    let members = c.get_members(&dao_id);
    let updated = members.iter().find(|m: &Member| m.address == member).unwrap();
    assert_eq!(updated.role, Role::Treasurer);
}

#[test]
#[should_panic]
fn test_treasurer_cannot_add_member() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let treasurer = Address::generate(&env);
    let newcomer = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    c.add_member(&dao_id, &admin, &treasurer, &Role::Treasurer);
    // Treasurer cannot add members — Unauthorized
    c.add_member(&dao_id, &treasurer, &newcomer, &Role::Viewer);
}

// ─────────────────────────────────────────────────────────────────────────────
// Employee tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_add_get_employee() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    let emp_id = make_employee(&env, &c, dao_id, &admin, &wallet);

    let emp = c.get_employee(&dao_id, &emp_id);
    assert_eq!(emp.wallet, wallet);
    assert_eq!(emp.status, EmployeeStatus::Active);
    assert_eq!(emp.last_paid_period, 0u64);
}

#[test]
fn test_freeze_activate_employee() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    let emp_id = make_employee(&env, &c, dao_id, &admin, &wallet);

    c.freeze_employee(&dao_id, &emp_id, &admin);
    assert_eq!(c.get_employee(&dao_id, &emp_id).status, EmployeeStatus::Frozen);

    c.activate_employee(&dao_id, &emp_id, &admin);
    assert_eq!(c.get_employee(&dao_id, &emp_id).status, EmployeeStatus::Active);
}

#[test]
fn test_remove_employee() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    let emp_id = make_employee(&env, &c, dao_id, &admin, &wallet);

    c.remove_employee(&dao_id, &emp_id, &admin);
    assert_eq!(c.get_employee(&dao_id, &emp_id).status, EmployeeStatus::Removed);
}

#[test]
fn test_get_all_employees() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    for _ in 0..3 {
        make_employee(&env, &c, dao_id, &admin, &Address::generate(&env));
    }
    assert_eq!(c.get_all_employees(&dao_id).len(), 3u32);
}

// ─────────────────────────────────────────────────────────────────────────────
// Token whitelist tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_add_and_check_token() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    assert!(!c.is_token_whitelisted(&dao_id, &token));

    c.add_token(&dao_id, &admin, &token);
    assert!(c.is_token_whitelisted(&dao_id, &token));
}

#[test]
fn test_remove_token() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    c.add_token(&dao_id, &admin, &token);
    assert!(c.is_token_whitelisted(&dao_id, &token));

    c.remove_token(&dao_id, &admin, &token);
    assert!(!c.is_token_whitelisted(&dao_id, &token));
}

#[test]
#[should_panic]
fn test_treasurer_cannot_whitelist_token() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let treasurer = Address::generate(&env);
    let token = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    c.add_member(&dao_id, &admin, &treasurer, &Role::Treasurer);
    // Treasurer role is below Admin — should panic
    c.add_token(&dao_id, &treasurer, &token);
}

// ─────────────────────────────────────────────────────────────────────────────
// Payroll helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Compute the same claim leaf the contract uses: SHA-256(employee_id_be8 ‖ amount_be16)
fn claim_leaf(env: &Env, employee_id: u64, amount: i128) -> BytesN<32> {
    let mut pre = Bytes::new(env);
    for b in employee_id.to_be_bytes().iter() { pre.push_back(*b); }
    for b in amount.to_be_bytes().iter() { pre.push_back(*b); }
    env.crypto().sha256(&pre).into()
}

/// Build a structurally valid ZKProof accepted by ZKVerifier::verify_payroll_proof.
fn make_proof(env: &Env, total: i128, count: u32, root: &BytesN<32>) -> ZKProof {
    let mut proof_bytes = Bytes::new(env);
    for _ in 0..256u32 { proof_bytes.push_back(0xAB); }

    let mut a0 = Bytes::new(env);
    for b in total.to_be_bytes().iter() { a0.push_back(*b); }

    let mut a1 = Bytes::new(env);
    for b in count.to_be_bytes().iter() { a1.push_back(*b); }

    let mut a2 = Bytes::new(env);
    for i in 0..32u32 { a2.push_back(root.get(i).unwrap()); }

    let mut inputs: Vec<Bytes> = Vec::new(env);
    inputs.push_back(a0);
    inputs.push_back(a1);
    inputs.push_back(a2);

    ZKProof { proof: proof_bytes, public_inputs: inputs }
}

// ─────────────────────────────────────────────────────────────────────────────
// Payroll tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_create_and_get_payroll() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    let token = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    let emp_id = make_employee(&env, &c, dao_id, &admin, &wallet);
    c.add_token(&dao_id, &admin, &token);

    let total = 1000i128;
    let root = claim_leaf(&env, emp_id, total);

    let commitment = SalaryCommitment {
        employee_id: emp_id,
        commitment_hash: rand_hash(&env),
        period: 1,
        created_at: env.ledger().timestamp(),
    };
    let mut emps: Vec<u64> = Vec::new(&env); emps.push_back(emp_id);
    let mut commits: Vec<SalaryCommitment> = Vec::new(&env); commits.push_back(commitment);

    let pid = c.create_payroll(&dao_id, &admin, &1u64, &emps, &commits, &total, &root, &token);
    let p = c.get_payroll(&pid);
    assert_eq!(p.status, PayrollStatus::Pending);
    assert_eq!(p.total_amount, total);
}

#[test]
fn test_approve_payroll() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    let token = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    let emp_id = make_employee(&env, &c, dao_id, &admin, &wallet);
    c.add_token(&dao_id, &admin, &token);

    let total = 500i128;
    let root = claim_leaf(&env, emp_id, total);
    let commitment = SalaryCommitment {
        employee_id: emp_id, commitment_hash: rand_hash(&env), period: 1,
        created_at: env.ledger().timestamp(),
    };
    let mut emps: Vec<u64> = Vec::new(&env); emps.push_back(emp_id);
    let mut commits: Vec<SalaryCommitment> = Vec::new(&env); commits.push_back(commitment);

    let pid = c.create_payroll(&dao_id, &admin, &1u64, &emps, &commits, &total, &root, &token);
    c.approve_payroll(&pid, &dao_id, &admin);
    assert_eq!(c.get_payroll(&pid).status, PayrollStatus::Approved);
}

#[test]
fn test_cancel_pending_payroll() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    let token = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    let emp_id = make_employee(&env, &c, dao_id, &admin, &wallet);
    c.add_token(&dao_id, &admin, &token);

    let total = 200i128;
    let root = claim_leaf(&env, emp_id, total);
    let commitment = SalaryCommitment {
        employee_id: emp_id, commitment_hash: rand_hash(&env), period: 1,
        created_at: env.ledger().timestamp(),
    };
    let mut emps: Vec<u64> = Vec::new(&env); emps.push_back(emp_id);
    let mut commits: Vec<SalaryCommitment> = Vec::new(&env); commits.push_back(commitment);

    let pid = c.create_payroll(&dao_id, &admin, &1u64, &emps, &commits, &total, &root, &token);
    c.cancel_payroll(&pid, &dao_id, &admin, &token);
    assert_eq!(c.get_payroll(&pid).status, PayrollStatus::Cancelled);
}

#[test]
#[should_panic]
fn test_double_cancel_panics() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    let token = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    let emp_id = make_employee(&env, &c, dao_id, &admin, &wallet);
    c.add_token(&dao_id, &admin, &token);

    let total = 100i128;
    let root = claim_leaf(&env, emp_id, total);
    let commitment = SalaryCommitment {
        employee_id: emp_id, commitment_hash: rand_hash(&env), period: 1,
        created_at: env.ledger().timestamp(),
    };
    let mut emps: Vec<u64> = Vec::new(&env); emps.push_back(emp_id);
    let mut commits: Vec<SalaryCommitment> = Vec::new(&env); commits.push_back(commitment);

    let pid = c.create_payroll(&dao_id, &admin, &1u64, &emps, &commits, &total, &root, &token);
    c.cancel_payroll(&pid, &dao_id, &admin, &token);
    c.cancel_payroll(&pid, &dao_id, &admin, &token); // panics
}

// ─────────────────────────────────────────────────────────────────────────────
// Period double-pay guard
// The guard fires at employee_claim time (when Storage::mark_period_paid is set).
// A second payroll for the same period can be created, but claiming twice fails.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_different_periods_allowed() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    let token = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    let emp_id = make_employee(&env, &c, dao_id, &admin, &wallet);
    c.add_token(&dao_id, &admin, &token);

    // Period 1 payroll
    let commitment1 = SalaryCommitment {
        employee_id: emp_id, commitment_hash: rand_hash(&env),
        period: 1, created_at: env.ledger().timestamp(),
    };
    let mut emps: Vec<u64> = Vec::new(&env); emps.push_back(emp_id);
    let mut commits1: Vec<SalaryCommitment> = Vec::new(&env); commits1.push_back(commitment1);
    let root1 = claim_leaf(&env, emp_id, 500i128);
    let pid1 = c.create_payroll(&dao_id, &admin, &1u64, &emps, &commits1, &500i128, &root1, &token);

    // Period 2 payroll — different period, should succeed
    let commitment2 = SalaryCommitment {
        employee_id: emp_id, commitment_hash: rand_hash(&env),
        period: 2, created_at: env.ledger().timestamp(),
    };
    let mut commits2: Vec<SalaryCommitment> = Vec::new(&env); commits2.push_back(commitment2);
    let root2 = claim_leaf(&env, emp_id, 500i128);
    let pid2 = c.create_payroll(&dao_id, &admin, &2u64, &emps, &commits2, &500i128, &root2, &token);

    // Both payrolls exist and are Pending
    assert_eq!(c.get_payroll(&pid1).status, PayrollStatus::Pending);
    assert_eq!(c.get_payroll(&pid2).status, PayrollStatus::Pending);
}

// ─────────────────────────────────────────────────────────────────────────────
// Multisig tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_proposal_threshold_1_auto_executes() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    let target = Address::generate(&env);

    let pid = c.create_proposal(
        &dao_id, &admin, &target,
        &String::from_str(&env, "transfer"),
        &Bytes::new(&env),
    );
    assert_eq!(c.get_proposal(&dao_id, &pid).status, ProposalStatus::Executed);
}

#[test]
fn test_proposal_executes_at_threshold() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let a2 = Address::generate(&env);
    let a3 = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 3);
    let target = Address::generate(&env);

    let pid = c.create_proposal(
        &dao_id, &admin, &target,
        &String::from_str(&env, "do_thing"),
        &Bytes::new(&env),
    );
    assert_eq!(c.get_proposal(&dao_id, &pid).status, ProposalStatus::Active);

    c.approve_proposal(&dao_id, &pid, &a2);
    assert_eq!(c.get_proposal(&dao_id, &pid).status, ProposalStatus::Active);

    c.approve_proposal(&dao_id, &pid, &a3);
    assert_eq!(c.get_proposal(&dao_id, &pid).status, ProposalStatus::Executed);
}

#[test]
fn test_reject_proposal() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 3);
    let target = Address::generate(&env);

    let pid = c.create_proposal(
        &dao_id, &proposer, &target,
        &String::from_str(&env, "transfer"),
        &Bytes::new(&env),
    );
    c.reject_proposal(&dao_id, &pid, &admin);
    assert_eq!(c.get_proposal(&dao_id, &pid).status, ProposalStatus::Rejected);
}

#[test]
#[should_panic]
fn test_duplicate_approval_panics() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 3);
    let target = Address::generate(&env);

    let pid = c.create_proposal(
        &dao_id, &admin, &target,
        &String::from_str(&env, "transfer"),
        &Bytes::new(&env),
    );
    c.approve_proposal(&dao_id, &pid, &admin); // admin already approved at creation
}

// ─────────────────────────────────────────────────────────────────────────────
// ZK commitment helper
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_compute_commitment_deterministic() {
    let (env, addr) = setup();
    let c = client(&env, &addr);

    let mut rand = Bytes::new(&env);
    rand.push_back(0xDE); rand.push_back(0xAD);
    rand.push_back(0xBE); rand.push_back(0xEF);

    let h1 = c.compute_commitment(&1u64, &50000i128, &rand);
    let h2 = c.compute_commitment(&1u64, &50000i128, &rand);
    assert_eq!(h1, h2);

    let h3 = c.compute_commitment(&2u64, &50000i128, &rand);
    assert_ne!(h1, h3);
}

// ─────────────────────────────────────────────────────────────────────────────
// Verifying key
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_set_and_get_verifying_key() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    assert!(c.get_verifying_key().is_none());

    let mut vk = Bytes::new(&env);
    for i in 0..64u32 { vk.push_back(i as u8); }

    c.set_verifying_key(&dao_id, &admin, &vk);
    let stored = c.get_verifying_key().unwrap();
    assert_eq!(stored.vk_bytes, vk);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_set_vk() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let rando = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = make_dao(&env, &c, &admin, 1);
    let mut vk = Bytes::new(&env);
    vk.push_back(0xFF);
    c.set_verifying_key(&dao_id, &rando, &vk); // Unauthorized
}
