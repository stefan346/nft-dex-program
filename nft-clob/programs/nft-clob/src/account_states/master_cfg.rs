use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;

/// Master config
///
/// Values can only be changed by an admin.
#[account]
pub struct MasterCfg {
    pub maker_fee_bps: u8, // Maker fee in basis points (BPS). Max BPS 255 = 2.55%.
    pub taker_fee_bps: u8, // Taker fee in basis points (BPS). Max BPS 255 = 2.55%.
    pub fee_exempt: Vec<Pubkey>, // Whitelisted clients such as market makers et cetera.
    pub fee_treasury: Pubkey, // Treasury wallet collecting fees for all instruments in its respective ATA accout.
    pub bump: u8,
}

impl MasterCfg {
    pub fn space(len: usize) -> usize {
        8 + 1 + (4 + 32 * len) + 32
    }
}
