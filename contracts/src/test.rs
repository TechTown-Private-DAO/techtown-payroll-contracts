#![cfg(test)]

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, Bytes, BytesN, Env, String, Vec,
};

use crate::{
    EmployeeStatus, PayrollStatus, ProposalStatus,
    SalaryCommitment, TechTownPayroll, TechTownPayrollClient, ZKProof,
};

// ─────────────────────────────────────────────────────────────────────────────
// Test Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn setup() -> (Env, Address) {
    let env = Env::default();
    // register returns the contract address in SDK 22
    let addr = env.register(TechTownPayroll, ());
    (env, addr)
}

fn client<'a>(env: &'a Env, addr: &Address) -> TechTownPayrollClient<'a> {
    TechTownPayrollClient::new(env, addr)
}

fn random_hash(env: &Env) -> BytesN<32> {
    BytesN::random(env)
}

/// Build a structurally valid ZKProof that passes ZKVerifier::verify_payroll_proof.
fn make_proof(env: &Env, total_amount: i128, employee_count: u32, merkle_root: &BytesN<32>) -> ZKProof {
    // Non-empty proof bytes
    let mut proof_bytes = Bytes::new(env);
    for _ in 0..256u32 {
        proof_bytes.push_back(0xAB);
    }

    // input[0]: total_amount as 16-byte big-endian
    let mut a0 = Bytes::new(env);
    for b in total_amount.to_be_bytes().iter() {
        a0.push_back(*b);
    }

    // input[1]: employee_count as 4-byte big-endian
    let mut a1 = Bytes::new(env);
    for b in employee_count.to_be_bytes().iter() {
        a1.push_back(*b);
    }

    // input[2]: merkle_root as 32 bytes
    let mut a2 = Bytes::new(env);
    for i in 0..32u32 {
        a2.push_back(merkle_root.get(i).unwrap());
    }

    let mut inputs: Vec<Bytes> = Vec::new(env);
    inputs.push_back(a0);
    inputs.push_back(a1);
    inputs.push_back(a2);

    ZKProof {
        proof: proof_bytes,
        public_inputs: inputs,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DAO Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_create_dao() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);

    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "TechTown"),
        &String::from_str(&env, "TT"),
        &3u32,
    );

    assert_eq!(dao_id, 1u64);
    let config = c.get_dao(&dao_id);
    assert_eq!(config.admin, admin);
    assert!(!config.paused);
    assert_eq!(config.multisig_threshold, 3);
}

#[test]
#[should_panic]
fn test_create_dao_zero_threshold_panics() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    // threshold = 0 → should panic (ContractError::InvalidThreshold)
    c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &0u32,
    );
}

#[test]
fn test_pause_unpause_dao() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "TechTown"),
        &String::from_str(&env, "TT"),
        &2u32,
    );

    c.pause_dao(&dao_id, &admin);
    assert!(c.get_dao(&dao_id).paused);

    c.unpause_dao(&dao_id, &admin);
    assert!(!c.get_dao(&dao_id).paused);
}

#[test]
#[should_panic]
fn test_unauthorized_pause_panics() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "TechTown"),
        &String::from_str(&env, "TT"),
        &2u32,
    );

    // attacker is not admin → should panic
    c.pause_dao(&dao_id, &attacker);
}

#[test]
fn test_update_dao_settings() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "OldName"),
        &String::from_str(&env, "OLD"),
        &2u32,
    );

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
// Employee Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_add_get_employee() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &2u32,
    );

    let emp_id = c.add_employee(
        &dao_id,
        &admin,
        &wallet,
        &String::from_str(&env, "Engineering"),
        &random_hash(&env),
        &1u64,
    );

    let emp = c.get_employee(&dao_id, &emp_id);
    assert_eq!(emp.wallet, wallet);
    assert_eq!(emp.status, EmployeeStatus::Active);
}

#[test]
fn test_freeze_activate_employee() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &2u32,
    );
    let emp_id = c.add_employee(
        &dao_id, &admin, &wallet,
        &String::from_str(&env, "Eng"),
        &random_hash(&env), &1u64,
    );

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

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &2u32,
    );
    let emp_id = c.add_employee(
        &dao_id, &admin, &wallet,
        &String::from_str(&env, "Eng"),
        &random_hash(&env), &1u64,
    );

    c.remove_employee(&dao_id, &emp_id, &admin);
    assert_eq!(c.get_employee(&dao_id, &emp_id).status, EmployeeStatus::Removed);
}

