use anchor_lang::prelude::*;

const REPORT_SIZE: u8 = 200;

// Ring Buffer Filled Execution Report
#[zero_copy]
pub struct RingBufferFilledExecReport {
    pub filled_exec_reports: [FilledExecReport; REPORT_SIZE as usize],
    pub next_index: u8,
    pub padding: [u8; 7],
}

impl RingBufferFilledExecReport {
    pub fn insert(&mut self, filled_exec_report: FilledExecReport) {
        self.filled_exec_reports[self.next_index as usize] = filled_exec_report;
        self.next_index = (self.next_index + 1) % REPORT_SIZE
    }

    pub fn space() -> usize {
        REPORT_SIZE as usize * FilledExecReport::space() + 1 + 7
    }
}

// Filled Execution Report
#[zero_copy]
pub struct FilledExecReport {
    pub maker: Pubkey,      // Seller.
    pub taker: Pubkey,      // Buyer.
    pub quantity: u64,      // Total quantitz filled.
    pub avg_px: u64,        // Average price.
    pub slot: u64,          // Slot of execution.
    pub transact_time: i64, // Time of execution, expressed in UTC.
    side: u8,               // Buy = 0 or sell = 1.
    pub padding: [u8; 7],
}

impl FilledExecReport {
    pub fn space() -> usize {
        32 * 2 + 8 * 4 + 1 + 7
    }
}
