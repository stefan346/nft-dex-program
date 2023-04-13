use anchor_lang::prelude::*;

#[zero_copy]
pub struct Order {
    pub price: u64,      // Limit price per unit of quantity.
    pub cum_qty: u64,    // Amount executed.
    pub leaves_qty: u64, // Amount open for further execution.
    pub avg_px: u64,     // Average execution price.
    pub maker: Pubkey,   // Order creator.
}

impl Order {
    pub fn new(price: u64, qty: u64, maker: Pubkey) -> Order {
        Order {
            price: price,
            cum_qty: 0,
            leaves_qty: qty,
            avg_px: 0,
            maker: maker,
        }
    }
    pub fn space() -> usize {
        8 + (8 * 4) + 32
    }
}
