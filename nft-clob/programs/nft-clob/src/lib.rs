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

#[derive(Accounts)]
pub struct Initialize {}

pub fn sort_arr<T: Ord>(arr: &mut [T]) {
    sorting::bubble_sort(arr);
}

mod sorting {
    pub fn selection_sort<T: Ord>(arr: &mut [T]) {
        let len = arr.len();
        for i in 0..len {
            let mut min_idx = i;
            for j in (i + 1)..len {
                if arr[j] < arr[min_idx] {
                    min_idx = j;
                }
            }
            arr.swap(min_idx, i);
        }
    }

    pub fn bubble_sort<T: Ord>(arr: &mut [T]) {
        let len = arr.len();
        for i in 0..len {
            for j in 0..len - i - 1 {
                if arr[j] > arr[j + 1] {
                    arr.swap(j, j + 1);
                }
            }
        }
    }
}
