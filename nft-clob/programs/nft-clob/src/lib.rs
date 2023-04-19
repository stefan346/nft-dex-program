pub mod account_states;
pub mod constants;
pub mod enums;
pub mod errors;
pub mod instructions;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod nft_clob {
    use super::*;

    pub fn new_nft_pool(ctx: Context<NewNftPoolCtx>, ix: NewNftPoolIx) -> Result<()> {
        new_nft_pool::handler(ctx, ix)
    }

    pub fn init_master_cfg(ctx: Context<InitMasterCfgCtx>, ix: InitMasterCfgIx) -> Result<()> {
        init_master_cfg::handler(ctx, ix)
    }

    pub fn new_instrmt_grp(ctx: Context<NewInstrmtGrpCtx>) -> Result<()> {
        new_instrmt_grp::handler(ctx)
    }
}