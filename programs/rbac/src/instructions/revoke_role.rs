use anchor_lang::prelude::*;
use crate::state::{ResourceAccount, RoleAccount, AssignmentAccount};
use crate::errors::RbacError;
use crate::events::RoleRevoked;

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

pub fn handler(ctx: Context<RevokeRole>) -> Result<()> {
      emit!(RoleRevoked {
                resource: ctx.accounts.resource.key(),
                role: ctx.accounts.role.key(),
                user: ctx.accounts.user.key(),
      });
      Ok(())
}
