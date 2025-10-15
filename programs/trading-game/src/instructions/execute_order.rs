use anchor_lang::prelude::*;

use crate::{calculate_portfolio_value, error::ErrorCode, OrderExecuted, OrderSide, Portfolio, Position, PositionSide, PriceFeed, TradingSession};

#[derive(Accounts)]
pub struct ExecuteOrder<'info> {
    #[account(mut)]
    pub portfolio: Account<'info, Portfolio>,
    pub session: Account<'info, TradingSession>,
    pub price_feed: Account<'info, PriceFeed>,
    pub user: Signer<'info>,
}

    pub fn process_execute_market_order(
        ctx: Context<ExecuteOrder>,
        trading_pair: String,
        side: OrderSide,
        quantity: u64,
    ) -> Result<()> {
        let portfolio = &mut ctx.accounts.portfolio;
        let price_feed = &ctx.accounts.price_feed;
        let session = &ctx.accounts.session;
        let clock = Clock::get()?;

        require!(session.is_active, ErrorCode::SessionInactive);
        require!(clock.unix_timestamp < session.end_time, ErrorCode::SessionEnded);

        // current_price expressed with 6 decimals (e.g., 1 SOL = 10_000_000)
        let current_price = price_feed.price;
        // order_value in the same 6-decimals base
        let order_value = (quantity as i64)
            .checked_mul(current_price)
            .ok_or(ErrorCode::MathOverflow)?
            / 1_000_000;

        match side {
            OrderSide::Buy => {
                require!(portfolio.cash_balance >= order_value, ErrorCode::InsufficientFunds);

                portfolio.cash_balance = portfolio
                    .cash_balance
                    .checked_sub(order_value)
                    .ok_or(ErrorCode::MathOverflow)?;

                // add or average into existing position
                if let Some(pos) = portfolio
                    .positions
                    .iter_mut()
                    .find(|p| p.trading_pair == trading_pair)
                {
                    let total_cost = (pos.quantity as i64)
                        .checked_mul(pos.avg_entry_price)
                        .ok_or(ErrorCode::MathOverflow)?
                        .checked_add(order_value.checked_mul(1_000_000).ok_or(ErrorCode::MathOverflow)?)
                        .ok_or(ErrorCode::MathOverflow)?;

                    pos.quantity = pos.quantity.checked_add(quantity).ok_or(ErrorCode::MathOverflow)?;
                    // avg_entry_price stored in same 6-decimal basis as price_feed.price
                    pos.avg_entry_price = total_cost.checked_div(pos.quantity as i64).ok_or(ErrorCode::MathOverflow)?;
                } else {
                    portfolio.positions.push(Position {
                        trading_pair: trading_pair.clone(),
                        quantity,
                        avg_entry_price: current_price,
                        side: PositionSide::Long,
                    });
                }
            }
            OrderSide::Sell => {
                let pos_idx = portfolio
                    .positions
                    .iter()
                    .position(|p| p.trading_pair == trading_pair)
                    .ok_or(ErrorCode::NoPosition)?;

                let position = &mut portfolio.positions[pos_idx];
                require!(position.quantity >= quantity, ErrorCode::InsufficientPosition);

                // cost basis and proceeds use same decimals
                let cost_basis = (quantity as i64)
                    .checked_mul(position.avg_entry_price)
                    .ok_or(ErrorCode::MathOverflow)?
                    / 1_000_000;
                let sale_proceeds = order_value;
                let pnl = sale_proceeds.checked_sub(cost_basis).ok_or(ErrorCode::MathOverflow)?;

                portfolio.realized_pnl = portfolio.realized_pnl.checked_add(pnl).ok_or(ErrorCode::MathOverflow)?;
                portfolio.cash_balance = portfolio.cash_balance.checked_add(sale_proceeds).ok_or(ErrorCode::MathOverflow)?;

                // update or remove position
                position.quantity = position.quantity.checked_sub(quantity).ok_or(ErrorCode::MathOverflow)?;
                if position.quantity == 0 {
                    portfolio.positions.remove(pos_idx);
                }
            }
        }

        portfolio.num_trades = portfolio.num_trades.checked_add(1).ok_or(ErrorCode::MathOverflow)?;

        // Recalculate total portfolio value using price feeds passed as remaining accounts
        calculate_portfolio_value(portfolio, &ctx.remaining_accounts)?;

        emit!(OrderExecuted {
            user: portfolio.owner,
            trading_pair,
            side,
            quantity,
            price: current_price,
            timestamp: clock.unix_timestamp,
        });

        msg!(
            "Executed {:?} order: {} qty @ {} for user {}",
            side,
            quantity,
            current_price,
            portfolio.owner
        );

        Ok(())
    }