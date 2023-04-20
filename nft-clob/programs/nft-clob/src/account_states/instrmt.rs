use anchor_lang::prelude::*;

/// Instrument
#[account]
#[derive(Default)]
pub struct Instrmt {
    pub base_mint: Pubkey,                  // Base currency.
    pub base_vault: Pubkey,                 // Vault to store base currency.
    pub quote_mint: Pubkey,                 // Quote currency.
    pub quote_vault: Pubkey,                // Vault to store quote currency.
    pub book: Pubkey,                       // Central limit order book.
    pub top_of_filled_exec_reports: Pubkey, // Execution reports for activity view.
    pub bumps: InstrmtBumps,                // Bumps,
}

#[derive(Default, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct InstrmtBumps {
    pub base_vault_bump: u8,
    pub quote_vault_bump: u8,
    pub instrmt_bump: u8,
}

impl Instrmt {
    pub fn space() -> usize {
        8 + 6 * 32 + InstrmtBumps::space() + 6
    }
}

impl InstrmtBumps {
    pub fn space() -> usize {
        1 + 1
    }
}
