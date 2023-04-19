use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::account_states::*;
use crate::enums::OrderType;
use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct NewOrderSingleIx {
    pub is_buy: bool,
    pub limit: u64,
    pub size: u64,
    pub order_type: OrderType,
}

#[derive(Accounts)]
pub struct NewOrderSingleCtx<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        constraint = instrmt.load()?.book == book.key(),
        constraint = instrmt.load()?.base_vault == base_vault.key(),
        constraint = instrmt.load()?.quote_vault == quote_vault.key()
    )]
    pub instrmt: AccountLoader<'info, Instrmt>,

    pub base_vault: Box<Account<'info, TokenAccount>>,
    pub quote_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub book: AccountLoader<'info, Book>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<NewOrderSingleCtx>, ix: NewOrderSingleIx) -> Result<()> {
    let book = &mut ctx.accounts.book.load_mut()?;
    let instrmt = &mut ctx.accounts.instrmt.load_mut()?;
    let order = book.new_limit(&ix, ctx.accounts.authority.key(), &mut instrmt.top_of_filled_exec_reports);

    let cost: u64 = match ix.order_type {
        OrderType::FOK => {
            require!(order.is_filled(), ErrorCode::FillOrKillFailed);
            order.get_cum_cost()
        },
        // All good. No checks required.
        OrderType::GTC => order.get_cum_cost() + order.get_leaves_cost(),
        // All good. Only check to prevent order insert within new_limit fn required.
        OrderType::IOC => order.get_cum_cost(),
        OrderType::MO => {
            require!(!order.is_partially_filed(), ErrorCode::MakerOnlyFailed);
            order.get_leaves_cost()
        }
    };
    
    // take funds
    Ok(())
}
