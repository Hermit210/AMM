use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};

use crate::error::AmmError;
use crate::helpers::calculate_swap_output;
use crate::state::Config;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub trader: Signer<'info>,

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
        associated_token::authority = trader,
    )]
    pub trader_ata_x: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = trader,
    )]
    pub trader_ata_y: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    /// Swap tokens. If `is_x` is true, swap X -> Y. Otherwise, swap Y -> X.
    pub fn swap(
        &mut self,
        amount_in: u64,
        min_amount_out: u64,
        is_x: bool,
    ) -> Result<()> {
        require!(amount_in > 0, AmmError::ZeroAmount);

        let (reserve_in, reserve_out) = if is_x {
            (self.vault_x.amount, self.vault_y.amount)
        } else {
            (self.vault_y.amount, self.vault_x.amount)
        };

        let amount_out = calculate_swap_output(
            amount_in,
            reserve_in,
            reserve_out,
            self.config.fee,
        )?;

        require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);

        if is_x {
            // Transfer X from trader to vault
            self.transfer_to_vault(
                &self.trader_ata_x,
                &self.vault_x,
                &self.mint_x,
                amount_in,
            )?;
            // Transfer Y from vault to trader
            self.transfer_from_vault(
                &self.vault_y,
                &self.trader_ata_y,
                &self.mint_y,
                amount_out,
            )?;
        } else {
            // Transfer Y from trader to vault
            self.transfer_to_vault(
                &self.trader_ata_y,
                &self.vault_y,
                &self.mint_y,
                amount_in,
            )?;
            // Transfer X from vault to trader
            self.transfer_from_vault(
                &self.vault_x,
                &self.trader_ata_x,
                &self.mint_x,
                amount_out,
            )?;
        }

        Ok(())
    }

    fn transfer_to_vault(
        &self,
        from: &InterfaceAccount<'info, TokenAccount>,
        to: &InterfaceAccount<'info, TokenAccount>,
        mint: &InterfaceAccount<'info, Mint>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: self.trader.to_account_info(),
            mint: mint.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer_checked(cpi_ctx, amount, mint.decimals)
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
