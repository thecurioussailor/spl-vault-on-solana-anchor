use anchor_lang::prelude::*;

pub mod error;
pub mod events;
pub mod state;
pub mod instructions;

pub use instructions::*;


declare_id!("CPq94NSjtQtEm2rKVoGCkMLCusmeYq9j4EZqbuhsjupQ");

#[program]
pub mod spl_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}
