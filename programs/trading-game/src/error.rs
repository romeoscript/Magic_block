use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Trading session is not active")]
    SessionInactive,
    #[msg("Trading session has ended")]
    SessionEnded,
    #[msg("Insufficient cash balance")]
    InsufficientFunds,
    #[msg("No position found for this trading pair")]
    NoPosition,
    #[msg("Insufficient position size")]
    InsufficientPosition,
    #[msg("Session is still active, cannot close yet")]
    SessionStillActive,
    #[msg("Math overflow")]
    MathOverflow,
}