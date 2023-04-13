use anchor_lang::prelude::*;

use crate::account_states::*;
use crate::constants::ADMIN_PUBKEY_STR;
use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitMasterCfgIx {
    pub maker_fee_bps: u8,
    pub taker_fee_bps: u8,
    pub fee_treasury: Pubkey,
}

#[derive(Accounts)]
pub struct InitMasterCfgCtx<'info> {
    #[account(
        mut,
        constraint = admin.key.to_string() == ADMIN_PUBKEY_STR @ ErrorCode::NotAdmin
    )]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [b"master-cfg", admin.key().as_ref()],
        bump,
        space = MasterCfg::space(0)
    )]
    pub master_cfg: Box<Account<'info, MasterCfg>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitMasterCfgCtx>, ix: InitMasterCfgIx) -> Result<()> {
    let master_cfg = &mut ctx.accounts.master_cfg;

    master_cfg.maker_fee_bps = ix.maker_fee_bps;
    master_cfg.taker_fee_bps = ix.taker_fee_bps;
    master_cfg.fee_treasury = ix.fee_treasury;
    master_cfg.bump = *ctx.bumps.get("master_cfg").unwrap();

    Ok(())
}
