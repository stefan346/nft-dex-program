use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::account_states::*;
use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct NewInstrmtIx {}

#[derive(Accounts)]
pub struct NewInstrmtCtx<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"instrmt-grp", authority.key().as_ref()],
        bump = instrmt_grp.bump,
        realloc = InstrmtGrp::space(instrmt_grp.instrmts.len() + 1),
        realloc::payer = authority,
        realloc::zero = false
    )]
    pub instrmt_grp: Box<Account<'info, InstrmtGrp>>,

    #[account(zero)]
    pub instrmt: AccountLoader<'info, Instrmt>,

    #[account(zero)]
    pub book: AccountLoader<'info, Book>,

    #[account(
        constraint = base_mint.decimals == 0
    )]
    pub base_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        seeds = [b"base-vault", instrmt.key().as_ref()],
        bump,
        payer = authority,
        token::mint = base_mint,
        token::authority = instrmt_grp,
    )]
    pub base_vault: Box<Account<'info, TokenAccount>>,

    pub quote_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        seeds = [b"quote-vault", instrmt.key().as_ref()],
        bump,
        payer = authority,
        token::mint = quote_mint,
        token::authority = instrmt_grp,
    )]
    pub quote_vault: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<NewInstrmtCtx>, ix: NewInstrmtIx) -> Result<()> {
    let instrmt_grp = &mut ctx.accounts.instrmt_grp;
    instrmt_grp.instrmts.push(ctx.accounts.instrmt.key());

    let instrmt = &mut ctx.accounts.instrmt.load_init()?;

    instrmt.base_mint = ctx.accounts.base_mint.key();
    instrmt.base_vault = ctx.accounts.base_vault.key();

    instrmt.quote_mint = ctx.accounts.quote_mint.key();
    instrmt.quote_vault = ctx.accounts.quote_vault.key();

    let book = &mut ctx.accounts.book.load_init()?;

    instrmt.book = ctx.accounts.book.key();
    instrmt.bumps = InstrmtBumps {
        base_vault_bump: *ctx.bumps.get("base_vault").unwrap(),
        quote_vault_bump: *ctx.bumps.get("quote_vault").unwrap(),
    };

    instrmt.top_of_filled_exec_reports.next_index = 0;

    book.ask_min = 0;
    book.bid_max = 0;
    
    book.asks.head = 0;
    book.asks.tail = 0;

    book.bids.head = 0;
    book.bids.tail = 0;
    Ok(())
}
