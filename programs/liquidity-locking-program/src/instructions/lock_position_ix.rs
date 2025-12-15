// filepath: /Users/kevinjacob/Developer/projects/liquidity-locking-program/programs/liquidity-locking-program/src/instructions/lock_position.rs
use anchor_lang::prelude::*;
use crate::context::damm_v2::{cpi::accounts::LockPosition, cpi::lock_position, VestingParameters};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct DammV2LockPosition<'info> {
    /// CHECK: Meteora pool account, readonly in lock_position
    pub pool: UncheckedAccount<'info>,

    /// Position account, writable in lock_position
    /// CHECK: Meteora position account
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// Vesting account, writable signer in lock_position
    /// CHECK: Meteora vesting account
    #[account(mut)]
    pub vesting: UncheckedAccount<'info>,

    /// Position NFT account, readonly in lock_position
    /// CHECK: Meteora position NFT account
    pub position_nft_account: UncheckedAccount<'info>,

    /// Owner, signer in lock_position
    pub owner: Signer<'info>,

    /// Payer, writable signer in lock_position
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program, readonly in lock_position
    pub system_program: Program<'info, System>,

    /// Event authority, readonly in lock_position
    /// CHECK: Event authority
    pub event_authority: UncheckedAccount<'info>,

    /// Meteora program, readonly in lock_position
    /// CHECK: Meteora program
    pub damm_program: UncheckedAccount<'info>,
}

pub fn handle_lock_position(ctx: Context<DammV2LockPosition>, duration_months: u8) -> Result<()> {
    let (cliff_point, period_frequency, number_of_period) = match duration_months {
        3 => (Some(7776000), 7776000 / 4, 4),
        6 => (Some(15552000), 15552000 / 4, 4),
        12 => (Some(31104000), 31104000 / 4, 4),
        _ => return Err(ErrorCode::InvalidDuration.into()),
    };
    let vesting_params = VestingParameters {
        cliff_point,
        period_frequency,
        cliff_unlock_liquidity: 0,
        liquidity_per_period: 0,
        number_of_period,
    };
    let lock_accounts = LockPosition {
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        vesting: ctx.accounts.vesting.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        owner: ctx.accounts.owner.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.damm_program.to_account_info(),
    };
    lock_position(CpiContext::new(ctx.accounts.damm_program.to_account_info(), lock_accounts), vesting_params)?;
    Ok(())
}