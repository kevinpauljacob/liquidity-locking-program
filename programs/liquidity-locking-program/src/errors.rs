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
    #[msg("Invalid vesting account")]
    InvalidVesting,
    #[msg("Lock period not expired")]
    LockNotExpired, 
    #[msg("Invalid unlock amount")]
    InvalidUnlockAmount,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Lock is not active")]
    LockNotActive,
}
