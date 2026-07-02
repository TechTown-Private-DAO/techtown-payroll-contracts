use soroban_sdk::{Address, BytesN, Env, String, Vec};
use crate::types::{Employee, EmployeeStatus, SalaryCommitment};
use crate::storage::Storage;
use crate::errors::ContractError;
use crate::event::Events;
use crate::dao::DAOContract;

pub struct EmployeeContract;

impl EmployeeContract {
    /// Register a new employee in the DAO.
    ///
    /// `commitment_hash` – hash(salary, randomness, employee_id) committed by
    ///                     the admin/employee off-chain; salary stays private.
    pub fn add_employee(
        env: &Env,
        dao_id: u64,
        admin: Address,
        wallet: Address,
        department: String,
        commitment_hash: BytesN<32>,
        period: u64,
    ) -> Result<u64, ContractError> {
        admin.require_auth();
        DAOContract::require_admin(env, dao_id, &admin)?;

        let employee_id = Storage::next_employee_id(env);

        let employee = Employee {
            id: employee_id,
            wallet: wallet.clone(),
            department,
            status: EmployeeStatus::Active,
            commitment_hash,
            joined_at: env.ledger().timestamp(),
            last_payroll: 0,
            last_paid_period: 0,
        };

        // Store the commitment alongside the employee
        let commitment = SalaryCommitment {
            employee_id,
            commitment_hash: employee.commitment_hash.clone(),
            period,
            created_at: env.ledger().timestamp(),
        };

        Storage::save_employee(env, dao_id, &employee);
        Storage::save_commitment(env, dao_id, employee_id, &commitment);
        Events::employee_added(env, dao_id, employee_id, &wallet);

        // Increment DAO member count
        let mut config = Storage::get_dao(env, dao_id)?;
        config.total_members += 1;
        Storage::save_dao(env, dao_id, &config);

        Ok(employee_id)
    }

    /// Soft-remove an employee (sets status to Removed; data is preserved).
    pub fn remove_employee(
        env: &Env,
        dao_id: u64,
        employee_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        DAOContract::require_admin(env, dao_id, &admin)?;

        let mut employee = Storage::get_employee(env, dao_id, employee_id)?;
        employee.status = EmployeeStatus::Removed;
        Storage::save_employee(env, dao_id, &employee);
        Events::employee_removed(env, dao_id, employee_id);
        Ok(())
    }

    /// Employee can update their own receiving wallet.
    pub fn update_wallet(
        env: &Env,
        dao_id: u64,
        employee_id: u64,
        caller: Address,
        new_wallet: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let mut emp = Storage::get_employee(env, dao_id, employee_id)?;

        if emp.wallet != caller {
            return Err(ContractError::Unauthorized);
        }

        if emp.status == EmployeeStatus::Removed {
            return Err(ContractError::EmployeeNotActive);
        }

        emp.wallet = new_wallet.clone();
        Storage::save_employee(env, dao_id, &emp);
        Events::wallet_updated(env, dao_id, employee_id, &new_wallet);
        Ok(())
    }

    /// Admin freezes an employee — they remain in the registry but cannot claim salary.
    pub fn freeze_employee(
        env: &Env,
        dao_id: u64,
        employee_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        DAOContract::require_admin(env, dao_id, &admin)?;

        let mut employee = Storage::get_employee(env, dao_id, employee_id)?;

        if employee.status == EmployeeStatus::Removed {
            return Err(ContractError::EmployeeNotFound);
        }

        employee.status = EmployeeStatus::Frozen;
        Storage::save_employee(env, dao_id, &employee);
        Events::employee_frozen(env, dao_id, employee_id);
        Ok(())
    }

    /// Admin activates a previously frozen employee.
    pub fn activate_employee(
        env: &Env,
        dao_id: u64,
        employee_id: u64,
        admin: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        DAOContract::require_admin(env, dao_id, &admin)?;

        let mut employee = Storage::get_employee(env, dao_id, employee_id)?;

        if employee.status == EmployeeStatus::Removed {
            return Err(ContractError::EmployeeNotFound);
        }

        employee.status = EmployeeStatus::Active;
        Storage::save_employee(env, dao_id, &employee);
        Events::employee_activated(env, dao_id, employee_id);
        Ok(())
    }

    /// Update the salary commitment hash for an employee (e.g. when pay changes).
    pub fn update_commitment(
        env: &Env,
        dao_id: u64,
        employee_id: u64,
        admin: Address,
        new_commitment_hash: BytesN<32>,
        period: u64,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        DAOContract::require_admin(env, dao_id, &admin)?;

        let mut employee = Storage::get_employee(env, dao_id, employee_id)?;

        if employee.status != EmployeeStatus::Active {
            return Err(ContractError::EmployeeNotActive);
        }

        employee.commitment_hash = new_commitment_hash.clone();
        Storage::save_employee(env, dao_id, &employee);

        let commitment = SalaryCommitment {
            employee_id,
            commitment_hash: new_commitment_hash,
            period,
            created_at: env.ledger().timestamp(),
        };
        Storage::save_commitment(env, dao_id, employee_id, &commitment);

        Ok(())
    }

    pub fn get_employee(env: &Env, dao_id: u64, employee_id: u64) -> Result<Employee, ContractError> {
        Storage::get_employee(env, dao_id, employee_id)
    }

    pub fn get_all_employees(env: &Env, dao_id: u64) -> Vec<Employee> {
        Storage::get_all_employees(env, dao_id)
    }
}
