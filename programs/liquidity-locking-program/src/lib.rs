use anchor_lang::prelude::*;
use instructions::lock_liquidity::*;

pub mod instructions;
pub mod context;
pub mod states;
pub mod errors;
pub mod constants;
pub use context::*;
pub use constants::*;

declare_id!("DtnLiyCepzKfNiyFHBHEqabhrNe65tx8FPxLWQeh6JeC");

#[program]
pub mod liquidity_locking_program {
    use super::*;

    pub fn damm_v2_lock_liquidity(
        ctx: Context<DynamicAmmLockLiquidity>,
        duration_months: u8,
    ) -> Result<()> {
        handle_lock_liquidity(ctx, duration_months)
    }
}
