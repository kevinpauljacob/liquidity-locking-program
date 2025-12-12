use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid duration")]
    InvalidDuration,
    #[msg("Invalid liquidity amount")]
    InvalidLiquidity,
    #[msg("Pool is disabled")]
    PoolDisabled,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Invalid pool configuration")]
    InvalidPool,
}