#[test]
fn test_get_all_employees() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &2u32,
    );

    for _ in 0..3 {
        let w = Address::generate(&env);
        c.add_employee(
            &dao_id, &admin, &w,
            &String::from_str(&env, "Eng"),
            &random_hash(&env), &1u64,
        );
    }

    assert_eq!(c.get_all_employees(&dao_id).len(), 3u32);
}

// ─────────────────────────────────────────────────────────────────────────────
// Payroll Tests
// ─────────────────────────────────────────────────────────────────────────────

/// Returns (env, contract_addr, admin_addr, dao_id, emp_id, commitment_hash)
fn setup_payroll() -> (Env, Address, Address, u64, u64, BytesN<32>) {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);

    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &1u32,
    );

    let commitment_hash = random_hash(&env);
    let emp_id = c.add_employee(
        &dao_id, &admin, &wallet,
        &String::from_str(&env, "Eng"),
        &commitment_hash, &1u64,
    );

    (env, addr, admin, dao_id, emp_id, commitment_hash)
}

fn make_payroll_args(
    env: &Env,
    emp_id: u64,
    commitment_hash: &BytesN<32>,
    total_amount: i128,
) -> (Vec<u64>, Vec<SalaryCommitment>, BytesN<32>) {
    let commitment = SalaryCommitment {
        employee_id: emp_id,
        commitment_hash: commitment_hash.clone(),
        period: 1,
        created_at: env.ledger().timestamp(),
    };
    let mut emps: Vec<u64> = Vec::new(env);
    emps.push_back(emp_id);
    let mut commits: Vec<SalaryCommitment> = Vec::new(env);
    commits.push_back(commitment);

    // Compute a merkle root that is consistent with the claim leaf
    // For testing, use sha256(employee_id || amount) as both leaf and root (single-leaf tree)
    let mut preimage = Bytes::new(env);
    for b in emp_id.to_be_bytes().iter() {
        preimage.push_back(*b);
    }
    for b in total_amount.to_be_bytes().iter() {
        preimage.push_back(*b);
    }
    let merkle_root: BytesN<32> = env.crypto().sha256(&preimage).into();

    (emps, commits, merkle_root)
}

#[test]
fn test_create_and_get_payroll() {
    let (env, addr, admin, dao_id, emp_id, commitment_hash) = setup_payroll();
    let c = client(&env, &addr);
    let total = 1000i128;
    let (emps, commits, root) = make_payroll_args(&env, emp_id, &commitment_hash, total);

    let payroll_id = c.create_payroll(&dao_id, &admin, &1u64, &emps, &commits, &total, &root);

    let p = c.get_payroll(&payroll_id);
    assert_eq!(p.status, PayrollStatus::Pending);
    assert_eq!(p.total_amount, total);
    assert_eq!(p.employee_count, 1u32);
}

#[test]
fn test_approve_payroll() {
    let (env, addr, admin, dao_id, emp_id, commitment_hash) = setup_payroll();
    let c = client(&env, &addr);
    let total = 2000i128;
    let (emps, commits, root) = make_payroll_args(&env, emp_id, &commitment_hash, total);

    let payroll_id = c.create_payroll(&dao_id, &admin, &1u64, &emps, &commits, &total, &root);
    c.approve_payroll(&payroll_id, &dao_id, &admin);

    assert_eq!(c.get_payroll(&payroll_id).status, PayrollStatus::Approved);
}

#[test]
fn test_cancel_payroll() {
    let (env, addr, admin, dao_id, emp_id, commitment_hash) = setup_payroll();
    let c = client(&env, &addr);
    let total = 500i128;
    let (emps, commits, root) = make_payroll_args(&env, emp_id, &commitment_hash, total);

    let payroll_id = c.create_payroll(&dao_id, &admin, &1u64, &emps, &commits, &total, &root);
    c.cancel_payroll(&payroll_id, &dao_id, &admin);

    assert_eq!(c.get_payroll(&payroll_id).status, PayrollStatus::Cancelled);
}

#[test]
#[should_panic]
fn test_double_cancel_panics() {
    let (env, addr, admin, dao_id, emp_id, commitment_hash) = setup_payroll();
    let c = client(&env, &addr);
    let total = 500i128;
    let (emps, commits, root) = make_payroll_args(&env, emp_id, &commitment_hash, total);

    let payroll_id = c.create_payroll(&dao_id, &admin, &1u64, &emps, &commits, &total, &root);
    c.cancel_payroll(&payroll_id, &dao_id, &admin);
    // Second cancel on a Cancelled payroll → InvalidStatus
    c.cancel_payroll(&payroll_id, &dao_id, &admin);
}

