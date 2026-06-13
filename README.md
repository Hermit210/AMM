# Solana AMM Program

A Solana-based Automated Market Maker (AMM) smart contract built with Anchor that enables token swaps, liquidity provision, and fee management.

## Features

- **Initialize Pool**: Create a new AMM pool with two tokens and configurable fees
- **Deposit Liquidity**: Add tokens to the pool and receive LP tokens
- **Withdraw Liquidity**: Remove tokens from the pool by burning LP tokens
- **Swap Tokens**: Exchange one token for another using the constant product formula (x * y = k)
- **Configurable Fees**: Set swap fees in basis points (e.g., 300 = 3%)

## Project Structure

```
programs/amm/src/
├── lib.rs              # Main program entry point
├── state.rs            # AMM pool configuration state
├── error.rs            # Custom error types
├── helpers.rs          # Utility functions (swap calculations)
└── contexts/
    ├── mod.rs          # Context module exports
    ├── initialize.rs   # Pool initialization context
    ├── deposit.rs      # Liquidity deposit context
    ├── withdraw.rs     # Liquidity withdrawal context
    └── swap.rs         # Token swap context
```

## Instructions

### Initialize
Creates a new AMM pool with two token mints.

**Parameters:**
- `seed`: Unique seed for deterministic pool PDA
- `fee`: Swap fee in basis points (0-10000)

### Deposit
Adds tokens to the pool and mints LP tokens to the depositor.

**Parameters:**
- `amount_x`: Amount of token X to deposit
- `amount_y`: Amount of token Y to deposit
- `min_lp`: Minimum LP tokens to receive (slippage protection)

### Withdraw
Removes tokens from the pool by burning LP tokens.

**Parameters:**
- `lp_amount`: Amount of LP tokens to burn
- `min_x`: Minimum token X to receive (slippage protection)
- `min_y`: Minimum token Y to receive (slippage protection)

### Swap
Exchanges one token for another.

**Parameters:**
- `amount_in`: Amount of input token
- `min_amount_out`: Minimum output tokens (slippage protection)
- `is_x`: True to swap X → Y, False to swap Y → X

## Building

```bash
cargo build --release --features init-if-needed
```

## Accounts Used

- **Config**: Stores AMM pool state (seeds, fee, mints, bumps)
- **LP Mint**: Token mint for liquidity provider tokens
- **Vault X/Y**: Token accounts holding the pool reserves
- **ATA X/Y**: User's associated token accounts

## Security Considerations

- Slippage protection on all user operations
- Checked arithmetic to prevent overflow attacks
- PDA-based account derivation for deterministic addresses
- LP token supply tracks liquidity accurately

## Testing

Tests are configured but implementation can be extended with TypeScript client tests using the package.json setup.

## License

MIT
