use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};

use crate::error::AmmError;
use crate::state::Config;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,

    pub mint_x: InterfaceAccount<'info, Mint>,
    pub mint_y: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"amm", config.mint_x.as_ref(), config.mint_y.as_ref(), config.seed.to_le_bytes().as_ref()],
        bump = config.bump,
        has_one = mint_x,
        has_one = mint_y,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub lp_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = withdrawer,
    )]
    pub withdrawer_ata_x: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = withdrawer,
    )]
    pub withdrawer_ata_y: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = withdrawer,
    )]
    pub withdrawer_ata_lp: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(
        &mut self,
        lp_amount: u64,
        min_x: u64,
        min_y: u64,
    ) -> Result<()> {
        require!(lp_amount > 0, AmmError::ZeroAmount);

        let lp_supply = self.lp_mint.supply;
        require!(lp_supply > 0, AmmError::PoolEmpty);

        let vault_x_amount = self.vault_x.amount;
        let vault_y_amount = self.vault_y.amount;

        // Calculate proportional share of each token
        let amount_x = (vault_x_amount as u128)
            .checked_mul(lp_amount as u128)
            .ok_or(AmmError::Overflow)?
            .checked_div(lp_supply as u128)
            .ok_or(AmmError::Overflow)? as u64;

        let amount_y = (vault_y_amount as u128)
            .checked_mul(lp_amount as u128)
            .ok_or(AmmError::Overflow)?
            .checked_div(lp_supply as u128)
            .ok_or(AmmError::Overflow)? as u64;

        require!(amount_x >= min_x, AmmError::SlippageExceeded);
        require!(amount_y >= min_y, AmmError::SlippageExceeded);

        // Burn LP tokens
        self.burn_lp_tokens(lp_amount)?;

        // Transfer token X from vault to withdrawer
        self.transfer_from_vault(
            &self.vault_x,
            &self.withdrawer_ata_x,
            &self.mint_x,
            amount_x,
        )?;

        // Transfer token Y from vault to withdrawer
        self.transfer_from_vault(
            &self.vault_y,
            &self.withdrawer_ata_y,
            &self.mint_y,
            amount_y,
        )?;

        Ok(())
    }

    fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Burn {
            mint: self.lp_mint.to_account_info(),
            from: self.withdrawer_ata_lp.to_account_info(),
            authority: self.withdrawer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        burn(cpi_ctx, amount)
    }

    fn transfer_from_vault(
        &self,
        from: &InterfaceAccount<'info, TokenAccount>,
        to: &InterfaceAccount<'info, TokenAccount>,
        mint: &InterfaceAccount<'info, Mint>,
        amount: u64,
    ) -> Result<()> {
        let seeds = &[
            b"amm",
            self.config.mint_x.as_ref(),
            self.config.mint_y.as_ref(),
            &self.config.seed.to_le_bytes(),
            &[self.config.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = TransferChecked {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: self.config.to_account_info(),
            mint: mint.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        transfer_checked(cpi_ctx, amount, mint.decimals)
    }
}
