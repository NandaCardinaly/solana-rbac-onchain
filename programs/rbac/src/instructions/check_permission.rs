use anchor_lang::prelude::*;
use crate::state::{RoleAccount, AssignmentAccount};
use crate::errors::RbacError;

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

pub fn handler(ctx: Context<CheckPermission>) -> Result<()> {
      msg!(
                "Permission check passed: user {} holds role {}",
                ctx.accounts.user.key(),
                ctx.accounts.role.name
            );
      Ok(())
}
