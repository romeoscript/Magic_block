use anchor_lang::prelude::*;

use crate::{error::ErrorCode, ParticipantJoined, Portfolio, TradingSession};
#[derive(Accounts)]
pub struct JoinSession<'info> {
    // #[account(mut, has_one = session)]
    #[account(mut)]
    pub session: Account<'info, TradingSession>,
    #[account(init, payer = user, space = 8 + Portfolio::INIT_SPACE, seeds = [b"portfolio", session.key().as_ref(), user.key().as_ref()], bump)]
    pub portfolio: Account<'info, Portfolio>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

 pub fn process_join_session(ctx: Context<JoinSession>, session_id: u64) -> Result<()> {
        let session = &mut ctx.accounts.session;
        let portfolio = &mut ctx.accounts.portfolio;

        require!(session.is_active, ErrorCode::SessionInactive);
        let clock = Clock::get()?;
        require!(clock.unix_timestamp < session.end_time, ErrorCode::SessionEnded);

        portfolio.owner = ctx.accounts.user.key();
        portfolio.session_id = session_id;
        portfolio.cash_balance = session.virtual_balance_per_user as i64;
        portfolio.total_value = session.virtual_balance_per_user as i64;
        portfolio.unrealized_pnl = 0;
        portfolio.realized_pnl = 0;
        portfolio.num_trades = 0;
        portfolio.positions = vec![];

        session.participant_count = session.participant_count.saturating_add(1);

        emit!(ParticipantJoined {
            session_id,
            user: portfolio.owner,
            initial_balance: session.virtual_balance_per_user,
        });

        msg!(
            "User {} joined session {} with {} virtual balance",
            ctx.accounts.user.key(),
            session_id,
            session.virtual_balance_per_user
        );
        Ok(())
    }