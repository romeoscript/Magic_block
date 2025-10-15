use anchor_lang::prelude::*;

use crate::OrderSide;
// Events
// ----------------------------
#[event]
pub struct SessionInitialized {
    pub session_id: u64,
    pub start_time: i64,
    pub end_time: i64,
}

#[event]
pub struct ParticipantJoined {
    pub session_id: u64,
    pub user: Pubkey,
    pub initial_balance: u64,
}

#[event]
pub struct OrderExecuted {
    pub user: Pubkey,
    pub trading_pair: String,
    pub side: OrderSide,
    pub quantity: u64,
    pub price: i64,
    pub timestamp: i64,
}

#[event]
pub struct PnlUpdated {
    pub user: Pubkey,
    pub unrealized_pnl: i64,
    pub realized_pnl: i64,
    pub total_value: i64,
}

#[event]
pub struct LeaderboardUpdated {
    pub session_id: u64,
}

#[event]
pub struct SessionClosed {
    pub session_id: u64,
    pub participant_count: u32,
}

#[event]
pub struct AccountsDelegated {
    pub user: Pubkey,
    pub session_id: u64,
}
#[event]
pub struct AccountsCheckpointed {
    pub session_id: u64,
    pub user: Pubkey,
}
#[event]
pub struct AccountsFinalized {
    pub session_id: u64,
    pub user: Pubkey,
}
