use anchor_lang::prelude::*;

// Define supporting structs from IDL
#[repr(C)]
#[derive(Clone, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct BaseFeeStruct {
    pub cliff_fee_numerator: u64,
    pub base_fee_mode: u8,
    pub padding: [u8; 5],
    pub first_factor: u16,
    pub second_factor: [u8; 8],
    pub third_factor: u64,
    pub padding_1: u64,
}

#[repr(C)]
#[derive(Clone, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct DynamicFeeStruct {
    pub initialized: u8,
    pub padding: [u8; 7],
    pub max_volatility_accumulator: u32,
    pub variable_fee_control: u32,
    pub bin_step: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub last_update_timestamp: u64,
    pub bin_step_u128: u128,
    pub sqrt_price_reference: u128,
    pub volatility_accumulator: u128,
    pub volatility_reference: u128,
}

#[repr(C)]
#[derive(Clone, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct PoolFeesStruct {
    pub base_fee: BaseFeeStruct,
    pub protocol_fee_percent: u8,
    pub partner_fee_percent: u8,
    pub referral_fee_percent: u8,
    pub padding_0: [u8; 5],
    pub dynamic_fee: DynamicFeeStruct,
    pub padding_1: [u64; 2],
}

#[repr(C)]
#[derive(Clone, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct RewardInfo {
    pub initialized: u8,
    pub reward_token_flag: u8,
    pub _padding_0: [u8; 6],
    pub _padding_1: [u8; 8],
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub funder: Pubkey,
    pub reward_duration: u64,
    pub reward_duration_end: u64,
    pub reward_rate: u128,
    pub reward_per_token_stored: [u8; 32],
    pub last_update_time: u64,
    pub cumulative_seconds_with_empty_liquidity_reward: u64,
}

// Pool struct from IDL (external, add AccountDeserialize for deserialization)
#[repr(C)]
#[derive(Clone, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct Pool {
    pub pool_fees: PoolFeesStruct,  // Use the corrected PoolFeesStruct
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub whitelisted_vault: Pubkey,
    pub partner: Pubkey,
    pub liquidity: u128,
    pub _padding: u128,  // IDL has 'padding' as u128
    pub protocol_a_fee: u64,
    pub protocol_b_fee: u64,
    pub partner_a_fee: u64,
    pub partner_b_fee: u64,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub sqrt_price: u128,
    pub activation_point: u64,
    pub activation_type: u8,
    pub pool_status: u8,
    pub token_a_flag: u8,
    pub token_b_flag: u8,
    pub collect_fee_mode: u8,
    pub pool_type: u8,
    pub version: u8,
    pub _padding_0: u8,  // IDL has 'padding_0'
    pub fee_a_per_liquidity: [u8; 32],
    pub fee_b_per_liquidity: [u8; 32],
    pub permanent_lock_liquidity: u128,
    pub metrics: PoolMetrics,  // Use the corrected PoolMetrics
    pub creator: Pubkey,
    pub _padding_1: [u64; 6],  // IDL has 'padding_1'
    pub reward_infos: [RewardInfo; 2],  // Use the corrected RewardInfo
}

// Position struct from IDL
#[account]
#[repr(C)]  // Add if needed for bytemuck
pub struct Position {
    pub pool: Pubkey,
    pub nft_mint: Pubkey,
    pub fee_a_per_token_checkpoint: [u8; 32],
    pub fee_b_per_token_checkpoint: [u8; 32],
    pub fee_a_pending: u64,
    pub fee_b_pending: u64,
    pub unlocked_liquidity: u128,
    pub vested_liquidity: u128,
    pub permanent_locked_liquidity: u128,
    pub metrics: PositionMetrics,
    pub reward_infos: [UserRewardInfo; 2],  // Need UserRewardInfo (see below)
    pub padding: [u128; 6],
}

// PositionMetrics from IDL
#[derive(Clone, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct PositionMetrics {
    pub total_claimed_a_fee: u64,
    pub total_claimed_b_fee: u64,
}

// Vesting struct from IDL
#[account]
pub struct Vesting {
    pub position: Pubkey,
    pub cliff_point: u64,
    pub period_frequency: u64,
    pub cliff_unlock_liquidity: u128,
    pub liquidity_per_period: u128,
    pub total_released_liquidity: u128,
    pub number_of_period: u16,
    pub padding: [u8; 14],
    pub padding2: [u128; 4],
}

#[repr(C)]
#[derive(Clone, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct PoolMetrics {
    pub total_lp_a_fee: u128,
    pub total_lp_b_fee: u128,
    pub total_protocol_a_fee: u64,
    pub total_protocol_b_fee: u64,
    pub total_partner_a_fee: u64,
    pub total_partner_b_fee: u64,
    pub total_position: u64,
    pub padding: u64,
}

#[repr(C)]
#[derive(Clone, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct UserRewardInfo {
    pub reward_per_token_checkpoint: [u8; 32],
    pub reward_pendings: u64,
    pub total_claimed_rewards: u64,
}

// LockStatus enum for lock state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum LockStatus {
    Active,
    Unlocked,
    Claimed,
}

// LockAccount PDA for user locks
#[account]
pub struct LockAccount {
    pub user: Pubkey,                    // Owner of the lock
    pub position_nft_mint: Pubkey,       // Meteora NFT mint for the position
    pub position_pda: Pubkey,            // Derived Meteora position PDA
    pub lock_start: u64,                 // Timestamp when lock started
    pub lock_end: u64,                   // Timestamp when lock ends
    pub liquidity_locked: u128,          // Amount of liquidity added
    pub duration_months: u8,             // Lock duration (3/6/12)
    pub status: LockStatus,              // Current lock status
    // New fields for reward vesting
    pub total_rewards_earned: u64,       // Total SLERF rewards claimed from Meteora
    pub rewards_claimed: u64,            // Total vested SLERF transferred to user
    pub last_claim_time: u64,            // Timestamp of last reward claim (init to lock_start)
}

// Config PDA for global program settings
#[account]
pub struct Config {
    pub pool_id: Pubkey,                 // Meteora pool ID
    pub admin: Pubkey,                   // Admin pubkey
    pub fee_bps: u16,                    // Optional program fee in basis points
    pub slf_mint: Pubkey,                // SLERF mint for rewards
}