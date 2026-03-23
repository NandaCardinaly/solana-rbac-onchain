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

    pub fn initialize_resource(
                ctx: Context<InitializeResource>,
                name: String,
                resource_id: [u8; 16],
            ) -> Result<()> {
                require!(name.len() <= 32, errors::RbacError::NameTooLong);
                instructions::initialize_resource::handler(ctx, name, resource_id)
    }

    pub fn create_role(ctx: Context<CreateRole>, role_name: String) -> Result<()> {
                require!(role_name.len() <= 32, errors::RbacError::NameTooLong);
                instructions::create_role::handler(ctx, role_name)
    }

    pub fn grant_role(ctx: Context<GrantRole>) -> Result<()> {
                instructions::grant_role::handler(ctx)
    }

    pub fn revoke_role(ctx: Context<RevokeRole>) -> Result<()> {
                instructions::revoke_role::handler(ctx)
    }

    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
                instructions::transfer_admin::handler(ctx, new_admin)
    }

    pub fn check_permission(ctx: Context<CheckPermission>) -> Result<()> {
                instructions::check_permission::handler(ctx)
    }
}
