use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::damm_v2::{cpi::accounts::CreatePosition, cpi::accounts::AddLiquidity, cpi::accounts::LockPosition, cpi::create_position, cpi::add_liquidity, cpi::lock_position, AddLiquidityParameters, VestingParameters};
use crate::errors::ErrorCode;
use crate::constants::*;

#[derive(Accounts)]
pub struct DynamicAmmLockLiquidity<'info> {
    /// CHECK: Meteora pool account, writable in create_position and add_liquidity
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// User's token A account (e.g., USDC), writable in add_liquidity
    #[account(mut)]
    pub user_token_a_account: Account<'info, TokenAccount>,

    /// User's token B account (e.g., SLERF), writable in add_liquidity
    #[account(mut)]
    pub user_token_b_account: Account<'info, TokenAccount>,

    /// Token A mint, readonly in add_liquidity
    pub token_a_mint: Account<'info, Mint>,

    /// Token B mint, readonly in add_liquidity
    pub token_b_mint: Account<'info, Mint>,

    /// Token A vault, writable in add_liquidity
    /// CHECK: Meteora token A vault
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,

    /// Token B vault, writable in add_liquidity
    /// CHECK: Meteora token B vault
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,

    /// Pool authority, fixed address from IDL
    /// CHECK: Meteora pool authority
    #[account(address = POOL_AUTHORITY)]
    pub pool_authority: UncheckedAccount<'info>,

    /// Position NFT mint, writable signer in create_position
    #[account(mut)]
    pub position_nft_mint: Signer<'info>, 

    /// Position NFT account, writable in create_position
    /// CHECK: Meteora position NFT account
    #[account(mut)]
    pub position_nft_account: UncheckedAccount<'info>,

    /// Position account, writable in create_position and add_liquidity
    /// CHECK: Meteora position account
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// Vesting account, writable signer in lock_position
    /// CHECK: Meteora vesting account
    #[account(mut,)]
    pub vesting: UncheckedAccount<'info>,

    /// Payer, signer in create_position, add_liquidity, and lock_position
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Token program, readonly in create_position
    /// CHECK: Token program
    pub token_program: UncheckedAccount<'info>,

    /// Token A program, readonly in add_liquidity
    /// CHECK: Token A program
    pub token_a_program: UncheckedAccount<'info>,

    /// Token B program, readonly in add_liquidity
    /// CHECK: Token B program
    pub token_b_program: UncheckedAccount<'info>,

    /// Event authority, readonly in all CPIs
    /// CHECK: Event authority
    pub event_authority: UncheckedAccount<'info>,

    /// System program, readonly in create_position and lock_position
    pub system_program: Program<'info, System>,

    /// Associated token program, readonly in add_liquidity (for ATA creation)
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Meteora program, readonly in all CPIs
    /// CHECK: Meteora program
    pub damm_program: UncheckedAccount<'info>,
}

pub fn handle_lock_liquidity(ctx: Context<DynamicAmmLockLiquidity>, duration_months: u8) -> Result<()> {
    // Validate vesting duration (no Pool deserialization needed; Meteora CPIs validate everything)
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

    // 1. Create position
    let create_accounts = CreatePosition {
        owner: ctx.accounts.payer.to_account_info(),
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

    // 2. Add liquidity
    let liquidity_delta = 1000000;
    let add_params = AddLiquidityParameters {
        liquidity_delta,
        token_a_amount_threshold: 1000,
        token_b_amount_threshold: 1000,
    };
    let add_accounts = AddLiquidity {
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        token_a_account: ctx.accounts.user_token_a_account.to_account_info(),
        token_b_account: ctx.accounts.user_token_b_account.to_account_info(),
        token_a_vault: ctx.accounts.token_a_vault.to_account_info(),
        token_b_vault: ctx.accounts.token_b_vault.to_account_info(),
        token_a_mint: ctx.accounts.token_a_mint.to_account_info(),
        token_b_mint: ctx.accounts.token_b_mint.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        owner: ctx.accounts.payer.to_account_info(),
        token_a_program: ctx.accounts.token_a_program.to_account_info(),
        token_b_program: ctx.accounts.token_b_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.damm_program.to_account_info(),
    };
    add_liquidity(CpiContext::new(ctx.accounts.damm_program.to_account_info(), add_accounts), add_params)?;

    // 3. Lock position
    let lock_accounts = LockPosition {
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        vesting: ctx.accounts.vesting.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        owner: ctx.accounts.payer.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.damm_program.to_account_info(),
    };
    lock_position(CpiContext::new(ctx.accounts.damm_program.to_account_info(), lock_accounts), vesting_params)?;

    Ok(())
}