#[test]
#[should_panic]
fn test_approve_cancelled_payroll_panics() {
    let (env, addr, admin, dao_id, emp_id, commitment_hash) = setup_payroll();
    let c = client(&env, &addr);
    let total = 500i128;
    let (emps, commits, root) = make_payroll_args(&env, emp_id, &commitment_hash, total);

    let payroll_id = c.create_payroll(&dao_id, &admin, &1u64, &emps, &commits, &total, &root);
    c.cancel_payroll(&payroll_id, &dao_id, &admin);
    // Approve after cancel → should panic
    c.approve_payroll(&payroll_id, &dao_id, &admin);
}

// ─────────────────────────────────────────────────────────────────────────────
// Multisig Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_create_proposal_threshold_1_auto_executes() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    // threshold=1 → proposer's auto-approval immediately executes
    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &1u32,
    );

    let target = Address::generate(&env);
    let proposal_id = c.create_proposal(
        &dao_id,
        &admin,
        &target,
        &String::from_str(&env, "transfer"),
        &Bytes::new(&env),
    );

    let p = c.get_proposal(&dao_id, &proposal_id);
    assert_eq!(p.status, ProposalStatus::Executed);
}

#[test]
fn test_proposal_stays_active_below_threshold() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let approver2 = Address::generate(&env);
    env.mock_all_auths();

    // threshold=3: need 3 approvals
    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &3u32,
    );

    let target = Address::generate(&env);
    let proposal_id = c.create_proposal(
        &dao_id,
        &admin,             // 1st approval
        &target,
        &String::from_str(&env, "transfer"),
        &Bytes::new(&env),
    );

    // Still Active after creation (1/3)
    assert_eq!(c.get_proposal(&dao_id, &proposal_id).status, ProposalStatus::Active);

    // 2nd approval
    c.approve_proposal(&dao_id, &proposal_id, &approver2);
    // Still Active (2/3)
    assert_eq!(c.get_proposal(&dao_id, &proposal_id).status, ProposalStatus::Active);
}

#[test]
fn test_proposal_executes_at_threshold() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let approver2 = Address::generate(&env);
    let approver3 = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &3u32,
    );

    let target = Address::generate(&env);
    let proposal_id = c.create_proposal(
        &dao_id, &admin, &target,
        &String::from_str(&env, "transfer"),
        &Bytes::new(&env),
    );
    // 1/3
    c.approve_proposal(&dao_id, &proposal_id, &approver2);
    // 2/3
    c.approve_proposal(&dao_id, &proposal_id, &approver3);
    // 3/3 → should execute
    assert_eq!(c.get_proposal(&dao_id, &proposal_id).status, ProposalStatus::Executed);
}

#[test]
fn test_reject_proposal() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &3u32,
    );

    let target = Address::generate(&env);
    let proposal_id = c.create_proposal(
        &dao_id, &proposer, &target,
        &String::from_str(&env, "transfer"),
        &Bytes::new(&env),
    );

    c.reject_proposal(&dao_id, &proposal_id, &admin);
    assert_eq!(c.get_proposal(&dao_id, &proposal_id).status, ProposalStatus::Rejected);
}

#[test]
#[should_panic]
fn test_duplicate_approval_panics() {
    let (env, addr) = setup();
    let c = client(&env, &addr);
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let dao_id = c.create_dao(
        &admin,
        &String::from_str(&env, "DAO"),
        &String::from_str(&env, "DAO"),
        &3u32,
    );

    let target = Address::generate(&env);
    let proposal_id = c.create_proposal(
        &dao_id, &admin, &target,
        &String::from_str(&env, "transfer"),
        &Bytes::new(&env),
    );

    // admin already approved at creation → second approve panics
    c.approve_proposal(&dao_id, &proposal_id, &admin);
}

// ─────────────────────────────────────────────────────────────────────────────
// ZK commitment helper test
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_compute_commitment_deterministic() {
    let (env, addr) = setup();
    let c = client(&env, &addr);

    let mut randomness = Bytes::new(&env);
    randomness.push_back(0xDE);
    randomness.push_back(0xAD);
    randomness.push_back(0xBE);
    randomness.push_back(0xEF);

    let h1 = c.compute_commitment(&1u64, &50000i128, &randomness);
    let h2 = c.compute_commitment(&1u64, &50000i128, &randomness);
    // Same inputs → same hash
    assert_eq!(h1, h2);

    let h3 = c.compute_commitment(&2u64, &50000i128, &randomness);
    // Different employee_id → different hash
    assert_ne!(h1, h3);
}
