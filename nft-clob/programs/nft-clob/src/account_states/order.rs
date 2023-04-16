use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use super::NewOrderSingle;

#[zero_copy]
pub struct Order {
    pub price: u64,    // Limit price per unit of quantity.
    cum_qty: u64,      // Amount executed.
    leaves_qty: u64,   // Amount open for further execution.
    avg_px: u64,       // Average execution price.
    pub maker: Pubkey, // Order creator.
}

impl Order {
    pub fn new(price: u64, qty: u64, maker: Pubkey) -> Order {
        Order {
            price: price,
            cum_qty: 0,
            leaves_qty: qty,
            avg_px: 0,
            maker,
        }
    }

    pub fn is_partially_filed(&self) -> bool {
        self.cum_qty > 0
    }

    pub fn is_filled(&self) -> bool {
        self.leaves_qty == 0
    }

    pub fn is_better_than(&self, other: &Order, is_buy: bool) -> bool {
        if is_buy {
            self.price > other.price
        } else {
            self.price < other.price
        }
    }

    /// Existing order matches with a new incoming order.
    pub fn execute_trade(&mut self, new_order: &mut Order) {
        let match_qty;
        if new_order.leaves_qty >= self.leaves_qty {
            match_qty = self.leaves_qty;
        } else {
            match_qty = new_order.leaves_qty;
        }

        self.avg_px =
            (self.cum_qty * self.avg_px + match_qty * new_order.price) / (self.cum_qty + match_qty);

        self.cum_qty += match_qty;
        self.leaves_qty -= match_qty;
    }

    pub fn is_tombstone(&self) -> bool {
        self.leaves_qty == 0
    }

    pub fn space() -> usize {
        8 + (8 * 4) + 32
    }
}
