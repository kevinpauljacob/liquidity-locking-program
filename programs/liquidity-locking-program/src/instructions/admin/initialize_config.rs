use anchor_lang::prelude::*;
use crate::states::Config;
use crate::constants::seeds;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    // Config PDA (initialize once)
    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<Config>(),
        seeds = [seeds::CONFIG_SEED],
        bump,
    )]
    pub config: Account<'info, Config>,

    // Admin (signer, will be stored in config)
    #[account(mut)]
    pub admin: Signer<'info>,

    // System program for PDA creation
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_config(
    ctx: Context<InitializeConfig>,
    pool_id: Pubkey,
    fee_bps: u16,
    slf_mint: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.pool_id = pool_id;
    config.admin = ctx.accounts.admin.key();
    config.fee_bps = fee_bps;
    config.slf_mint = slf_mint;

    Ok(())
}