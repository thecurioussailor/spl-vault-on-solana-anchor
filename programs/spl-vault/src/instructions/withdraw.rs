use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{error::VaultError, events::Withdrawn, state::VaultState};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault_state", owner.key().as_ref(), vault_state.mint.as_ref()],
        bump = vault_state.bump,
        has_one = owner @ VaultError::Unauthorized,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        constraint = owner_token_account.mint == vault_state.mint @ VaultError::MintMismatch,
        constraint = owner_token_account.owner == owner.key(),
    )]
    pub owner_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref(), vault_state.mint.as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::ZeroAmount);
        require!(
            self.vault_token_account.amount >= amount,
            VaultError::InsufficientFunds
        );

        let owner_key = self.owner.key();
        let mint_key = self.vault_state.mint;
        let seeds = &[
            b"vault_state",
            owner_key.as_ref(),
            mint_key.as_ref(),
            &[self.vault_state.bump]
        ];

        let remaining = self.vault_token_account.amount - amount;

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.key(), 
                Transfer {
                    from: self.vault_token_account.to_account_info(),
                    to: self.owner_token_account.to_account_info(),
                    authority: self.vault_state.to_account_info(),
                }, 
                &[&seeds[..]],
            ),
            amount,
        )?;

        emit!(Withdrawn {
            owner: self.owner.key(),
            mint: self.vault_state.mint,
            amount,
            remaining,
        });

        Ok(())
    }
}