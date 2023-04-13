use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;

#[account]
pub struct NftPool {
    pub verified_creators: Vec<Pubkey>,
    pub mint: Pubkey,
    pub bump: u8,
}

impl NftPool {
    pub fn space(len: usize) -> usize {
        8 + (4 + 32 * len) + 32 + 1
    }
}
