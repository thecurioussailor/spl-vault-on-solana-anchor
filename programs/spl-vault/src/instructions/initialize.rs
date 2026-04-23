use anchor_lang::prelude::*;

use anchor_spl::{
    token::{Mint, Token, TokenAccount},
};

use crate::{events::VaultInitialized, state::VaultState};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = owner,
        space = 8 + VaultState::INIT_SPACE,
        seeds = [b"vault_state", owner.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = owner,
        token::mint = mint,
        token::authority = vault_state,
        seeds = [b"vault", owner.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.vault_state.set_inner(VaultState { 
            owner: self.owner.key(), 
            mint: self.mint.key(), 
            bump: bumps.vault_state, 
            vault_bump: bumps.vault_token_account,
        });

        emit!( VaultInitialized {
            owner: self.owner.key(),
            mint: self.mint.key(),
            vault: self.vault_token_account.key(),
        });

        Ok(())
    }
}