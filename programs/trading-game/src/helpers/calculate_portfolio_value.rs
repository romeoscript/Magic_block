use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{PriceUpdateV2, get_feed_id_from_hex};
use crate::{error::ErrorCode, Portfolio, Position};

pub fn calculate_portfolio_value(
    portfolio: &mut Portfolio,
    price_update_accounts: &[AccountInfo],
) -> Result<()> {
    let mut unrealized_pnl: i64 = 0;
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    for position in &portfolio.positions {
        // Get the Pyth price feed for this trading pair
        let price = get_price_for_pair(
            &position.trading_pair,
            price_update_accounts,
            current_timestamp,
        )?;

        // Calculate position value with oracle price
        let current_value = (position.quantity as i64)
            .checked_mul(price)
            .ok_or(ErrorCode::MathOverflow)?
            / 1_000_000;

        let cost_basis = (position.quantity as i64)
            .checked_mul(position.avg_entry_price)
            .ok_or(ErrorCode::MathOverflow)?
            / 1_000_000;

        unrealized_pnl = unrealized_pnl
            .checked_add(
                current_value
                    .checked_sub(cost_basis)
                    .ok_or(ErrorCode::MathOverflow)?
            )
            .ok_or(ErrorCode::MathOverflow)?;
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

fn get_price_for_pair(
    trading_pair: &str,
    price_accounts: &[AccountInfo],
    current_timestamp: i64,
) -> Result<i64> {
    // Map your trading pairs to Pyth feed IDs
    let feed_id = match trading_pair {
        "JUP/USD" => get_feed_id_from_hex("0x0a0408d619e9380abad35060f9192039ed5042fa6f82301d0e48bb52be830996")?,
        "SOL/USD" => get_feed_id_from_hex("0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d")?,
        _ => return Err(ErrorCode::UnsupportedTradingPair.into()),
    };

    // Find the matching price account
    for account_info in price_accounts {
        let price_update = PriceUpdateV2::try_deserialize(&mut &account_info.data.borrow()[..])?;
        
        if price_update.price_message.feed_id == feed_id {
            // Check price is not stale (e.g., within last 60 seconds)
            let price_age = current_timestamp
                .checked_sub(price_update.price_message.publish_time)
                .ok_or(ErrorCode::MathOverflow)?;
            
            require!(price_age <= 60, ErrorCode::StalePriceData);

            // Return price scaled to your 6 decimal format
            return Ok(price_update.price_message.price);
        }
    }

    Err(ErrorCode::PriceFeedNotFound.into())
}