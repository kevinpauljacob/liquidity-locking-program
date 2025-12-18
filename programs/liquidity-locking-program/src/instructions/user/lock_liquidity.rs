use anchor_lang::prelude::*;
use anchor_spl::token::{self};
use anchor_spl::token_2022;
use anchor_spl::associated_token::AssociatedToken;
use crate::context::damm_v2::{cpi::accounts::CreatePosition, cpi::create_position, cpi::accounts::AddLiquidity, cpi::add_liquidity, AddLiquidityParameters};
use crate::states::{LockAccount, LockStatus, Config};
use crate::constants::{seeds, SLERF_USDC_POOL, METEORA_PROGRAM_ID, POOL_AUTHORITY};
use crate::errors::ErrorCode; 

#[derive(Accounts)]
pub struct LockLiquidity<'info> {
    // Config PDA
    #[account(
        seeds = [seeds::CONFIG_SEED],
        bump,
    )]
    pub config: Account<'info, Config>,

    // Escrow Authority PDA (signer for ATA)
    #[account(
        seeds = [seeds::ESCROW_AUTHORITY_SEED],
        bump,
    )]
    pub escrow_authority: SystemAccount<'info>,

    // Lock Account PDA (new)
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<LockAccount>(),
        seeds = [seeds::LOCK_SEED, user.key().as_ref(), position_nft_mint.key().as_ref()],
        bump,
    )]
    pub lock_account: Account<'info, LockAccount>,

    // User's token accounts (for add_liquidity)
    #[account(mut)]
    pub user_token_a: Account<'info, token::TokenAccount>, // SLERF
    #[account(mut)]
    pub user_token_b: Account<'info, token::TokenAccount>, // USDC

    // Position NFT mint (new, signer for create_position)
    #[account(mut)]
    pub position_nft_mint: Signer<'info>,

    // Position NFT ATA (user's, for transfer)
    /// CHECK: Position NFT ATA (created by create_position CPI)
    #[account(
        mut
    )]
    pub position_nft_account: UncheckedAccount<'info>,

    // Escrow ATA for NFT (program-owned) - Now unchecked, created in handler
    /// CHECK: Escrow NFT ATA (created manually in handler)
    #[account(mut)]
    pub escrow_nft_account: UncheckedAccount<'info>,

    // Meteora accounts
    /// CHECK: Pool account
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,
    /// CHECK: Position account
    #[account(mut)]
    pub position: UncheckedAccount<'info>,
    /// CHECK: Pool authority
    #[account(address = POOL_AUTHORITY)]
    pub pool_authority: UncheckedAccount<'info>,
    /// CHECK: Token A vault
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,
    /// CHECK: Token B vault
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,
    /// CHECK: Token A mint (SLERF)
    pub token_a_mint: UncheckedAccount<'info>,
    /// CHECK: Token B mint (USDC)
    pub token_b_mint: UncheckedAccount<'info>,
    /// CHECK: Event authority
    #[account(
        seeds = [seeds::EVENT_AUTHORITY_SEED],
        bump,
        seeds::program = METEORA_PROGRAM_ID,
    )]
    pub event_authority: UncheckedAccount<'info>,

    // Programs
    pub token_program: Program<'info, token_2022::Token2022>,  // CHANGE: Use Token2022
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    /// CHECK: Meteora program
    #[account(address = METEORA_PROGRAM_ID)]
    pub damm_program: UncheckedAccount<'info>,

    /// CHECK: Token A program (SPL Token)
    pub token_a_program: UncheckedAccount<'info>,
    /// CHECK: Token B program (SPL Token)
    pub token_b_program: UncheckedAccount<'info>,

    // User (signer, payer)
    #[account(mut)]
    pub user: Signer<'info>,

    // Clock for timestamps
    pub clock: Sysvar<'info, Clock>,
}

pub fn handle_lock_liquidity(
    ctx: Context<LockLiquidity>,
    liquidity_delta: u128,
    duration_months: u8,
) -> Result<()> {
    // Validate config pool_id
    if ctx.accounts.config.pool_id != SLERF_USDC_POOL {
        return err!(ErrorCode::InvalidPool);
    }

    // Validate duration
    let duration_seconds = match duration_months {
        3 => 3 * 30 * 24 * 3600, // ~3 months
        6 => 6 * 30 * 24 * 3600, // ~6 months
        12 => 12 * 30 * 24 * 3600, // ~12 months
        _ => return err!(ErrorCode::InvalidDuration),
    };

    let now = ctx.accounts.clock.unix_timestamp as u64;
    let lock_end = now + duration_seconds;

    // Derive position PDA (for reference in LockAccount)
    let position_pda = Pubkey::find_program_address(
        &[seeds::POSITION_SEED, ctx.accounts.position_nft_mint.key().as_ref()],
        &METEORA_PROGRAM_ID,
    ).0;

    // CPI: Create position
    let create_accounts = CreatePosition {
        owner: ctx.accounts.user.to_account_info(),
        position_nft_mint: ctx.accounts.position_nft_mint.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        pool_authority: ctx.accounts.pool_authority.to_account_info(),
        payer: ctx.accounts.user.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.damm_program.to_account_info(),
    };
    create_position(CpiContext::new(ctx.accounts.damm_program.to_account_info(), create_accounts))?;

    // ATA creation for escrow (SPL Token)
    anchor_spl::associated_token::create(
        CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.user.to_account_info(),
                associated_token: ctx.accounts.escrow_nft_account.to_account_info(),
                authority: ctx.accounts.escrow_authority.to_account_info(),
                mint: ctx.accounts.position_nft_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),  // SPL Token
            },
        ),
    )?;

    // CPI: Add liquidity
    let add_params = AddLiquidityParameters {
        liquidity_delta,
        token_a_amount_threshold: u64::MAX, // Allow any amount
        token_b_amount_threshold: u64::MAX,
    };
    let add_accounts = AddLiquidity {
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        token_a_account: ctx.accounts.user_token_a.to_account_info(),
        token_b_account: ctx.accounts.user_token_b.to_account_info(),
        token_a_vault: ctx.accounts.token_a_vault.to_account_info(),
        token_b_vault: ctx.accounts.token_b_vault.to_account_info(),
        token_a_mint: ctx.accounts.token_a_mint.to_account_info(),
        token_b_mint: ctx.accounts.token_b_mint.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        owner: ctx.accounts.user.to_account_info(),
        token_a_program: ctx.accounts.token_a_program.to_account_info(),
        token_b_program: ctx.accounts.token_b_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.damm_program.to_account_info(),
    };
    add_liquidity(CpiContext::new(ctx.accounts.damm_program.to_account_info(), add_accounts), add_params)?;

    // Transfer NFT (Token2022)
token_2022::transfer_checked(
    CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token_2022::TransferChecked {
            from: ctx.accounts.position_nft_account.to_account_info(),
            mint: ctx.accounts.position_nft_mint.to_account_info(),
            to: ctx.accounts.escrow_nft_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    ),
    1,   // NFT amount
    0,   // Decimals
)?;

    // Create Lock Account
    ctx.accounts.lock_account.set_inner(LockAccount {
        user: ctx.accounts.user.key(),
        position_nft_mint: ctx.accounts.position_nft_mint.key(),
        position_pda,
        lock_start: now,
        lock_end,
        liquidity_locked: liquidity_delta,
        duration_months,
        status: LockStatus::Active,
        total_rewards_earned: 0,
        rewards_claimed: 0,
        last_claim_time: now,
    });

    Ok(())
}