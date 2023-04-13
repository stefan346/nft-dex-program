use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

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

    #[account(
        constraint = quote_mint.decimals == 0
    )]
    pub quote_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = base_mint.decimals == 0
    )]
    pub base_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<NewInstrmtCtx>, ix: NewInstrmtIx) -> Result<()> {
    Ok(())
}
