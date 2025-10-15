use crate::{state::TradingSession, AccountsDelegated};
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::delegate, cpi::DelegateConfig};
#[delegate]
#[derive(Accounts)]
pub struct DelegateTradingAccounts<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub session: Account<'info, TradingSession>,

    /// CHECK: Will be validated by delegate program
    #[account(mut, del)]
    pub portfolio: AccountInfo<'info>,

    /// CHECK: Optional validator account (passed as remaining account or in DelegateConfig)
    pub validator: Option<AccountInfo<'info>>,

    #[account(mut, del)]
    pub pda: AccountInfo<'info>,
}

/// Delegate the portfolio (or multiple PDAs) to an ER validator so they may execute high-frequency updates off-chain
pub fn process_delegate_trading_accounts(ctx: Context<DelegateTradingAccounts>) -> Result<()> {
    // Use delegate macro helper to perform the CPI into delegation program
    // Validator optionally passed as remaining account or provided in DelegateConfig
    ctx.accounts.delegate_pda(
        &ctx.accounts.user,
        &[
            b"portfolio",
            ctx.accounts.session.key().as_ref(),
            ctx.accounts.user.key().as_ref(),
        ],
        DelegateConfig {
            // Optionally set a validator pubkey (first remaining account)
            validator: ctx.remaining_accounts.first().map(|acc| acc.key()),
            // You may configure auto-commit frequency, etc. via DelegateConfig fields if available
            ..Default::default()
        },
    )?;

    emit!(AccountsDelegated {
        user: ctx.accounts.user.key(),
        session_id: ctx.accounts.session.session_id,
    });

    msg!(
        "Delegated trading accounts for user {}",
        ctx.accounts.user.key()
    );
    Ok(())
}
