use anchor_lang::prelude::*;

use crate::account_states::*;
use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct NewOrderSingleIx {
    pub is_buy: bool,
    pub limit: u64,
    pub size: u64,
    pub order_type: OrderType
}

#[derive(Accounts)]
pub struct NewOrderSingleCtx<'info> {
    
}

pub fn handler(ctx: Context<Ctx>, ix: Ix) -> Result<()> {
    Ok(())
}
