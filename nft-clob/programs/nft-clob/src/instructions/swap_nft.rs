use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::account_states::*;
use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SwapNftIx {}

#[derive(Accounts)]
pub struct SwapNftCtx<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub nft_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = nft_mint.key() == nft_token_account.mint,
        constraint = authority.key() == nft_token_account.owner,
        constraint = 1 == nft_token_account.amount
    )]
    pub nft_token_account: Box<Account<'info, TokenAccount>>,

    pub nft_metadata_account: AccountInfo<'info>,
}

pub fn handler(ctx: Context<SwapNftCtx>, ix: SwapNftIx) -> Result<()> {
    Ok(())
}
