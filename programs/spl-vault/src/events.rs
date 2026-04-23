use anchor_lang::prelude::*;

#[event]
pub struct VaultInitialized {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
}

#[event]
pub struct Deposited {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub balance: u64, //current vault balance
}

#[event]
pub struct Withdrawn {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub remaining: u64,
}

#[event]
pub struct VaultClosed {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub token_returned: u64,
}
