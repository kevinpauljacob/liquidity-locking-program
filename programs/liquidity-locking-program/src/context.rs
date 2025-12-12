use anchor_lang::prelude::*;
use crate::constants::*;  // Added import

// Manual damm_v2 CPI module
pub mod damm_v2 {
    use super::*;

    pub const ID: Pubkey = METEORA_PROGRAM_ID; 

    pub mod cpi {
        use super::*;
        use anchor_lang::solana_program::program::invoke_signed;

        // Single accounts module with all structs
        pub mod accounts {
            use super::*;

            #[derive(Accounts)]
            pub struct CreatePosition<'info> {
                /// CHECK: Owner account, signer for Meteora create_position
                pub owner: AccountInfo<'info>,                    // signer
                /// CHECK: Position NFT mint account, writable signer for Meteora create_position
                pub position_nft_mint: AccountInfo<'info>,        // writable, signer
                /// CHECK: Position NFT associated token account, writable for Meteora create_position
                pub position_nft_account: AccountInfo<'info>,     // writable
                /// CHECK: Pool account, writable for Meteora create_position
                pub pool: AccountInfo<'info>,                     // writable
                /// CHECK: Position account, writable for Meteora create_position
                pub position: AccountInfo<'info>,                 // writable
                /// CHECK: Pool authority PDA, readonly for Meteora create_position
                pub pool_authority: AccountInfo<'info>,           // fixed
                /// CHECK: Payer account, writable signer for Meteora create_position
                pub payer: AccountInfo<'info>,                    // writable, signer
                /// CHECK: Token program, readonly for Meteora create_position
                pub token_program: AccountInfo<'info>,            // fixed
                /// CHECK: System program, readonly for Meteora create_position
                pub system_program: AccountInfo<'info>,           // fixed
                /// CHECK: Event authority PDA, readonly for Meteora create_position
                pub event_authority: AccountInfo<'info>,          // PDA
                /// CHECK: Meteora program, readonly
                pub program: AccountInfo<'info>,                  // fixed
            }

            #[derive(Accounts)]
            pub struct LockPosition<'info> {
                /// CHECK: Pool account, readonly for Meteora lock_position
                pub pool: AccountInfo<'info>,
                /// CHECK: Position account, readonly for Meteora lock_position
                pub position: AccountInfo<'info>,
                /// CHECK: Vesting account, writable signer for Meteora lock_position
                pub vesting: AccountInfo<'info>,
                /// CHECK: Position NFT account, readonly for Meteora lock_position
                pub position_nft_account: AccountInfo<'info>,
                /// CHECK: Owner, signer for Meteora lock_position
                pub owner: AccountInfo<'info>,
                /// CHECK: Payer, writable signer for Meteora lock_position
                pub payer: AccountInfo<'info>,
                /// CHECK: System program, readonly for Meteora lock_position
                pub system_program: AccountInfo<'info>,
                /// CHECK: Event authority, readonly for Meteora lock_position
                pub event_authority: AccountInfo<'info>,
                /// CHECK: Meteora program, readonly
                pub program: AccountInfo<'info>,
            }

