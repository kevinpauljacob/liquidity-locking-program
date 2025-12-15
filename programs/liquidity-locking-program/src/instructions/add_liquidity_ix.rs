use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use crate::context::damm_v2::{cpi::accounts::AddLiquidity, cpi::add_liquidity, AddLiquidityParameters};

#[derive(Accounts)]
pub struct DammV2AddLiquidity<'info> {
    /// CHECK: Meteora pool account, writable in add_liquidity
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// Position account, writable in add_liquidity
    /// CHECK: Meteora position account
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// User's token A account, writable in add_liquidity
    #[account(mut)]
    pub token_a_account: Account<'info, TokenAccount>, 

    /// User's token B account, writable in add_liquidity
    #[account(mut)]
    pub token_b_account: Account<'info, TokenAccount>, 

    /// Token A vault, writable in add_liquidity
    /// CHECK: Meteora token A vault
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,

    /// Token B vault, writable in add_liquidity
    /// CHECK: Meteora token B vault
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,

    /// Token A mint, readonly in add_liquidity
    pub token_a_mint: Account<'info, Mint>,

    /// Token B mint, readonly in add_liquidity
    pub token_b_mint: Account<'info, Mint>,

    /// Position NFT account, readonly in add_liquidity
    /// CHECK: Meteora position NFT account
    pub position_nft_account: UncheckedAccount<'info>,

    /// Owner, signer in add_liquidity
    pub owner: Signer<'info>,

    /// Token A program, readonly in add_liquidity
    /// CHECK: Token A program
    pub token_a_program: UncheckedAccount<'info>,

    /// Token B program, readonly in add_liquidity
    /// CHECK: Token B program
    pub token_b_program: UncheckedAccount<'info>,

    /// Event authority, readonly in add_liquidity
    /// CHECK: Event authority
    pub event_authority: UncheckedAccount<'info>,

    /// Meteora program, readonly in add_liquidity
    /// CHECK: Meteora program
    pub damm_program: UncheckedAccount<'info>,
}

pub fn handle_add_liquidity(ctx: Context<DammV2AddLiquidity>) -> Result<()> {
    let liquidity_delta = 1000000; // Example value; adjust as needed
    let add_params = AddLiquidityParameters {
        liquidity_delta,
        token_a_amount_threshold: 1000,
        token_b_amount_threshold: 1000,
    };
    let add_accounts = AddLiquidity {
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        token_a_account: ctx.accounts.token_a_account.to_account_info(),  
        token_b_account: ctx.accounts.token_b_account.to_account_info(),  
        token_a_vault: ctx.accounts.token_a_vault.to_account_info(),
        token_b_vault: ctx.accounts.token_b_vault.to_account_info(),
        token_a_mint: ctx.accounts.token_a_mint.to_account_info(),
        token_b_mint: ctx.accounts.token_b_mint.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        owner: ctx.accounts.owner.to_account_info(),
        token_a_program: ctx.accounts.token_a_program.to_account_info(),
        token_b_program: ctx.accounts.token_b_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.damm_program.to_account_info(),
    };
    add_liquidity(CpiContext::new(ctx.accounts.damm_program.to_account_info(), add_accounts), add_params)?;
    Ok(())
}