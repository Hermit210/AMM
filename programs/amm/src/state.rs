use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub seed: u64,
    pub fee: u16, // basis points (e.g., 300 = 3%)
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub lp_bump: u8,
    pub bump: u8,
}

impl Config {
    pub const LEN: usize = 8 + 8 + 2 + 32 + 32 + 1 + 1;
}
