# Anchor AMM

A constant-product automated market maker (AMM) built with Anchor on Solana. Supports token swaps, liquidity deposits, and withdrawals using the x*y=k invariant.

## Program ID

`DJnx5rDpK9tyuCyRt3UtyHmz5o2mK1sTn72FvJPn7dsn`

## Architecture

### State

**Config** — one PDA per pool, stores all pool metadata:

| Field | Type | Description |
|---|---|---|
| `seed` | `u64` | Unique seed used to derive the config PDA |
| `authority` | `Option<Pubkey>` | Optional admin authority |
| `mint_x` | `Pubkey` | Token X mint |
| `mint_y` | `Pubkey` | Token Y mint |
| `fee` | `u16` | Swap fee in basis points (e.g. 30 = 0.3%) |
| `locked` | `bool` | If true, deposits/swaps are paused |
| `config_bump` | `u8` | PDA bump for the config account |
| `lp_bump` | `u8` | PDA bump for the LP mint |

### PDAs

| Account | Seeds |
|---|---|
| Config | `["config", seed.to_le_bytes()]` |
| LP Mint | `["lp", config.key()]` |
| Vault X | Associated token account: authority = config, mint = mint_x |
| Vault Y | Associated token account: authority = config, mint = mint_y |

## Instructions

### `initialize(seed, fee, authority)`

Creates a new AMM pool. Initialises the config PDA, LP mint, and both vault token accounts.

- `seed` — unique u64 to allow multiple pools for the same token pair
- `fee` — swap fee in basis points
- `authority` — optional admin that can lock/unlock the pool

### `deposit(amount, max_x, max_y)`

Deposits liquidity into the pool and mints LP tokens to the user.

- `amount` — number of LP tokens to mint (determines how much liquidity to provide)
- `max_x` / `max_y` — slippage limits: maximum tokens X/Y the user is willing to deposit

On first deposit (empty pool), the ratio is set by `max_x` and `max_y` directly.

### `withdraw(amount, min_x, min_y)`

Burns LP tokens and returns a proportional share of the pool reserves to the user.

- `amount` — LP tokens to burn
- `min_x` / `min_y` — slippage limits: minimum tokens X/Y the user expects to receive

### `swap(is_x, amount_in, min_amount_out)`

Swaps one token for the other using the constant-product curve with fee deduction.

- `is_x` — if `true`, swaps X → Y; if `false`, swaps Y → X
- `amount_in` — amount of input token to swap
- `min_amount_out` — minimum output expected (slippage protection)

## Tech Stack

- [Anchor 1.0.1](https://github.com/coral-xyz/anchor)
- [anchor-spl 1.0.1](https://github.com/coral-xyz/anchor) — `token_interface` (supports SPL Token and Token-2022)
- [constant-product-curve](https://github.com/deanmlittle/constant-product-curve) — x*y=k math
- [LiteSVM 0.10.0](https://github.com/LiteSVM/litesvm) — fast in-process Solana VM for tests

## Tests

Tests are written using LiteSVM (no validator required) and cover all four instructions end-to-end.

```
test test_initialize ... ok
test test_deposit    ... ok
test test_withdraw   ... ok
test test_swap       ... ok
```

### Run tests

```bash
# Build the program first
cargo build-sbf

# Run all tests
cargo test
```

## Project Structure

```
programs/anchor-amm-q2-2026/
├── src/
│   ├── lib.rs                  # Program entrypoint, instruction routing
│   ├── state.rs                # Config account definition
│   ├── error.rs                # Custom error codes
│   ├── constants.rs
│   └── instructions/
│       ├── initialize.rs       # Pool initialisation
│       ├── deposit.rs          # Add liquidity
│       ├── withdraw.rs         # Remove liquidity
│       └── swap.rs             # Token swap
└── tests/
    ├── tests.rs                # Integration tests
    └── ix_handlers/            # Instruction builders for tests
        ├── init.rs
        ├── deposit.rs
        ├── withdraw.rs
        └── swap.rs
```
