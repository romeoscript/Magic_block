pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod event;
pub mod helpers;

use anchor_lang::prelude::*;
 use ephemeral_rollups_sdk::anchor::{
    commit,
    delegate,
    ephemeral,
};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::{
    commit_accounts,
    commit_and_undelegate_accounts,
};


pub use constants::*;
pub use instructions::*;
pub use state::*;
pub use event::*;
pub use helpers::*;
declare_id!("H6Nh8SuybPujX1jibAn1bbuQhp7cmf7oWmaq4SVjj2cw");

#[ephemeral]
#[program]
pub mod trading_game {

    use super::*;


    pub fn initialize_session(ctx: Context<InitializeSession>, session_id: u64, duration_seconds: i64, virtual_balance: u64, trading_pairs: Vec<String>) -> Result<()> {
        process_initialize_session(ctx, session_id, duration_seconds, virtual_balance, trading_pairs)
    }
    pub fn join_session(ctx: Context<JoinSession>, session_id: u64) -> Result<()> {
        process_join_session(ctx, session_id)
    }

    pub fn execute_order(ctx: Context<ExecuteOrder>, trading_pair: String) -> Result<()>{
        process_execute_market_order(ctx, trading_pair, side, quantity);
    }
    pub fn update_pnl(ctx: Context<UpdatePnl>)->Result<()>{
        process_update_pnl(ctx)
    }
    pub fn update_leaderboard(ctx: Context<UpdateLeaderboard>)->Result<()>{
        process_update_leaderboard(ctx)
    }
    pub fn checkpoint_accounts(ctx: Context<CheckpointAccounts>)->Result<()>{
        process_checkpoint_trading_accounts(ctx)
    }
    pub fn finalize_trading_accounts(ctx: Context<FinalizeTradingAccounts>)->Result<()>{
        process_finalize_and_undelegate(ctx)
    }
    pub fn close_session(ctx: Context<CloseSession>)->Result<()>{
        process_close_session(ctx)
    }
}
