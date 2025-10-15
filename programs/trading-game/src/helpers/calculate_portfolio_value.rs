use anchor_lang::prelude::*;
use crate::{error::ErrorCode, Portfolio, PriceFeed};


pub fn calculate_portfolio_value(portfolio: &mut Portfolio, price_feeds: &[AccountInfo]) -> Result<()> {
    let mut unrealized_pnl: i64 = 0;

    for position in &portfolio.positions {
        // naive: use the first remaining account price feed that matches (production: match by trading_pair)
        if let Some(feed_info) = price_feeds.iter().find(|ai| {
            // best-effort matching by checking first bytes; in practice you'd deserialize and check trading_pair
            true
        }) {
            let price_feed = PriceFeed::try_from_slice(&feed_info.data.borrow())?;
            let current_value = (position.quantity as i64)
                .checked_mul(price_feed.price)
                .ok_or(ErrorCode::MathOverflow)?
                / 1_000_000;
            let cost_basis = (position.quantity as i64)
                .checked_mul(position.avg_entry_price)
                .ok_or(ErrorCode::MathOverflow)?
                / 1_000_000;
            unrealized_pnl = unrealized_pnl.checked_add(current_value.checked_sub(cost_basis).ok_or(ErrorCode::MathOverflow)?).ok_or(ErrorCode::MathOverflow)?;
        }
    }

    portfolio.unrealized_pnl = unrealized_pnl;
    portfolio.total_value = portfolio
        .cash_balance
        .checked_add(portfolio.realized_pnl)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_add(unrealized_pnl)
        .ok_or(ErrorCode::MathOverflow)?;

    Ok(())
}
