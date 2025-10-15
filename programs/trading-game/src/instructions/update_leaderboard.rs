use anchor_lang::prelude::*;

use crate::{calculate_roi, Leaderboard, LeaderboardEntry, LeaderboardUpdated, Portfolio};
#[derive(Accounts)]
pub struct UpdateLeaderboard<'info> {
    #[account(mut)]
    pub leaderboard: Account<'info, Leaderboard>,
    pub portfolio: Account<'info, Portfolio>,
}


pub fn process_update_leaderboard(ctx: Context<UpdateLeaderboard>) -> Result<()> {
        let leaderboard = &mut ctx.accounts.leaderboard;
        let portfolio = &ctx.accounts.portfolio;
        let clock = Clock::get()?;

        // find or push
        if let Some(entry) = leaderboard.entries.iter_mut().find(|e| e.user == portfolio.owner) {
            entry.total_pnl = portfolio.realized_pnl + portfolio.unrealized_pnl;
            entry.roi_percentage = calculate_roi(portfolio)?;
            entry.num_trades = portfolio.num_trades;
            entry.last_updated = clock.unix_timestamp;
        } else {
            leaderboard.entries.push(LeaderboardEntry {
                user: portfolio.owner,
                total_pnl: portfolio.realized_pnl + portfolio.unrealized_pnl,
                roi_percentage: calculate_roi(portfolio)?,
                num_trades: portfolio.num_trades,
                last_updated: clock.unix_timestamp,
                rank: 0,
            });
        }

        // sort & rank
        leaderboard.entries.sort_by(|a, b| b.total_pnl.cmp(&a.total_pnl));
        for (idx, entry) in leaderboard.entries.iter_mut().enumerate() {
            entry.rank = (idx + 1) as u32;
        }

        emit!(LeaderboardUpdated {
            session_id: leaderboard.session_id,
        });

        Ok(())
    }