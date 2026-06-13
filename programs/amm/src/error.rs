use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Fee must be between 0 and 10000 basis points")]
    InvalidFee,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Pool reserves are zero")]
    PoolEmpty,
    #[msg("Overflow occurred during calculation")]
    Overflow,
}
