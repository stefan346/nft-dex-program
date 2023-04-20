use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

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
        seeds = [b"instrmt", book.key().as_ref()],
        bump = instrmt.bumps.instrmt_bump,
        constraint = instrmt.book == book.key(),
        constraint = instrmt.base_vault == base_vault.key(),
        constraint = instrmt.quote_vault == quote_vault.key(),
        constraint = instrmt.base_mint == base_user_token_account.mint,
        constraint = instrmt.quote_mint == quote_user_token_account.mint,
        constraint = instrmt.rb_filled_exec_reports == rb_filled_exec_reports.key(),
    )]
    pub instrmt: Box<Account<'info, Instrmt>>,

    #[account(
        constraint = instrmt_grp.key() == instrmt.instrmt_grp
    )]
    pub instrmt_grp: Box<Account<'info, InstrmtGrp>>,

    #[account(mut)]
    pub rb_filled_exec_reports: AccountLoader<'info, RingBufferFilledExecReport>,

    #[account(mut, constraint = rb_crank.load()?.instrmt_grp == instrmt.instrmt_grp)]
    pub rb_crank: AccountLoader<'info, RingBufferCrank>,

    #[account(mut)]
    pub base_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub quote_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = base_user_token_account.owner == authority.key())]
    pub base_user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = quote_user_token_account.owner == authority.key())]
    pub quote_user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub book: AccountLoader<'info, Book>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

impl<'info> NewOrderSingleCtx<'info> {
    pub fn into_base_transfer_user_to_vault(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.base_user_token_account.to_account_info().clone(),
            to: self.base_vault.to_account_info().clone(),
            authority: self.authority.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn into_quote_transfer_user_to_vault(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.quote_user_token_account.to_account_info().clone(),
            to: self.quote_vault.to_account_info().clone(),
            authority: self.authority.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn handler(ctx: Context<NewOrderSingleCtx>, ix: NewOrderSingleIx) -> Result<()> {
    let book = &mut ctx.accounts.book.load_mut()?;
    let rb_filled_exec_reports = &mut ctx.accounts.rb_filled_exec_reports.load_mut()?;
    let rb_crank = &mut ctx.accounts.rb_crank.load_mut()?;

    let recv_token_account = match ix.is_buy {
        true => ctx.accounts.base_user_token_account.key(),
        false => ctx.accounts.quote_user_token_account.key(),
    };

    let order = book.new_limit(
        &ix,
        ctx.accounts.authority.key(),
        recv_token_account,
        rb_filled_exec_reports,
        rb_crank,
    );

    match ix.order_type {
        // All good. No checks required.
        OrderType::GTC => {}
        OrderType::FOK => require!(order.is_filled(), ErrorCode::FillOrKillFailed),
        OrderType::IOC => require!(
            order.get_leaves_qty() == 0,
            ErrorCode::ImmediateOrCancelError
        ),
        OrderType::MO => require!(!order.is_partially_filed(), ErrorCode::MakerOnlyFailed),
    };

    let instrmt_grp_seeds = &[
        b"instrmt-grp".as_ref(),
        &ctx.accounts.instrmt_grp.admin.as_ref(),
        &[ctx.accounts.instrmt_grp.bump],
    ];

    let signer = &[&instrmt_grp_seeds[..]];

    if ix.is_buy {
        // user deposit quote.
        // user receive base if partially filled.
        let leaves_cost: u64 = order.get_leaves_cost().checked_mul(order.price).unwrap();
        let user_deposit_qty = leaves_cost.checked_add(order.get_cum_cost()).unwrap();
        token::transfer(
            ctx.accounts.into_quote_transfer_user_to_vault(),
            user_deposit_qty,
        )?;

        let cpi_accounts = Transfer {
            from: ctx.accounts.base_vault.to_account_info().clone(),
            to: ctx
                .accounts
                .base_user_token_account
                .to_account_info()
                .clone(),
            authority: ctx.accounts.instrmt.to_account_info().clone(),
        };
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_context, order.get_cum_qty())?;
    } else {
        // user deposit base.
        // user receive quote but only the curent cumulative quantity.
        // For IOC get_leaves_qty() has been set to 0!
        let user_deposit_qty = order
            .get_cum_qty()
            .checked_add(order.get_leaves_qty())
            .unwrap();
        token::transfer(
            ctx.accounts.into_base_transfer_user_to_vault(),
            user_deposit_qty,
        )?;

        let cpi_accounts = Transfer {
            from: ctx.accounts.quote_vault.to_account_info().clone(),
            to: ctx.accounts.quote_user_token_account.to_account_info().clone(),
            authority: ctx.accounts.instrmt.to_account_info().clone(),
        };
        let cpi_context = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, signer);

        token::transfer(
            cpi_context,
            order.get_cum_cost(),
        )?;
    }
    Ok(())
}
