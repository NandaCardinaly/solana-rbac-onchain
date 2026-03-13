use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::RbacError;
use crate::events::*;

pub mod initialize_resource;
pub mod create_role;
pub mod grant_role;
pub mod revoke_role;
pub mod transfer_admin;
pub mod check_permission;

pub use initialize_resource::*;
pub use create_role::*;
pub use grant_role::*;
pub use revoke_role::*;
pub use transfer_admin::*;
pub use check_permission::*;

// ─────────────────────────────────────────────
// initialize_resource
// ─────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(name: String, resource_id: [u8; 16])]
pub struct InitializeResource<'info> {
    #[account(
        init,
        payer = admin,
        space = ResourceAccount::LEN,
        seeds = [b"resource", name.as_bytes(), admin.key().as_ref()],
        bump
    )]
    pub resource: Account<'info, ResourceAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub mod initialize_resource {
    use super::*;
    pub fn handler(ctx: Context<InitializeResource>, name: String, resource_id: [u8; 16]) -> Result<()> {
        let r = &mut ctx.accounts.resource;
        r.admin = ctx.accounts.admin.key();
        r.name = name.clone();
        r.resource_id = resource_id;
        r.bump = ctx.bumps.resource;
        emit!(ResourceInitialized { resource: r.key(), admin: r.admin, name });
        Ok(())
    }
}

// ─────────────────────────────────────────────
// create_role
// ─────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(role_name: String)]
pub struct CreateRole<'info> {
    #[account(has_one = admin @ RbacError::NotAdmin)]
    pub resource: Account<'info, ResourceAccount>,
    #[account(
        init,
        payer = admin,
        space = RoleAccount::LEN,
        seeds = [b"role", resource.key().as_ref(), role_name.as_bytes()],
        bump
    )]
    pub role: Account<'info, RoleAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub mod create_role {
    use super::*;
    pub fn handler(ctx: Context<CreateRole>, role_name: String) -> Result<()> {
        let role = &mut ctx.accounts.role;
        role.resource = ctx.accounts.resource.key();
        role.name = role_name.clone();
        role.bump = ctx.bumps.role;
        emit!(RoleCreated { resource: ctx.accounts.resource.key(), role: role.key(), name: role_name });
        Ok(())
    }
}

// ─────────────────────────────────────────────
// grant_role
// ─────────────────────────────────────────────

#[derive(Accounts)]
pub struct GrantRole<'info> {
    #[account(has_one = admin @ RbacError::NotAdmin)]
    pub resource: Account<'info, ResourceAccount>,
    #[account(constraint = role.resource == resource.key())]
    pub role: Account<'info, RoleAccount>,
    #[account(
        init,
        payer = admin,
        space = AssignmentAccount::LEN,
        seeds = [b"assignment", role.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub assignment: Account<'info, AssignmentAccount>,
    /// CHECK: Recipient wallet. Any pubkey is valid.
    pub user: UncheckedAccount<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub mod grant_role {
    use super::*;
    pub fn handler(ctx: Context<GrantRole>) -> Result<()> {
        let a = &mut ctx.accounts.assignment;
        a.role = ctx.accounts.role.key();
        a.user = ctx.accounts.user.key();
        a.granted_at = Clock::get()?.unix_timestamp;
        a.bump = ctx.bumps.assignment;
        emit!(RoleGranted {
            resource: ctx.accounts.resource.key(),
            role: a.role,
            user: a.user,
            granted_at: a.granted_at,
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────
// revoke_role
// ─────────────────────────────────────────────

#[derive(Accounts)]
pub struct RevokeRole<'info> {
    #[account(has_one = admin @ RbacError::NotAdmin)]
    pub resource: Account<'info, ResourceAccount>,
    pub role: Account<'info, RoleAccount>,
    #[account(
        mut,
        close = admin,
        seeds = [b"assignment", role.key().as_ref(), user.key().as_ref()],
        bump = assignment.bump,
        constraint = assignment.user == user.key()
    )]
    pub assignment: Account<'info, AssignmentAccount>,
    /// CHECK: The user losing the role.
    pub user: UncheckedAccount<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

pub mod revoke_role {
    use super::*;
    pub fn handler(ctx: Context<RevokeRole>) -> Result<()> {
        emit!(RoleRevoked {
            resource: ctx.accounts.resource.key(),
            role: ctx.accounts.role.key(),
            user: ctx.accounts.user.key(),
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────
// transfer_admin
// ─────────────────────────────────────────────

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(mut, has_one = admin @ RbacError::NotAdmin)]
    pub resource: Account<'info, ResourceAccount>,
    pub admin: Signer<'info>,
}

pub mod transfer_admin {
    use super::*;
    pub fn handler(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        require!(new_admin != ctx.accounts.admin.key(), RbacError::SameAdmin);
        let old = ctx.accounts.resource.admin;
        ctx.accounts.resource.admin = new_admin;
        emit!(AdminTransferred {
            resource: ctx.accounts.resource.key(),
            old_admin: old,
            new_admin,
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────
// check_permission
// ─────────────────────────────────────────────

#[derive(Accounts)]
pub struct CheckPermission<'info> {
    pub role: Account<'info, RoleAccount>,
    #[account(
        seeds = [b"assignment", role.key().as_ref(), user.key().as_ref()],
        bump = assignment.bump,
        constraint = assignment.user == user.key() @ RbacError::AccessDenied,
    )]
    pub assignment: Account<'info, AssignmentAccount>,
    /// CHECK: The user whose permission we are verifying.
    pub user: UncheckedAccount<'info>,
}

pub mod check_permission {
    use super::*;
    pub fn handler(ctx: Context<CheckPermission>) -> Result<()> {
        msg!(
            "Permission check passed: user {} holds role {}",
            ctx.accounts.user.key(),
            ctx.accounts.role.name
        );
        Ok(())
    }
}
