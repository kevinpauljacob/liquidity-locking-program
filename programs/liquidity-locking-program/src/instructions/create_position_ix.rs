// filepath: /Users/kevinjacob/Developer/projects/liquidity-locking-program/programs/liquidity-locking-program/src/instructions/create_position.rs
use anchor_lang::prelude::*;
use crate::context::damm_v2::{cpi::accounts::CreatePosition, cpi::create_position};
use crate::constants::*;

#[derive(Accounts)]
pub struct DammV2CreatePosition<'info> {
    /// Owner, signer in create_position (same as payer in this case)
    pub owner: Signer<'info>,  // #1

    /// Position NFT mint, writable signer in create_position
    #[account(mut)]
    pub position_nft_mint: Signer<'info>,  // #2

    /// Position NFT account, writable in create_position
    /// CHECK: Meteora position NFT account
    #[account(mut)]
    pub position_nft_account: UncheckedAccount<'info>,  // #3

    /// CHECK: Meteora pool account, writable in create_position
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,  // #4

    /// Position account, writable in create_position
    /// CHECK: Meteora position account
    #[account(mut)]
    pub position: UncheckedAccount<'info>,  // #5

    /// Pool authority, fixed address from IDL (readonly)
    /// CHECK: Meteora pool authority
    pub pool_authority: UncheckedAccount<'info>,  // #6

    /// Payer, signer in create_position
    #[account(mut)]
    pub payer: Signer<'info>,  // #7

    /// Token program, readonly in create_position
    /// CHECK: Token program
    pub token_program: UncheckedAccount<'info>,  // #8

    /// System program, readonly in create_position
    pub system_program: Program<'info, System>,  // #9

    /// Event authority, readonly in create_position
    /// CHECK: Event authority
    pub event_authority: UncheckedAccount<'info>,  // #10

    /// Meteora program, readonly in create_position
    /// CHECK: Meteora program
    pub damm_program: UncheckedAccount<'info>,  // #11
}

pub fn handle_create_position(ctx: Context<DammV2CreatePosition>) -> Result<()> {
    let create_accounts = CreatePosition {
        owner: ctx.accounts.owner.to_account_info(), 
        position_nft_mint: ctx.accounts.position_nft_mint.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        pool_authority: ctx.accounts.pool_authority.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.damm_program.to_account_info(),
    };
    create_position(CpiContext::new(ctx.accounts.damm_program.to_account_info(), create_accounts))?;
    Ok(())
}