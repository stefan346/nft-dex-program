use anchor_lang::prelude::*;

use super::FilledExecReport;

#[zero_copy]
#[derive(Debug)]
pub struct Order {
    pub limit: u64,           // Limit price per unit of quantity.
    cum_qty: u64,             // Amount executed.
    cum_cost: u64,            // Cost of executed amount.
    leaves_qty: u64,          // Amount open for further execution.
    pub maker: Pubkey,        // Order creator.
    pub payout_acc: Pubkey,   // Token account to receive the funds from an executed trade.
    pub payment_acc: Pubkey, // Token account for the deposit required to place an order.
}

#[cfg(test)]
impl Order {
    pub fn new_test(limit:u64, qty: u64) -> Self {
        Order {
            limit: limit,
            cum_qty: 0,
            cum_cost: 0,
            leaves_qty: qty,
            maker: Pubkey::default(),
            payout_acc: Pubkey::default(),
            payment_acc: Pubkey::default()
        }
    }
}

impl Order {
    /// Used for IOC orders as we don't want an order with a leaves qty after trade execution.
    pub fn clear_leaves_qty(&mut self) {
        self.leaves_qty = 0;
    }
    pub fn get_cum_cost(&self) -> u64 {
        self.cum_cost
    }

    pub fn get_leaves_cost(&self) -> Option<u64> {
        self.leaves_qty.checked_mul(self.limit)
    }

    pub fn new(
        limit: u64,
        qty: u64,
        maker: Pubkey,
        payout_acc: Pubkey,
        payment_acc: Pubkey,
    ) -> Order {
        Order {
            limit: limit,
            cum_qty: 0,
            cum_cost: 0,
            leaves_qty: qty,
            maker,
            payout_acc,
            payment_acc
        }
    }

    pub fn clear(&mut self) {
        self.limit = 0;
        self.cum_qty = 0;
        self.leaves_qty = 0;
        self.maker = Pubkey::default();
        self.payout_acc = Pubkey::default();
        self.payment_acc = Pubkey::default();
    }

    pub fn get_leaves_qty(&self) -> u64 {
        self.leaves_qty
    }

    pub fn get_cum_qty(&self) -> u64 {
        self.cum_qty
    }

    pub fn is_partially_filed(&self) -> bool {
        self.cum_qty > 0
    }

    pub fn is_filled(&self) -> bool {
        self.leaves_qty == 0
    }

    pub fn is_better_than(&self, other: &Order, is_buy: bool) -> bool {
        if other.limit == 0 {
            return true;
        }
        if is_buy {
            self.limit > other.limit
        } else {
            self.limit < other.limit
        }
    }

    /// Existing order matches with a new incoming order.
    pub fn execute_trade(
        &mut self,
        new_order: &mut Order,
        is_buy: bool,
    ) -> Result<FilledExecReport> {
        let match_qty;
        if new_order.leaves_qty >= self.leaves_qty {
            match_qty = self.leaves_qty;
        } else {
            match_qty = new_order.leaves_qty;
        }

        self.cum_qty += match_qty;
        self.leaves_qty -= match_qty;

        new_order.leaves_qty -= match_qty;
        new_order.cum_qty += match_qty;
        new_order.cum_cost += match_qty.checked_mul(self.limit).unwrap();

        let slot;
        let transact_time;

        if cfg!(test) {
            slot = 1;
            transact_time = 2;
        } else {
            slot = Clock::get()?.slot;
            transact_time = Clock::get()?.unix_timestamp;
        }
        Ok(FilledExecReport::new(
            self.maker,
            new_order.maker,
            match_qty,
            self.limit,
            is_buy,
            slot,
            transact_time,
        ))
    }

    pub fn is_tombstone(&self) -> bool {
        self.leaves_qty == 0
    }

    pub fn space() -> usize {
        8 + (8 * 4) + 32
    }
}
