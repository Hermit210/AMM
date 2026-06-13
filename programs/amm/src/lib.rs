use anchor_lang::prelude::*;

mod contexts;
mod error;
mod helpers;
mod state;

use contexts::*;

declare_id!("CMhDzEyr2JKSQYQwWpRhD6W5wyWJxhnjx5iyx9mTk6tD");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed: u64, fee: u16) -> Result<()> {
        ctx.accounts.initialize(seed, fee, &ctx.bumps)
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        amount_x: u64,
        amount_y: u64,
        min_lp: u64,
    ) -> Result<()> {
        ctx.accounts.deposit(amount_x, amount_y, min_lp)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        lp_amount: u64,
        min_x: u64,
        min_y: u64,
    ) -> Result<()> {
        ctx.accounts.withdraw(lp_amount, min_x, min_y)
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
        is_x: bool,
    ) -> Result<()> {
        ctx.accounts.swap(amount_in, min_amount_out, is_x)
    }
}
