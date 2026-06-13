use crate::error::AmmError;
use anchor_lang::prelude::*;

/// Calculate swap output using constant product formula: x * y = k
/// After applying fee, computes: dy = (y * dx_after_fee) / (x + dx_after_fee)
pub fn calculate_swap_output(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_bps: u16,
) -> Result<u64> {
    require!(amount_in > 0, AmmError::ZeroAmount);
    require!(reserve_in > 0 && reserve_out > 0, AmmError::PoolEmpty);

    // Deduct fee: amount_after_fee = amount_in * (10000 - fee_bps) / 10000
    let amount_after_fee = (amount_in as u128)
        .checked_mul((10000 - fee_bps) as u128)
        .ok_or(AmmError::Overflow)?
        / 10000u128;

    // dy = (y * dx) / (x + dx)
    let numerator = (reserve_out as u128)
        .checked_mul(amount_after_fee)
        .ok_or(AmmError::Overflow)?;

    let denominator = (reserve_in as u128)
        .checked_add(amount_after_fee)
        .ok_or(AmmError::Overflow)?;

    let amount_out = numerator
        .checked_div(denominator)
        .ok_or(AmmError::Overflow)?;

    Ok(amount_out as u64)
}