            #[derive(Accounts)]
            pub struct AddLiquidity<'info> {
                /// CHECK: Pool account, writable for Meteora add_liquidity
                pub pool: AccountInfo<'info>,                     // writable
                /// CHECK: Position account, writable for Meteora add_liquidity
                pub position: AccountInfo<'info>,                 // writable
                /// CHECK: Token A account, writable for Meteora add_liquidity
                pub token_a_account: AccountInfo<'info>,          // writable
                /// CHECK: Token B account, writable for Meteora add_liquidity
                pub token_b_account: AccountInfo<'info>,          // writable
                /// CHECK: Token A vault, writable for Meteora add_liquidity
                pub token_a_vault: AccountInfo<'info>,            // writable
                /// CHECK: Token B vault, writable for Meteora add_liquidity
                pub token_b_vault: AccountInfo<'info>,            // writable
                /// CHECK: Token A mint, readonly for Meteora add_liquidity
                pub token_a_mint: AccountInfo<'info>,             // readonly
                /// CHECK: Token B mint, readonly for Meteora add_liquidity
                pub token_b_mint: AccountInfo<'info>,             // readonly
                /// CHECK: Position NFT account, readonly for Meteora add_liquidity
                pub position_nft_account: AccountInfo<'info>,     // readonly
                /// CHECK: Owner, signer for Meteora add_liquidity
                pub owner: AccountInfo<'info>,                    // signer
                /// CHECK: Token A program, readonly for Meteora add_liquidity
                pub token_a_program: AccountInfo<'info>,          // readonly
                /// CHECK: Token B program, readonly for Meteora add_liquidity
                pub token_b_program: AccountInfo<'info>,          // readonly
                /// CHECK: Event authority, readonly for Meteora add_liquidity
                pub event_authority: AccountInfo<'info>,          // readonly
                /// CHECK: Meteora program, readonly
                pub program: AccountInfo<'info>,                  // readonly
            }
        }

        // CPI function for create_position
        pub fn create_position<'info>(
            ctx: CpiContext<'_, '_, '_, 'info, accounts::CreatePosition<'info>>,
        ) -> Result<()> {
            let discriminator = [48, 215, 197, 153, 96, 203, 180, 133];
            let data = discriminator.to_vec();

            let ix = anchor_lang::solana_program::instruction::Instruction {
                program_id: ID,
                accounts: vec![
                    AccountMeta::new_readonly(ctx.accounts.owner.key(), true),
                    AccountMeta::new(ctx.accounts.position_nft_mint.key(), true),
                    AccountMeta::new(ctx.accounts.position_nft_account.key(), false),
                    AccountMeta::new(ctx.accounts.pool.key(), false),
                    AccountMeta::new(ctx.accounts.position.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.pool_authority.key(), false),
                    AccountMeta::new(ctx.accounts.payer.key(), true),
                    AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.event_authority.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.program.key(), false),
                ],
                data,
            };
            invoke_signed(
                &ix,
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.position_nft_mint.clone(),
                    ctx.accounts.position_nft_account.clone(),
                    ctx.accounts.pool.clone(),
                    ctx.accounts.position.clone(),
                    ctx.accounts.pool_authority.clone(),
                    ctx.accounts.payer.clone(),
                    ctx.accounts.token_program.clone(),
                    ctx.accounts.system_program.clone(),
                    ctx.accounts.event_authority.clone(),
                    ctx.accounts.program.clone(),
                ],
                &[]
            )?;
            Ok(())
        }

        // CPI function for lock_position
        pub fn lock_position<'info>(
            ctx: CpiContext<'_, '_, '_, 'info, accounts::LockPosition<'info>>,
            params: super::VestingParameters,
        ) -> Result<()> {
            let discriminator = [227, 62, 2, 252, 247, 10, 171, 185];
            let mut data = discriminator.to_vec();
            params.serialize(&mut data)?;

            let ix = anchor_lang::solana_program::instruction::Instruction {
                program_id: ID,
                accounts: vec![
                    AccountMeta::new_readonly(ctx.accounts.pool.key(), false),
                    AccountMeta::new(ctx.accounts.position.key(), false),
                    AccountMeta::new(ctx.accounts.vesting.key(), true),
                    AccountMeta::new_readonly(ctx.accounts.position_nft_account.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.owner.key(), true),
                    AccountMeta::new(ctx.accounts.payer.key(), true),
                    AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.event_authority.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.program.key(), false),
                ],
                data,
            };
            invoke_signed(
                &ix,
                &[
                    ctx.accounts.pool.clone(),
                    ctx.accounts.position.clone(),
                    ctx.accounts.vesting.clone(),
                    ctx.accounts.position_nft_account.clone(),
                    ctx.accounts.owner.clone(),
                    ctx.accounts.payer.clone(),
                    ctx.accounts.system_program.clone(),
                    ctx.accounts.event_authority.clone(),
                    ctx.accounts.program.clone(),
                ],
                &[]
            )?;
            Ok(())
        }

        // CPI function for add_liquidity
        pub fn add_liquidity<'info>(
            ctx: CpiContext<'_, '_, '_, 'info, accounts::AddLiquidity<'info>>,
            params: super::AddLiquidityParameters,
        ) -> Result<()> {
            let discriminator = [181, 157, 89, 67, 143, 182, 52, 72];
            let mut data = discriminator.to_vec();
            params.serialize(&mut data)?;

            let ix = anchor_lang::solana_program::instruction::Instruction {
                program_id: ID,
                accounts: vec![
                    AccountMeta::new(ctx.accounts.pool.key(), false),
                    AccountMeta::new(ctx.accounts.position.key(), false),
                    AccountMeta::new(ctx.accounts.token_a_account.key(), false),
                    AccountMeta::new(ctx.accounts.token_b_account.key(), false),
                    AccountMeta::new(ctx.accounts.token_a_vault.key(), false),
                    AccountMeta::new(ctx.accounts.token_b_vault.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.token_a_mint.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.token_b_mint.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.position_nft_account.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.owner.key(), true),
                    AccountMeta::new_readonly(ctx.accounts.token_a_program.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.token_b_program.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.event_authority.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.program.key(), false),
                ],
                data,
            };
            invoke_signed(
                &ix,
                &[
                    ctx.accounts.pool.clone(),
                    ctx.accounts.position.clone(),
                    ctx.accounts.token_a_account.clone(),
                    ctx.accounts.token_b_account.clone(),
                    ctx.accounts.token_a_vault.clone(),
                    ctx.accounts.token_b_vault.clone(),
                    ctx.accounts.token_a_mint.clone(),
                    ctx.accounts.token_b_mint.clone(),
                    ctx.accounts.position_nft_account.clone(),
                    ctx.accounts.owner.clone(),
                    ctx.accounts.token_a_program.clone(),
                    ctx.accounts.token_b_program.clone(),
                    ctx.accounts.event_authority.clone(),
                    ctx.accounts.program.clone(),
                ],
                &[]
            )?;
            Ok(())
        }

    }

    // AddLiquidityParameters from IDL
    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct AddLiquidityParameters {
        pub liquidity_delta: u128,
        pub token_a_amount_threshold: u64,
        pub token_b_amount_threshold: u64,
    }

    // VestingParameters from IDL
    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct VestingParameters {
        pub cliff_point: Option<u64>,
        pub period_frequency: u64,
        pub cliff_unlock_liquidity: u128,
        pub liquidity_per_period: u128,
        pub number_of_period: u16,
    }
}

