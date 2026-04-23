use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{error::VaultError, events::Deposited, state::VaultState};

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        constraint = owner_token_account.owner == owner.key()     @VaultError::Unauthorized,
    )]
    pub owner_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref(), vault_state.mint.as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64 ) -> Result<()> {
        require!(amount > 0, VaultError::ZeroAmount);

        let balance = self.vault_token_account.amount.checked_add(amount).ok_or(VaultError::Overflow)?;

        token::transfer(
            CpiContext::new(
                self.token_program.key(), 
                Transfer {
                    from: self.owner_token_account.to_account_info(),
                    to: self.vault_token_account.to_account_info(),
                    authority: self.owner.to_account_info(),
                },
            ), 
            amount
        )?;

        emit!(Deposited {
            owner: self.owner.key(),
            mint: self.vault_state.mint,
            amount,
            balance,
        });

        Ok(())
    }
}