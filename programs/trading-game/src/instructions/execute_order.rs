use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{
    calculate_portfolio_value, error::ErrorCode, OrderExecuted, OrderSide, Portfolio, Position,
    PositionSide, TradingSession,
};

#[derive(Accounts)]
pub struct ExecuteOrder<'info> {
    #[account(mut)]
    pub portfolio: Account<'info, Portfolio>,
    pub session: Account<'info, TradingSession>,
    pub price_update: Account<'info, PriceUpdateV2>,
    pub user: Signer<'info>,
}

pub fn process_execute_market_order(
    ctx: Context<ExecuteOrder>,
    trading_pair: String,
    side: OrderSide,
    quantity: u64,
) -> Result<()> {
    let portfolio = &mut ctx.accounts.portfolio;
    let session = &ctx.accounts.session;
    let price_update = &ctx.accounts.price_update;
    let clock = Clock::get()?;
    require!(session.is_active, ErrorCode::SessionInactive);
    require!(
        clock.unix_timestamp < session.end_time,
        ErrorCode::SessionEnded
    );

    let feed_id = match trading_pair.as_str() {
        "JUP/USD" => get_feed_id_from_hex(
            "0x0a0408d619e9380abad35060f9192039ed5042fa6f82301d0e48bb52be830996",
        )?,
        "SOL/USD" => get_feed_id_from_hex(
            "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
        )?,
        _ => return Err(ErrorCode::UnsupportedTradingPair.into()),
    };

    // Get price from Pyth oracle (max 30 seconds old)
    let maximum_age: u64 = 30;
    let price_data = price_update
        .get_price_no_older_than(&clock, maximum_age, &feed_id)
        .map_err(|_| ErrorCode::StalePriceData)?;

    // Convert Pyth price to your 6-decimal format
    // Pyth typically uses exponent (e.g., exponent=-8 means price is in 10^-8)
    // We need to normalize to 6 decimals (1_000_000 = 1.0)
    // let current_price = normalize_price(price_data.price, price_data.exponent)?;

    msg!(
        "Pyth price for {}: ({} ± {}) * 10^{} = {} (normalized)",
        trading_pair,
        price_data.price,
        price_data.conf,
        price_data.exponent,
        price_data.price
    );
    // order_value in the same 6-decimals base
    let order_value = (quantity as i64)
        .checked_mul(price_data.price)
        .ok_or(ErrorCode::MathOverflow)?
        / 1_000_000;

    match side {
        OrderSide::Buy => {
            require!(
                portfolio.cash_balance >= order_value,
                ErrorCode::InsufficientFunds
            );

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
                    .checked_add(
                        order_value
                            .checked_mul(1_000_000)
                            .ok_or(ErrorCode::MathOverflow)?,
                    )
                    .ok_or(ErrorCode::MathOverflow)?;

                pos.quantity = pos
                    .quantity
                    .checked_add(quantity)
                    .ok_or(ErrorCode::MathOverflow)?;
                // avg_entry_price stored in same 6-decimal basis as price_feed.price
                pos.avg_entry_price = total_cost
                    .checked_div(pos.quantity as i64)
                    .ok_or(ErrorCode::MathOverflow)?;
            } else {
                portfolio.positions.push(Position {
                    trading_pair: trading_pair.clone(),
                    quantity,
                    avg_entry_price: price_data.price,
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

            // Read needed fields into locals so we don't hold a mutable borrow across portfolio updates
            let available_qty = portfolio.positions[pos_idx].quantity;
            require!(available_qty >= quantity, ErrorCode::InsufficientPosition);
            let avg_entry_price = portfolio.positions[pos_idx].avg_entry_price;

            // cost basis and proceeds use same decimals
            let cost_basis = (quantity as i64)
                .checked_mul(avg_entry_price)
                .ok_or(ErrorCode::MathOverflow)?
                / 1_000_000;
            let sale_proceeds = order_value;
            let pnl = sale_proceeds
                .checked_sub(cost_basis)
                .ok_or(ErrorCode::MathOverflow)?;

            portfolio.realized_pnl = portfolio
                .realized_pnl
                .checked_add(pnl)
                .ok_or(ErrorCode::MathOverflow)?;
            portfolio.cash_balance = portfolio
                .cash_balance
                .checked_add(sale_proceeds)
                .ok_or(ErrorCode::MathOverflow)?;

            // update or remove position without holding a mutable borrow while removing
            let new_qty = portfolio.positions[pos_idx]
                .quantity
                .checked_sub(quantity)
                .ok_or(ErrorCode::MathOverflow)?;
            if new_qty == 0 {
                portfolio.positions.remove(pos_idx);
            } else {
                portfolio.positions[pos_idx].quantity = new_qty;
            }
        }
    }

    portfolio.num_trades = portfolio
        .num_trades
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;

    // Recalculate total portfolio value using price feeds passed as remaining accounts
    calculate_portfolio_value(portfolio, &ctx.remaining_accounts)?;

    msg!(
        "Executed {:?} order: {} qty @ {} for user {}",
        side,
        quantity,
        price_data.price,
        portfolio.owner
    );

    emit!(OrderExecuted {
        user: portfolio.owner,
        trading_pair,
        side,
        quantity,
        price: price_data.price,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

// /// Normalize Pyth price to 6 decimals (1_000_000 = 1.0)
// fn normalize_price(price: i64, exponent: i32) -> Result<i64> {
//     // Pyth exponent is typically negative (e.g., -8 means 10^-8)
//     // Target exponent is -6 (for 6 decimals)
//     let target_exponent: i32 = -6;
//     let exponent_diff = target_exponent - exponent;

//     let normalized = if exponent_diff > 0 {
//         // Need to multiply (shift decimal right)
//         price
//             .checked_mul(10_i64.pow(exponent_diff as u32))
//             .ok_or(ErrorCode::MathOverflow)?
//     } else if exponent_diff < 0 {
//         // Need to divide (shift decimal left)
//         price
//             .checked_div(10_i64.pow((-exponent_diff) as u32))
//             .ok_or(ErrorCode::MathOverflow)?
//     } else {
//         price
//     };

//     Ok(normalized)
// }
