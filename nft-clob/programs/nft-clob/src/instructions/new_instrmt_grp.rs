use anchor_lang::prelude::*;

use crate::account_states::*;

#[derive(Accounts)]
pub struct NewInstrmtGrpCtx<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        seeds = [b"instrmt-grp", authority.key().as_ref()],
        bump,
        space = InstrmtGrp::space(0)
    )]
    pub instrmt_grp: Box<Account<'info, InstrmtGrp>>,

    #[account(zero)]
    pub rb_crank: AccountLoader<'info, RingBufferCrank>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<NewInstrmtGrpCtx>) -> Result<()> {
    let instrmt_grp = &mut ctx.accounts.instrmt_grp;
    let rb_crank = &mut ctx.accounts.rb_crank.load_init()?;

    rb_crank.instrmt_grp = instrmt_grp.key();

    rb_crank.head = 0;
    rb_crank.next = 1;

    instrmt_grp.admin = ctx.accounts.authority.key();
    instrmt_grp.bump = *ctx.bumps.get("instrmt_grp").unwrap();

    Ok(())
}
