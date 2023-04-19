use anchor_lang::prelude::*;

use crate::enums::Side;

use super::Order;

const REPORT_SIZE: u8 = 200;

// Ring Buffer Filled Execution Report
#[zero_copy]
pub struct RingBufferFilledExecReport {
    pub filled_exec_reports: [FilledExecReport; REPORT_SIZE as usize],
    pub next_index: u8,
    pub padding: [u8; 7],
}
#[cfg(test)]
impl RingBufferFilledExecReport {
    pub fn new() -> Self {
        Self {
            filled_exec_reports: [FilledExecReport::new_empty(); REPORT_SIZE as usize],
            next_index: 0,
            padding: [0u8; 7],
        }
    }
}
#[cfg(test)]
impl FilledExecReport {
    pub fn new_empty() -> Self {
        Self {
            maker: Pubkey::default(),
            taker: Pubkey::default(),
            quantity: 0,
            price: 0,
            slot: 0,
            transact_time: 0,
            side: 0,
            padding: [0u8; 7],
        }
    }
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
    pub quantity: u64,      // Total quantity filled.
    pub price: u64,         // Unit price.
    pub slot: u64,          // Slot of execution.
    pub transact_time: i64, // Time of execution, expressed in UTC.
    pub side: u8,           // Buy = 0 or sell = 1.
    pub padding: [u8; 7],
}

impl FilledExecReport {
    pub fn new(
        maker: Pubkey,
        taker: Pubkey,
        quantity: u64,
        price: u64,
        is_buy: bool,
        slot: u64,
        transact_time: i64,
    ) -> Self {
        Self {
            maker,
            taker,
            quantity,
            price,
            slot,
            transact_time,
            side: if is_buy {
                Side::Buy as u8
            } else {
                Side::Sell as u8
            },
            padding: [0u8; 7],
        }
    }
    pub fn space() -> usize {
        32 * 2 + 8 * 4 + 1 + 7
    }
}
