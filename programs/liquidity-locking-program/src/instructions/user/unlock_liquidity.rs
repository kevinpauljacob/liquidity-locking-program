use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self};
use anchor_spl::token_2022::{Token2022};
use anchor_spl::token_interface::TransferChecked;
use crate::context::damm_v2::{cpi::accounts::RemoveLiquidity, cpi::remove_liquidity, cpi::accounts::RemoveAllLiquidity, cpi::remove_all_liquidity, RemoveLiquidityParameters};
use crate::states::{LockAccount, LockStatus};
use crate::constants::{seeds, METEORA_PROGRAM_ID};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct UnlockLiquidity<'info> {
    // Lock Account PDA (mutable for updates)
    #[account(
        mut,
        seeds = [seeds::LOCK_SEED, user.key().as_ref(), position_nft_mint.key().as_ref()],
        bump,
        constraint = lock_account.user == user.key().clone() @ ErrorCode::Unauthorized,
        constraint = lock_account.status == LockStatus::Active @ ErrorCode::LockNotActive,
    )]
    pub lock_account: Account<'info, LockAccount>,

    // Position NFT mint (for validation and ATA mint reference)
    /// CHECK: Position NFT mint
    #[account(address = lock_account.position_nft_mint)]
    pub position_nft_mint: UncheckedAccount<'info>,

    // Escrow Authority PDA (for NFT ATA)
    #[account(
        seeds = [seeds::ESCROW_AUTHORITY_SEED],
        bump,
    )]
    pub escrow_authority: SystemAccount<'info>,

    // User's token accounts (for receiving removed tokens)
    #[account(mut)]
    pub user_token_a: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, token::TokenAccount>,

    // Escrow ATA for NFT (already exists from lock_liquidity)
    /// CHECK: Escrow NFT ATA (Token-2022)
    #[account(mut)]
    pub escrow_nft_account: UncheckedAccount<'info>,

    // User's NFT ATA (create if needed in handler)
    /// CHECK: User NFT ATA (Token-2022)
    #[account(mut)]
    pub user_nft_account: UncheckedAccount<'info>,

    // Meteora accounts
    /// CHECK: Pool
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,
    /// CHECK: Position PDA
    #[account(mut)]
    pub position: UncheckedAccount<'info>,
    /// CHECK: Token A vault
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,
    /// CHECK: Token B vault
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,
    /// CHECK: Token A mint
    pub token_a_mint: UncheckedAccount<'info>,
    /// CHECK: Token B mint
    pub token_b_mint: UncheckedAccount<'info>,
    /// CHECK: Event authority
    #[account(
        seeds = [seeds::EVENT_AUTHORITY_SEED],
        bump,
        seeds::program = METEORA_PROGRAM_ID,
    )]
    pub event_authority: UncheckedAccount<'info>,

    // Programs
    pub token_program: Program<'info, token::Token>,  // For SPL tokens (SLERF/USDC)
    pub token_2022_program: Program<'info, Token2022>,  // For Token-2022 NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    /// CHECK: Meteora program
    #[account(address = METEORA_PROGRAM_ID)]
    pub damm_program: UncheckedAccount<'info>,

    // User (signer)
    #[account(mut)]
    pub user: Signer<'info>,

    // Clock for time check
    pub clock: Sysvar<'info, Clock>,
}

