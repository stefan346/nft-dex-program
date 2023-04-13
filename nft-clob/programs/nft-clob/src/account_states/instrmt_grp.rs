use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;

/// Instrument Group
///
/// Only an admin of the instrument group can make updates.
/// If we exceed 10KB we will create a new instrument group.
#[account]
pub struct InstrmtGrp {
    pub admin: Pubkey, // Owner of the instrument group.
    pub instrmts: Vec<Pubkey>,
    pub bump: u8,
}

impl InstrmtGrp {
    pub fn space(len: usize) -> usize {
        8 + (4 + 32 * len) + 1
    }
}
