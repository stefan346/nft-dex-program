use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Transfer, Token, self};

use crate::account_states::*;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct CrankCtx<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, 
        constraint = rb_crank.to_account_info().owner == program_id,
        constraint = rb_crank.load()?.instrmt_grp == instrmt_grp.key()
    )]
    pub rb_crank: AccountLoader<'info, RingBufferCrank>,

    pub instrmt_grp: Box<Account<'info, InstrmtGrp>>,

    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CrankCtx>) -> Result<()> {
    let rb_crank = &mut ctx.accounts.rb_crank.load_mut()?;
    let crank = rb_crank.remove_head();
    
    require!(!crank.is_empty(), ErrorCode::RbCrankEmpty);
    require!(crank.get_token_account() == ctx.accounts.user.key(), ErrorCode::WrongTokenAccount);
    require!(crank.get_vault() == ctx.accounts.vault.key(), ErrorCode::WrongVaultAccount);

    let instrmt_grp_seeds = &[
        b"instrmt-grp".as_ref(),
        &ctx.accounts.instrmt_grp.admin.as_ref(),
        &[ctx.accounts.instrmt_grp.bump],
    ];

    let signer = &[&instrmt_grp_seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info().clone(),
        to: ctx.accounts.user.to_account_info().clone(),
        authority: ctx.accounts.instrmt_grp.to_account_info().clone(),
    };
    let cpi_context = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, signer);

    token::transfer(
        cpi_context, crank.get_quantity()
    )?;

    Ok(())
}
