use anchor_lang::prelude::*;

use crate::{calculate_portfolio_value, PnlUpdated, Portfolio};
#[derive(Accounts)]
pub struct UpdatePnl<'info> {
    #[account(mut)]
    pub portfolio: Account<'info, Portfolio>,
}


    /// Update P&L (can be called on-chain or executed frequently on ER)
    pub fn process_update_pnl(ctx: Context<UpdatePnl>) -> Result<()> {
        let portfolio = &mut ctx.accounts.portfolio;
        calculate_portfolio_value(portfolio, &ctx.remaining_accounts)?;
        emit!(PnlUpdated {
            user: portfolio.owner,
            unrealized_pnl: portfolio.unrealized_pnl,
            realized_pnl: portfolio.realized_pnl,
            total_value: portfolio.total_value,
        });
        Ok(())
    }
