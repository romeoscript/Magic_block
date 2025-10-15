use anchor_lang::prelude::*;

use crate::{error::ErrorCode, SessionClosed, TradingSession};  
  
#[derive(Accounts)]
pub struct CloseSession<'info> {
    #[account(mut)]
    pub session: Account<'info, TradingSession>,
    pub authority: Signer<'info>,
}
  
  
  // ----------------------------
    // Closing session (finalize)
    // ----------------------------
    pub fn process_close_session(ctx: Context<CloseSession>) -> Result<()> {
        let session = &mut ctx.accounts.session;
        let clock = Clock::get()?;

        require!(clock.unix_timestamp >= session.end_time, ErrorCode::SessionStillActive);

        session.is_active = false;

        emit!(SessionClosed {
            session_id: session.session_id,
            participant_count: session.participant_count,
        });

        msg!(
            "Session {} closed. Final participant count: {}",
            session.session_id,
            session.participant_count
        );
        Ok(())
    }
