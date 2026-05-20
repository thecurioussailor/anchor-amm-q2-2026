pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("DJnx5rDpK9tyuCyRt3UtyHmz5o2mK1sTn72FvJPn7dsn");

#[program]
pub mod anchor_amm_q2_2026 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
