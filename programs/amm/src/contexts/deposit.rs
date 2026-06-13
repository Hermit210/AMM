use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
    },
};

use crate::error::AmmError;
use crate::state::Config;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

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
        associated_token::authority = depositor,
    )]
    pub depositor_ata_x: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = depositor,
    )]
    pub depositor_ata_y: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = lp_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_ata_lp: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(
        &mut self,
        amount_x: u64,
        amount_y: u64,
        min_lp: u64,
    ) -> Result<()> {
        require!(amount_x > 0 && amount_y > 0, AmmError::ZeroAmount);

        let lp_supply = self.lp_mint.supply;
        let vault_x_amount = self.vault_x.amount;
        let vault_y_amount = self.vault_y.amount;

        // Calculate LP tokens to mint
        let lp_to_mint = if lp_supply == 0 {
            // First deposit: LP tokens = sqrt(amount_x * amount_y)
            let product = (amount_x as u128)
                .checked_mul(amount_y as u128)
                .ok_or(AmmError::Overflow)?;
            isqrt(product)
                .ok_or(AmmError::Overflow)?
                as u64
        } else {
            // Subsequent deposits: LP tokens proportional to contribution
            // lp_tokens = min(amount_x * lp_supply / vault_x, amount_y * lp_supply / vault_y)
            let lp_from_x = (amount_x as u128)
                .checked_mul(lp_supply as u128)
                .ok_or(AmmError::Overflow)?
                .checked_div(vault_x_amount as u128)
                .ok_or(AmmError::Overflow)?;
            let lp_from_y = (amount_y as u128)
                .checked_mul(lp_supply as u128)
                .ok_or(AmmError::Overflow)?
                .checked_div(vault_y_amount as u128)
                .ok_or(AmmError::Overflow)?;
            std::cmp::min(lp_from_x, lp_from_y) as u64
        };

        require!(lp_to_mint >= min_lp, AmmError::SlippageExceeded);

        // Transfer token X from depositor to vault
        self.transfer_to_vault(
            &self.depositor_ata_x,
            &self.vault_x,
            &self.mint_x,
            amount_x,
        )?;

        // Transfer token Y from depositor to vault
        self.transfer_to_vault(
            &self.depositor_ata_y,
            &self.vault_y,
            &self.mint_y,
            amount_y,
        )?;

        // Mint LP tokens to depositor
        self.mint_lp_tokens(lp_to_mint)?;

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
            authority: self.depositor.to_account_info(),
            mint: mint.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer_checked(cpi_ctx, amount, mint.decimals)
    }

    fn mint_lp_tokens(&self, amount: u64) -> Result<()> {
        let seeds = &[
            b"amm",
            self.config.mint_x.as_ref(),
            self.config.mint_y.as_ref(),
            &self.config.seed.to_le_bytes(),
            &[self.config.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: self.lp_mint.to_account_info(),
            to: self.depositor_ata_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        mint_to(cpi_ctx, amount)
    }
}

/// Integer square root using Newton's method
fn isqrt(n: u128) -> Option<u128> {
    if n == 0 {
        return Some(0);
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    Some(x)
}
