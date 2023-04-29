use anchor_lang::prelude::*;

/// Instrument
#[account]
#[derive(Default)]
pub struct Instrmt {
    pub base_symbol: [u8; 10],          // Symbol for base, eg. BTC.
    pub quote_symbol: [u8; 10],         // Symbol for quote, eg. USD.
    pub instrmt_grp: Pubkey,            // Instrument group instrument belongs to.
    pub base_mint: Pubkey,              // Base currency.
    pub base_vault: Pubkey,             // Vault to store base currency.
    pub quote_mint: Pubkey,             // Quote currency.
    pub quote_vault: Pubkey,            // Vault to store quote currency.
    pub book: Pubkey,                   // Central limit order book.
    pub rb_filled_exec_reports: Pubkey, // Execution reports for activity view.
    pub bumps: InstrmtBumps,            // Bumps,
}

#[derive(Default, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct InstrmtBumps {
    pub base_vault_bump: u8,
    pub quote_vault_bump: u8,
    pub instrmt_bump: u8,
}

impl Instrmt {
    pub fn space() -> usize {
        8  + 20 + 7 * 32 + InstrmtBumps::space()
    }

    pub fn to_u8_array(a: String) -> [u8; 10] {
        let src = a.as_bytes();
        let mut dest = [0u8; 10];
        dest[..src.len()].copy_from_slice(src);
        dest
    }
}

impl InstrmtBumps {
    pub fn space() -> usize {
        1 + 1 + 1
    }
}
