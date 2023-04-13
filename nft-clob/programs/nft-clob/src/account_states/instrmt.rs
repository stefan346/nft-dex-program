use anchor_lang::prelude::*;

use super::RingBufferFilledExecReport;

/// Instrument
#[account(zero_copy(unsafe))]
pub struct Instrmt {
    pub base_mint: Pubkey,                                      // Base currency.
    pub base_vault: Pubkey,                                     // Vault to store base currency.
    pub quote_mint: Pubkey,                                     // Quote currency.
    pub quote_vault: Pubkey,                                    // Vault to store quote currency.
    pub book: Pubkey,                                           // Central limit order book.
    pub top_of_filled_exec_reports: RingBufferFilledExecReport, // Execution reports for activity view.
}
