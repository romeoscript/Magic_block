use anchor_lang::prelude::*;
use crate::Portfolio;

pub fn calculate_roi(portfolio: &Portfolio) -> Result<f64> {
    // initial balance can be taken from session or constant; using a constant here for example
    let initial_balance: f64 = 100_000.0; // human-readable dollars
    let total_pnl = (portfolio.realized_pnl + portfolio.unrealized_pnl) as f64 / 1_000_000.0;
    // avoid divide by zero
    if initial_balance.abs() < f64::EPSILON {
        return Ok(0.0);
    }
    Ok((total_pnl / initial_balance) * 100.0)
}