use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;

use crate::{AccountsFinalized, Leaderboard, Portfolio, TradingSession};

#[derive(Accounts)]
pub struct FinalizeTradingAccounts<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub session: Account<'info, TradingSession>,

    #[account(mut)]
    pub portfolio: Account<'info, Portfolio>,

    #[account(mut)]
    pub leaderboard: Account<'info, Leaderboard>,

    /// CHECK: Magic ER context
    #[account(mut)]
    pub magic_context: AccountInfo<'info>,

    /// CHECK: ER program account
    pub magic_program: AccountInfo<'info>,
}


 /// Finalize: commit final state and undelegate the accounts back to the program
    pub fn process_finalize_and_undelegate(ctx: Context<FinalizeTradingAccounts>) -> Result<()> {
        commit_and_undelegate_accounts(
            &ctx.accounts.user,
            vec![
                &ctx.accounts.portfolio.to_account_info(),
                &ctx.accounts.leaderboard.to_account_info(),
            ],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;

        emit!(AccountsFinalized {
            session_id: ctx.accounts.session.session_id,
            user: ctx.accounts.user.key(),
        });

        msg!("Finalized and undelegated accounts for user {}", ctx.accounts.user.key());
        Ok(())
    }

