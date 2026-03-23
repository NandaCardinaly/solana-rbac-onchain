use anchor_lang::prelude::*;
use crate::state::ResourceAccount;
use crate::errors::RbacError;
use crate::events::AdminTransferred;

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
      #[account(mut, has_one = admin @ RbacError::NotAdmin)]
      pub resource: Account<'info, ResourceAccount>,
      pub admin: Signer<'info>,
}

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
