use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::account_states::*;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct CrankCtx<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, 
        constraint = rb_crank.to_account_info().owner == program_id,
    )]
    pub rb_crank: AccountLoader<'info, RingBufferCrank>,

    #[account(mut, constraint = vault.mint == user.mint)]
    pub vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Box<Account<'info, TokenAccount>>,
}

pub fn handler(ctx: Context<CrankCtx>) -> Result<()> {
    Ok(())
}
