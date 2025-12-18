// filepath: /Users/kevinjacob/Developer/projects/liquity-locking-program/programs/liqudity-locking-program/src/constants.rs
use anchor_lang::prelude::*;

/// SLERF-USDC Pool Address (hardcoded for this program)
pub const SLERF_USDC_POOL: Pubkey = pubkey!("8yswq8vqEDeTrN2Ez1Bdq2hRekzvFZgMxrdfUKVaNBtQ");

/// Meteora DAMM Program ID
pub const METEORA_PROGRAM_ID: Pubkey = pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

/// Pool Authority Address (fixed from IDL)
pub const POOL_AUTHORITY: Pubkey = pubkey!("HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC");

pub mod seeds {
    pub const POSITION_NFT_MINT_SEED: &[u8] = b"position_nft_mint";
    pub const POSITION_SEED: &[u8] = b"position";
    pub const VESTING_SEED: &[u8] = b"vesting";
    pub const EVENT_AUTHORITY_SEED: &[u8] = b"__event_authority";
    pub const LOCK_SEED: &[u8] = b"lock";                          // For LockAccount PDA: ["lock", user_pubkey, position_nft_mint]
    pub const CONFIG_SEED: &[u8] = b"config";                      // For Config PDA: ["config"]
    pub const ESCROW_AUTHORITY_SEED: &[u8] = b"escrow_authority";  // For EscrowAuthority PDA: ["escrow_authority"]
}