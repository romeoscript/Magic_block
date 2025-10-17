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
    #[msg("Price data is stale")]
    StalePriceData,
    #[msg("Invalid price data")]
    InvalidPriceData,
    #[msg("Price Feed Not Found")]
    PriceFeedNotFound,

    #[msg("Unsupported trading pair")]
    UnsupportedTradingPair,
}
