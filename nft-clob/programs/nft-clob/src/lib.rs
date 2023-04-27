pub mod account_states;
pub mod constants;
pub mod enums;
pub mod errors;
pub mod instructions;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("5pEBe82QbyavqRpBZHwGJLi8RofJaLWbP2FSDRLoUWZM");

#[program]
pub mod nft_clob {
    use super::*;

    pub fn cancel_order(ctx: Context<CancelOrderCtx>, ix: CancelOrderIx) -> Result<()> {
        cancel_order::handler(ctx, ix)
    }

    pub fn crank(ctx: Context<CrankCtx>) -> Result<()> {
        crank::handler(ctx)
    }

    pub fn init_master_cfg(ctx: Context<InitMasterCfgCtx>, ix: InitMasterCfgIx) -> Result<()> {
        init_master_cfg::handler(ctx, ix)
    }

    pub fn new_instrmt_grp(ctx: Context<NewInstrmtGrpCtx>) -> Result<()> {
        new_instrmt_grp::handler(ctx)
    }

    pub fn new_instrmt(ctx: Context<NewInstrmtCtx>, ix: NewInstrmtIx) -> Result<()> {
        new_instrmt::handler(ctx, ix)
    }

    pub fn new_nft_pool(ctx: Context<NewNftPoolCtx>, ix: NewNftPoolIx) -> Result<()> {
        new_nft_pool::handler(ctx, ix)
    }




}
