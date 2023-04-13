use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::account_states::*;
use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct NewNftPoolIx {
    pub verified_creators: Vec<Pubkey>,
}

#[derive(Accounts)]
#[instruction(ix: NewNftPoolIx)]
pub struct NewNftPoolCtx<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        seeds = [
            b"nft-pool",
            authority.key().as_ref(),
            ix.verified_creators.iter().fold("".to_string(), |cur, nxt| cur + &nxt.to_string()).as_ref()
        ],
        bump,
        space = NftPool::space(ix.verified_creators.len())

    )]
    pub nft_pool: Box<Account<'info, NftPool>>,

    #[account(
        init,
        payer = authority,
        seeds = [nft_pool.key().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = mint
    )]
    pub mint: Account<'info, Mint>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<NewNftPoolCtx>, ix: NewNftPoolIx) -> Result<()> {
    let nft_pool = &mut ctx.accounts.nft_pool;

    nft_pool.mint = ctx.accounts.mint.key();
    nft_pool.bump = *ctx.bumps.get("nft_pool").unwrap();
    nft_pool.verified_creators = ix.verified_creators;

    Ok(())
}
