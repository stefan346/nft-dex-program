use anchor_lang::prelude::*;

use super::FilledExecReport;

#[zero_copy]
#[derive(Debug)]
pub struct Order {
    pub price: u64,    // Limit price per unit of quantity.
    cum_qty: u64,      // Amount executed.
    cum_cost: u64,     // Cost of executed amount.
    leaves_qty: u64,   // Amount open for further execution.
    pub maker: Pubkey, // Order creator.
}

impl Order {
    pub fn get_cum_cost(&self) -> u64 {
        self.cum_cost
    }

    pub fn get_leaves_cost(&self) -> u64 {
        self.leaves_qty * self.price
    }

    pub fn new(price: u64, qty: u64, maker: Pubkey) -> Order {
        Order {
            price: price,
            cum_qty: 0,
            cum_cost: 0,
            leaves_qty: qty,
            maker,
        }
    }

    pub fn clear(&mut self) {
        self.price = 0;
        self.cum_qty = 0;
        self.leaves_qty = 0;
        self.maker = Pubkey::default()
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
        if other.price == 0 {
            return true;
        }
        if is_buy {
            self.price > other.price
        } else {
            self.price < other.price
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
        new_order.cum_cost += match_qty.checked_mul(self.price).unwrap();

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
            self.price,
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
