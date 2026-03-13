use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct ResourceAccount {
    pub admin: Pubkey,
    pub name: String,
    pub resource_id: [u8; 16],
    pub bump: u8,
}

impl ResourceAccount {
    pub const MAX_NAME_LEN: usize = 32;
    pub const LEN: usize = 8 + 32 + (4 + Self::MAX_NAME_LEN) + 16 + 1;
}

#[account]
#[derive(Default)]
pub struct RoleAccount {
    pub resource: Pubkey,
    pub name: String,
    pub bump: u8,
}

impl RoleAccount {
    pub const MAX_NAME_LEN: usize = 32;
    pub const LEN: usize = 8 + 32 + (4 + Self::MAX_NAME_LEN) + 1;
}

#[account]
#[derive(Default)]
pub struct AssignmentAccount {
    pub role: Pubkey,
    pub user: Pubkey,
    pub granted_at: i64,
    pub bump: u8,
}

impl AssignmentAccount {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 1;
}
