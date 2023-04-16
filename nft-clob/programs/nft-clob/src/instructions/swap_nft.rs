use crate::account_states::*;
use crate::errors::ErrorCode;
use anchor_lang::prelude::*;
use anchor_spl::metadata::{MasterEditionAccount, Metadata};
use anchor_spl::token::{Mint, TokenAccount};

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

    #[account(
        // address = Metadata::id(),
        seeds = [b"metadata", Metadata::id().as_ref(), nft_mint.key().as_ref()],
        seeds::program = Metadata::id(),
        bump
    )]
    pub nft_metadata_account: AccountInfo<'info>,

    #[account(
        seeds = [b"metadata", Metadata::id().as_ref(), nft_mint.key().as_ref(), b"edition"],
        seeds::program = Metadata::id(),
        bump,
        constraint = master_edition.supply > 0
    )]
    pub master_edition: Box<Account<'info, MasterEditionAccount>>,
}

pub fn handler(ctx: Context<SwapNftCtx>, ix: SwapNftIx) -> Result<()> {
    Ok(())
}
