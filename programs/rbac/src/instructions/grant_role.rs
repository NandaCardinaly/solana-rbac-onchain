use anchor_lang::prelude::*;
use crate::state::{ResourceAccount, RoleAccount, AssignmentAccount};
use crate::errors::RbacError;
use crate::events::RoleGranted;

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
