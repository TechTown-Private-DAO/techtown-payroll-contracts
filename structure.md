use soroban_sdk::{Address, Env, Vec, Bytes, String};
use crate::types::*;
use crate::storage::Storage;
use crate::errors::ContractError;

pub struct MultisigContract;

impl MultisigContract {
    pub fn create_proposal(
        env: Env,
        dao_id: u64,
        proposer: Address,
        target: Address,
        function: String,
        args: Bytes,
    ) -> Result<u64, ContractError> {
        proposer.require_auth();

        let config = Storage::get_dao(&env, dao_id)?;
        if config.paused {
            return Err(ContractError::Paused);
        }

        let proposal_id = Storage::increment_counter(&env, "proposal_counter");
        
        let proposal = MultisigProposal {
            id: proposal_id,
            dao_id,
            proposer: proposer.clone(),
            target,
            function,
            args,
            approvals: Vec::new(&env),
            status: ProposalStatus::Active,
            created_at: env.ledger().timestamp(),
            executed_at: None,
        };

        // Auto-approve by proposer
        let mut approvals = Vec::new(&env);
        approvals.push_back(proposer);
        let proposal = MultisigProposal {
            approvals,
            ..proposal
        };

        Storage::save_proposal(&env, dao_id, proposal);
        Ok(proposal_id)
    }

    pub fn approve_proposal(
        env: Env,
        dao_id: u64,
        proposal_id: u64,
        approver: Address,
    ) -> Result<(), ContractError> {
        approver.require_auth();

        let mut proposal = Storage::get_proposal(&env, dao_id, proposal_id)?;
        
        if proposal.status != ProposalStatus::Active {
            return Err(ContractError::ProposalExpired);
        }

        // Check if already approved
        for approval in &proposal.approvals {
            if approval.eq(&approver) {
                return Err(ContractError::AlreadyApproved);
            }
        }

        proposal.approvals.push_back(approver);

        // Check if threshold is met
        let config = Storage::get_dao(&env, dao_id)?;
        if proposal.approvals.len() >= config.multisig_threshold as usize {
            proposal.status = ProposalStatus::Approved;
            // Execute proposal automatically when threshold is met
            Self::execute_proposal(&env, &mut proposal)?;
        }

        Storage::save_proposal(&env, dao_id, proposal);
        Ok(())
    }

    fn execute_proposal(env: &Env, proposal: &mut MultisigProposal) -> Result<(), ContractError> {
        // Execute the proposal target function
        // In production, this would call the target contract with the specified function and args
        
        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = Some(env.ledger().timestamp());
        
        Ok(())
    }

    pub fn reject_proposal(
        env: Env,
        dao_id: u64,
        proposal_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();

        let mut proposal = Storage::get_proposal(&env, dao_id, proposal_id)?;
        
        let config = Storage::get_dao(&env, dao_id)?;
        if config.admin != admin {
            return Err(ContractError::Unauthorized);
        }

        proposal.status = ProposalStatus::Rejected;
        Storage::save_proposal(&env, dao_id, proposal);
        Ok(())
    }

    pub fn get_proposal(
        env: Env,
        dao_id: u64,
        proposal_id: u64,
    ) -> Result<MultisigProposal, ContractError> {
        Storage::get_proposal(&env, dao_id, proposal_id)
    }

    pub fn get_all_proposals(
        env: Env,
        dao_id: u64,
    ) -> Vec<MultisigProposal> {
        let mut proposals = Vec::new(&env);
        let mut id = 0;
        loop {
            if let Ok(proposal) = Storage::get_proposal(&env, dao_id, id) {
                proposals.push_back(proposal);
                id += 1;
            } else {
                break;
            }
        }
        proposals
    }
}