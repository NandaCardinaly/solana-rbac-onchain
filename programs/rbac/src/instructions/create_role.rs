use anchor_lang::prelude::*;
use crate::state::{ResourceAccount, RoleAccount};
use crate::errors::RbacError;
use crate::events::RoleCreated;

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

pub fn handler(ctx: Context<CreateRole>, role_name: String) -> Result<()> {
      require!(role_name.len() <= RoleAccount::MAX_NAME_LEN, RbacError::NameTooLong);
      let role = &mut ctx.accounts.role;
      role.resource = ctx.accounts.resource.key();
      role.name = role_name.clone();
      role.bump = ctx.bumps.role;
      emit!(RoleCreated {
                resource: ctx.accounts.resource.key(),
                role: role.key(),
                name: role_name,
      });
      Ok(())
}
