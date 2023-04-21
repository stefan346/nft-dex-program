use anchor_lang::prelude::*;

use crate::account_states::*;
use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CancelOrderIx {
    pub order_pos: u16,
    pub is_buy: bool,
}

#[derive(Accounts)]
pub struct CancelOrderCtx<'info> {
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"instrmt", book.key().as_ref()],
        bump = instrmt.bumps.instrmt_bump,
        constraint = instrmt.book == book.key(),
    )]
    pub instrmt: Box<Account<'info, Instrmt>>,

    #[account(
        constraint = instrmt_grp.key() == instrmt.instrmt_grp
    )]
    pub instrmt_grp: Box<Account<'info, InstrmtGrp>>,

    #[account(mut)]
    pub book: AccountLoader<'info, Book>,

    #[account(mut, constraint = rb_crank.load()?.instrmt_grp == instrmt.instrmt_grp)]
    pub rb_crank: AccountLoader<'info, RingBufferCrank>,
}

pub fn handler(ctx: Context<CancelOrderCtx>, ix: CancelOrderIx) -> Result<()> {
    let book = &mut ctx.accounts.book.load_mut()?;
    let rb_crank = &mut ctx.accounts.rb_crank.load_mut()?;

    let side = match ix.is_buy {
        true => &mut book.bids,
        false => &mut book.asks,
    };
    
    require!(!side.is_tombstone(ix.order_pos), ErrorCode::TombstoneOrder);

    let removed_order = side.remove_order(ix.order_pos);
    require!(
        removed_order.maker == ctx.accounts.authority.key(),
        ErrorCode::UnauthorizedOrderCancellation
    );
    rb_crank.insert(
        side.vault,
        removed_order.payment_acc,
        ix.is_buy,
        removed_order.maker,
        removed_order.get_leaves_qty(),
        removed_order.limit,
    );
    Ok(())
}
