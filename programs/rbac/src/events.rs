use anchor_lang::prelude::*;

#[event]
pub struct ResourceInitialized {
    pub resource: Pubkey,
    pub admin: Pubkey,
    pub name: String,
}

#[event]
pub struct RoleCreated {
    pub resource: Pubkey,
    pub role: Pubkey,
    pub name: String,
}

#[event]
pub struct RoleGranted {
    pub resource: Pubkey,
    pub role: Pubkey,
    pub user: Pubkey,
    pub granted_at: i64,
}

#[event]
pub struct RoleRevoked {
    pub resource: Pubkey,
    pub role: Pubkey,
    pub user: Pubkey,
}

#[event]
pub struct AdminTransferred {
    pub resource: Pubkey,
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
}
