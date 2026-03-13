use anchor_lang::prelude::*;

#[error_code]
pub enum RbacError {
    #[msg("Signer is not the resource admin")]
    NotAdmin,
    #[msg("Access denied: user does not hold the required role")]
    AccessDenied,
    #[msg("Name exceeds maximum length of 32 characters")]
    NameTooLong,
    #[msg("New admin must be different from the current admin")]
    SameAdmin,
}
