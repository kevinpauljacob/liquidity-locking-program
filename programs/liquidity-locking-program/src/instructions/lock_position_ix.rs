use anchor_lang::prelude::*;
use crate::context::damm_v2::{cpi::accounts::LockPosition, cpi::lock_position, VestingParameters};

#[derive(Accounts)]
pub struct DammV2LockPosition<'info> {
    /// CHECK: Meteora pool
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Meteora position
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// Must be a transaction signer (matches successful tx)
    #[account(mut)]
    pub vesting: Signer<'info>,

    /// CHECK: Position NFT token account
    pub position_nft_account: UncheckedAccount<'info>,

    pub owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: Meteora event authority PDA
    pub event_authority: UncheckedAccount<'info>,

    /// CHECK: Meteora program
    pub damm_program: UncheckedAccount<'info>,
}

// NOTE: accept full params (so tests can match Meteora's expectations)
pub fn handle_lock_position(
    ctx: Context<DammV2LockPosition>,
    params: VestingParameters,
) -> Result<()> {
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

    lock_position(
        CpiContext::new(ctx.accounts.damm_program.to_account_info(), lock_accounts),
        params,
    )
}