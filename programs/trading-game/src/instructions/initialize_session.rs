use anchor_lang::prelude::*;

use crate::{SessionInitialized, TradingSession};

#[derive(Accounts)]
pub struct InitializeSession<'info> {
    #[account(init, payer = authority, space = 8 + TradingSession::INIT_SPACE)]
    pub session: Account<'info, TradingSession>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

 // ----------------------------
    pub fn process_initialize_session(
        ctx: Context<InitializeSession>,
        session_id: u64,
        duration_seconds: i64,
        virtual_balance: u64,
        trading_pairs: Vec<String>,
    ) -> Result<()> {
        let session = &mut ctx.accounts.session;
        let clock = Clock::get()?;

        session.session_id = session_id;
        session.start_time = clock.unix_timestamp;
        session.end_time = clock.unix_timestamp + duration_seconds;
        session.virtual_balance_per_user = virtual_balance;
        session.trading_pairs = trading_pairs;
        session.is_active = true;
        session.participant_count = 0;

        emit!(SessionInitialized{
            session_id,
            start_time: session.start_time,
            end_time: session.end_time,
        });

        msg!("Session {} initialized for {} seconds", session_id, duration_seconds);
        Ok(())
    }