pub fn handle_unlock_liquidity(
    ctx: Context<UnlockLiquidity>,
    liquidity_delta: u128,
) -> Result<()> {
    let now = ctx.accounts.clock.unix_timestamp as u64;

    // Check if lock period has expired
    if now < ctx.accounts.lock_account.lock_end {
        return err!(ErrorCode::LockNotExpired);
    }

    // Validate liquidity_delta
    if liquidity_delta > ctx.accounts.lock_account.liquidity_locked {
        return err!(ErrorCode::InvalidUnlockAmount);
    }

    let is_full_unlock = liquidity_delta == 0 || liquidity_delta == ctx.accounts.lock_account.liquidity_locked;

    // Create user's NFT ATA if it doesn't exist
    if ctx.accounts.user_nft_account.owner == &ctx.accounts.system_program.key() {
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.user.to_account_info(),
                associated_token: ctx.accounts.user_nft_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                mint: ctx.accounts.position_nft_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_2022_program.to_account_info(),
            },
        ))?;
    }

    // Calculate escrow bump manually
    let (_escrow_pda, escrow_bump) = Pubkey::find_program_address(&[seeds::ESCROW_AUTHORITY_SEED], &crate::id());

    // Transfer NFT from escrow to user (Token-2022)
    anchor_spl::token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_2022_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.escrow_nft_account.to_account_info(),
                to: ctx.accounts.user_nft_account.to_account_info(),
                authority: ctx.accounts.escrow_authority.to_account_info(),
                mint: ctx.accounts.position_nft_mint.to_account_info(),
            },
            &[&[seeds::ESCROW_AUTHORITY_SEED, &[escrow_bump]]],
        ),
        1,   // NFT amount
        0,   // Decimals for NFT
    )?;

    if is_full_unlock {
        // Full unlock: Use remove_all_liquidity
        let remove_accounts = RemoveAllLiquidity {
            pool: ctx.accounts.pool.to_account_info(),
            position: ctx.accounts.position.to_account_info(),
            token_a_account: ctx.accounts.user_token_a.to_account_info(),
            token_b_account: ctx.accounts.user_token_b.to_account_info(),
            token_a_vault: ctx.accounts.token_a_vault.to_account_info(),
            token_b_vault: ctx.accounts.token_b_vault.to_account_info(),
            token_a_mint: ctx.accounts.token_a_mint.to_account_info(),
            token_b_mint: ctx.accounts.token_b_mint.to_account_info(),
            position_nft_account: ctx.accounts.user_nft_account.to_account_info(), // Use user's ATA
            owner: ctx.accounts.user.to_account_info(),
            token_a_program: ctx.accounts.token_program.to_account_info(),
            token_b_program: ctx.accounts.token_program.to_account_info(),
            event_authority: ctx.accounts.event_authority.to_account_info(),
            program: ctx.accounts.damm_program.to_account_info(),
        };
        remove_all_liquidity(CpiContext::new(ctx.accounts.damm_program.to_account_info(), remove_accounts), u64::MAX, u64::MAX)?;
    } else {
        // Partial unlock: Use remove_liquidity
        let remove_params = RemoveLiquidityParameters {
            liquidity_delta,
            token_a_amount_threshold: u64::MAX,
            token_b_amount_threshold: u64::MAX,
        };
        let remove_accounts = RemoveLiquidity {
            pool: ctx.accounts.pool.to_account_info(),
            position: ctx.accounts.position.to_account_info(),
            token_a_account: ctx.accounts.user_token_a.to_account_info(),
            token_b_account: ctx.accounts.user_token_b.to_account_info(),
            token_a_vault: ctx.accounts.token_a_vault.to_account_info(),
            token_b_vault: ctx.accounts.token_b_vault.to_account_info(),
            token_a_mint: ctx.accounts.token_a_mint.to_account_info(),
            token_b_mint: ctx.accounts.token_b_mint.to_account_info(),
            position_nft_account: ctx.accounts.user_nft_account.to_account_info(), // Use user's ATA
            owner: ctx.accounts.user.to_account_info(),
            token_a_program: ctx.accounts.token_program.to_account_info(),
            token_b_program: ctx.accounts.token_program.to_account_info(),
            event_authority: ctx.accounts.event_authority.to_account_info(),
            program: ctx.accounts.damm_program.to_account_info(),
        };
        remove_liquidity(CpiContext::new(ctx.accounts.damm_program.to_account_info(), remove_accounts), remove_params)?;
    }

    // Update Lock Account
    let lock_account = &mut ctx.accounts.lock_account;
    if is_full_unlock {
        lock_account.liquidity_locked = 0;
        lock_account.status = LockStatus::Claimed;
    } else {
        lock_account.liquidity_locked -= liquidity_delta;
    }

    Ok(())
}