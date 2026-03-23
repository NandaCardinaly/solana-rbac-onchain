use anchor_lang::prelude::*;
use crate::state::ResourceAccount;
use crate::errors::RbacError;
use crate::events::ResourceInitialized;

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

pub fn handler(ctx: Context<InitializeResource>, name: String, resource_id: [u8; 16]) -> Result<()> {
      require!(name.len() <= ResourceAccount::MAX_NAME_LEN, RbacError::NameTooLong);
      let r = &mut ctx.accounts.resource;
      r.admin = ctx.accounts.admin.key();
      r.name = name.clone();
      r.resource_id = resource_id;
      r.bump = ctx.bumps.resource;
      emit!(ResourceInitialized {
                resource: r.key(),
                admin: r.admin,
                name,
            });
      Ok(())
  }
