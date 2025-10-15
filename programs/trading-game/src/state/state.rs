use anchor_lang::prelude::*;


#[account]
pub struct TradingSession {
    pub session_id: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub virtual_balance_per_user: u64,
    pub trading_pairs: Vec<String>,
    pub is_active: bool,
    pub participant_count: u32,
}
impl TradingSession {
    // conservative estimate for space
    pub const INIT_SPACE: usize = 8 + 8 + 8 + 8 + (4 + 32 * 10) + 1 + 4;
}

#[account]
pub struct Portfolio {
    pub owner: Pubkey,
    pub session_id: u64,
    pub cash_balance: i64,
    pub total_value: i64,
    pub realized_pnl: i64,
    pub unrealized_pnl: i64,
    pub num_trades: u32,
    pub positions: Vec<Position>,
}
impl Portfolio {
    pub const INIT_SPACE: usize = 32 + 8 + 8 + 8 + 8 + 8 + 4 + (4 + Position::SIZE * 20);
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Position {
    pub trading_pair: String,
    pub quantity: u64,
    pub avg_entry_price: i64,
    pub side: PositionSide,
}
impl Position {
    const SIZE: usize = 32 + 8 + 8 + 1;
}

#[account]
pub struct Leaderboard {
    pub session_id: u64,
    pub entries: Vec<LeaderboardEntry>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct LeaderboardEntry {
    pub user: Pubkey,
    pub total_pnl: i64,
    pub roi_percentage: f64,
    pub num_trades: u32,
    pub last_updated: i64,
    pub rank: u32,
}

#[account]
pub struct PriceFeed {
    pub trading_pair: String,
    pub price: i64,      // price with 6 decimals
    pub last_updated: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum PositionSide {
    Long,
    Short,
}
