use soroban_sdk::{Address, Bytes, Env, String, Vec};
use crate::types::{MultisigProposal, ProposalStatus};
use crate::storage::Storage;
use crate::errors::ContractError;
use crate::event::Events;
use crate::dao::DAOContract;

pub struct MultisigContract;

impl MultisigContract {
    /// Create a new governance proposal.
    ///
    /// The proposer automatically counts as the first approval.
    pub fn create_proposal(
        env: &Env,
        dao_id: u64,
        proposer: Address,
        target: Address,
        function: String,
        args: Bytes,
    ) -> Result<u64, ContractError> {
        proposer.require_auth();
        DAOContract::require_active(env, dao_id)?;

        let proposal_id = Storage::next_proposal_id(env);

        let mut approvals: Vec<Address> = Vec::new(env);
        approvals.push_back(proposer.clone());

        let mut proposal = MultisigProposal {
            id: proposal_id,
            dao_id,
            proposer: proposer.clone(),
            target,
            function,
            args,
            approvals,
            status: ProposalStatus::Active,
            created_at: env.ledger().timestamp(),
            executed_at: 0,
        };

        // If threshold is 1, the proposer's auto-approval immediately executes
        let config = Storage::get_dao(env, dao_id)?;
        if 1u32 >= config.multisig_threshold {
            proposal.status = ProposalStatus::Executed;
            proposal.executed_at = env.ledger().timestamp();
            Events::proposal_executed(env, proposal_id);
        }

        Storage::save_proposal(env, dao_id, &proposal);
        Events::proposal_created(env, proposal_id, dao_id, &proposer);
        Ok(proposal_id)
    }

    /// Approve a proposal. Automatically executes when the threshold is met.
    pub fn approve_proposal(
        env: &Env,
        dao_id: u64,
        proposal_id: u64,
        approver: Address,
    ) -> Result<(), ContractError> {
        approver.require_auth();
        let config = DAOContract::require_active(env, dao_id)?;

        let mut proposal = Storage::get_proposal(env, dao_id, proposal_id)?;

        if proposal.status != ProposalStatus::Active {
            return Err(ContractError::ProposalExpired);
        }

        // Prevent duplicate approvals
        for existing in proposal.approvals.iter() {
            if existing == approver {
                return Err(ContractError::AlreadyApproved);
            }
        }

        proposal.approvals.push_back(approver.clone());
        Events::proposal_approved(env, proposal_id, &approver);

        // Check threshold — compare u32 values, no usize conversion needed
        let approval_count = proposal.approvals.len() as u32;
        if approval_count >= config.multisig_threshold {
            proposal.status = ProposalStatus::Approved;
            // Auto-execute when threshold is met
            proposal.status = ProposalStatus::Executed;
            proposal.executed_at = env.ledger().timestamp();
            Events::proposal_executed(env, proposal_id);
        }

        Storage::save_proposal(env, dao_id, &proposal);
        Ok(())
    }

    /// Admin can reject an active proposal.
    pub fn reject_proposal(
        env: &Env,
        dao_id: u64,
        proposal_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        DAOContract::require_admin(env, dao_id, &admin)?;

        let mut proposal = Storage::get_proposal(env, dao_id, proposal_id)?;

        if proposal.status != ProposalStatus::Active {
            return Err(ContractError::ProposalExpired);
        }

        proposal.status = ProposalStatus::Rejected;
        Storage::save_proposal(env, dao_id, &proposal);
        Events::proposal_rejected(env, proposal_id, &admin);
        Ok(())
    }

    pub fn get_proposal(env: &Env, dao_id: u64, proposal_id: u64) -> Result<MultisigProposal, ContractError> {
        Storage::get_proposal(env, dao_id, proposal_id)
    }

    /// Return all proposals for a DAO, up to the current proposal counter.
    pub fn get_all_proposals(env: &Env, dao_id: u64) -> Vec<MultisigProposal> {
        let mut proposals = Vec::new(env);
        // The proposal counter tracks the last assigned id
        let max_id = env
            .storage()
            .persistent()
            .get::<crate::storage::DataKey, u64>(&crate::storage::DataKey::ProposalCounter)
            .unwrap_or(0);

        for id in 1..=max_id {
            if let Ok(p) = Storage::get_proposal(env, dao_id, id) {
                proposals.push_back(p);
            }
        }
        proposals
    }
}
