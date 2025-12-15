use anchor_lang::prelude::*;
use instructions::{
    create_position_ix::*,
    add_liquidity_ix::*,
    lock_position_ix::*,
};

pub mod instructions;
pub mod context;
pub mod states;
pub mod errors;
pub mod constants;

declare_id!("DtnLiyCepzKfNiyFHBHEqabhrNe65tx8FPxLWQeh6JeC");

#[program]
pub mod liquidity_locking_program {
    use super::*;

    pub fn create_position_ix(ctx: Context<DammV2CreatePosition>) -> Result<()> {
        handle_create_position(ctx)
    }

    pub fn add_liquidity_ix(ctx: Context<DammV2AddLiquidity>, liquidity_delta: u128) -> Result<()> {
        handle_add_liquidity(ctx, liquidity_delta)
    }

    pub fn lock_position_ix(
        ctx: Context<DammV2LockPosition>,
        params: crate::context::damm_v2::VestingParameters,
    ) -> Result<()> {
        handle_lock_position(ctx, params)
    }
}
