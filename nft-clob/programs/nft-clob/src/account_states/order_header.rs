use anchor_lang::{zero_copy, prelude::Pubkey};

use super::Order;

#[zero_copy]
#[derive(Debug)]
pub struct OrderHeader {
    pub order: Order,
    pub next: u16,
    pub prev: u16,
    padding: [u8; 4],
}

impl OrderHeader {
    pub fn new() -> Self {
        Self {
            order: Order::new(0, 0, Pubkey::default(),Pubkey::default()),
            next: 0,
            prev: 0,
            padding: [0; 4],
        }
    }
}