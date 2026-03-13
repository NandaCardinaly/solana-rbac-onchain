use anchor_lang::prelude::*;

declare_id!("RBACxqmGJv8sF3FjQLZ7neQhQdaVHBFfFkCLmqNBjkG");

pub mod instructions;
pub mod state;
pub mod errors;
pub mod events;

use instructions::*;

#[program]
pub mod rbac {
    use super::*;

    /// Creates a new resource with the caller as admin.
    pub fn initialize_resource(
        ctx: Context<InitializeResource>,
        name: String,
        resource_id: [u8; 16],
    ) -> Result<()> {
        require!(name.len() <= 32, errors::RbacError::NameTooLong);
        instructions::initialize_resource::handler(ctx, name, resource_id)
    }

    /// Creates a named role scoped to a resource. Only the resource admin can call this.
    pub fn create_role(ctx: Context<CreateRole>, role_name: String) -> Result<()> {
        require!(role_name.len() <= 32, errors::RbacError::NameTooLong);
        instructions::create_role::handler(ctx, role_name)
    }

    /// Assigns a role to a user wallet. Only the resource admin can call this.
    pub fn grant_role(ctx: Context<GrantRole>) -> Result<()> {
        instructions::grant_role::handler(ctx)
    }

    /// Removes a role from a user wallet and closes the assignment account.
    /// Rent is returned to the admin.
    pub fn revoke_role(ctx: Context<RevokeRole>) -> Result<()> {
        instructions::revoke_role::handler(ctx)
    }

    /// Transfers admin authority to a new pubkey.
    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::transfer_admin::handler(ctx, new_admin)
    }

    /// On-chain permission check. Succeeds iff the user holds the role.
    /// Designed to be called via CPI from other programs as an access gate.
    pub fn check_permission(ctx: Context<CheckPermission>) -> Result<()> {
        instructions::check_permission::handler(ctx)
    }
}
