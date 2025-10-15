use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::ephem::commit_accounts;
use crate::{state::{Leaderboard, Portfolio, TradingSession}, AccountsCheckpointed};

#[derive(Accounts)]
pub struct CheckpointAccounts<'info> {
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

    /// payer authority
    #[account(mut)]
    pub payer: Signer<'info>,
}

 /// Commit a set of delegated accounts (checkpoint) while still delegated. This would be invoked by validator or client.
    pub fn process_checkpoint_trading_accounts(ctx: Context<CheckpointAccounts>) -> Result<()> {
        // Commit accounts in-place (the SDK helper expects: magic_context, vec![accounts], magic_program, payer)
        commit_accounts(
            &ctx.accounts.magic_context,
            vec![
                &ctx.accounts.portfolio.to_account_info(),
                &ctx.accounts.leaderboard.to_account_info(),
            ],
            &ctx.accounts.magic_program,
            &ctx.accounts.payer.to_account_info(),
        )?;

        emit!(AccountsCheckpointed {
            session_id: ctx.accounts.session.session_id,
            user: ctx.accounts.portfolio.owner,
        });

        msg!("Checkpoint committed for user {}", ctx.accounts.portfolio.owner);
        Ok(())
    }