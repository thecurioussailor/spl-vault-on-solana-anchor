use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("You are not the owner of this account")]
    Unauthorized,

    #[msg("Amount must be greater than Zero")]
    ZeroAmount,

    #[msg("Insufficient tokens in vault")]
    InsufficientFunds,

    #[msg("Token mint does not match this vault")]
    MintMismatch,

    #[msg("Overflow")]
    Overflow
}