use std::{mem::size_of, num::NonZeroUsize};

use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use super::Order;

/// Central Limit Order Book
#[account(zero_copy)]
pub struct Book {
    pub ask_min: u64, // Best ask
    pub bid_max: u64, // Best bid
    pub asks: Side,   // Ask side
    pub bids: Side,   // Bid side
}

impl Book {
    /// Process an incoming new order single.
    pub fn new_limit(&mut self, new_order_single: &NewOrderSingle, maker: Pubkey) {
        let mut new_order = new_order_single.into_order(maker);
        if new_order_single.is_buy {
            if new_order_single.limit >= self.ask_min {
                if !self.asks.is_empty() {
                    let mut pos = self.asks.head;
                    loop {
                        if self.asks.orders[pos as usize].order.price > new_order_single.limit {
                            break; // new order outside price range
                        }
                        self.asks.orders[pos as usize]
                            .order
                            .execute_trade(&mut new_order);

                        pos = match self.asks.next_order(pos) {
                            None => break,
                            Some(next_pos) => next_pos,
                        };

                        if self.asks.orders[pos as usize].order.is_filled() {
                            self.asks.remove_order(pos, true, false);
                            self.ask_min = self.asks.best_offer();
                        }

                        if new_order.is_filled() {
                            return; // new order is filled
                        }
                    }
                }
            }
        } else {
            if new_order_single.limit <= self.bid_max {
                if !self.bids.is_empty() {
                    let mut pos = self.bids.head;
                    loop {
                        if self.bids.orders[pos as usize].order.price < new_order_single.limit {
                            break; // outside price range
                        }

                        self.bids.orders[pos as usize]
                            .order
                            .execute_trade(&mut new_order);

                        pos = match self.bids.next_order(pos) {
                            None => break, // No more orders
                            Some(next_pos) => next_pos,
                        };

                        if self.bids.orders[pos as usize].order.is_filled() {
                            self.bids.remove_order(pos, true, false);
                            self.bid_max = self.bids.best_offer();
                        }

                        if new_order.is_filled() {
                            return; // filled
                        }
                    }
                }
            }
        }
        if new_order.is_partially_filed() {
            // insert new order in constant time O(1)
        } else {
            // insert new order in linear time O(n)
        }
    }
}

impl Book {
    pub fn space(bids_size: usize) -> usize {
        8
    }
}
const MAX_ORDERS: u16 = 128;

#[zero_copy]
pub struct Side {
    pub orders: [OrderHeader; MAX_ORDERS as usize],
    tombstone: i16,
    pub head: u16,
    pub tail: u16,
    padding: [u8; 10],
}

#[zero_copy]
pub struct OrderHeader {
    pub order: Order,
    pub next: u16,
    pub prev: u16,
    padding: [u8; 4],
}

impl Side {
    pub fn best_offer(&self) -> u64 {
        self.orders[self.head as usize].order.price
    }

    pub fn is_worse_than_worst(&self, new_order: Order, is_buy: bool) -> bool {
        if is_buy {
            self.orders[self.tail as usize].order.price > new_order.price
        } else {
            self.orders[self.tail as usize].order.price < new_order.price
        }
    }

    pub fn is_empty(&self) -> bool {
        self.orders[self.head as usize].order.is_tombstone()
    }

    pub fn is_head(&mut self, pos: u16) -> bool {
        self.head == pos
    }

    pub fn is_tail(&mut self, pos: u16) -> bool {
        self.tail == pos
    }

    /// Gets the next order index if exists.
    pub fn next_order(&mut self, pos: u16) -> Option<u16> {
        let next = self.orders[pos as usize].next;
        if self.orders[next as usize].order.is_tombstone() {
            return None;
        }
        Some(next)
    }

    pub fn next_tombstone(&mut self) -> Option<u16> {
        if self.tombstone == -1 {
            return None;
        }
        let ret = self.tombstone;
        if ret as u16 == MAX_ORDERS {
            self.tombstone = -1;
        }
        Some(ret as u16)
    }

    pub fn new_tombstone(&mut self, pos: u16) {
        if self.tombstone >= 0 {
            self.orders[pos as usize].next = self.tombstone as u16;
        }
        self.tombstone = pos as i16;
    }
    pub fn new_head(&mut self, pos: u16) {
        self.orders[pos as usize].prev = pos;
        self.orders[pos as usize].next = self.head;
        self.orders[self.head as usize].prev = pos;
        self.head = pos;
    }

    pub fn new_tail(&mut self, pos: u16) {
        self.orders[pos as usize].next = pos;
        self.orders[pos as usize].prev = self.tail;
        self.orders[self.tail as usize].next = pos;
        self.tail = pos;
    }

    pub fn new_before(&mut self, new: u16, before: u16) {
        let prev = self.orders[before as usize].prev;
        self.orders[new as usize].next = before;
        self.orders[new as usize].prev = prev;
        self.orders[prev as usize].next = new;
        self.orders[before as usize].prev = new;
    }

    // Constant O(1)
    pub fn remove_order(&mut self, ord_pos: u16, is_head: bool, is_tail: bool) {
        if self.is_head(ord_pos) {
            let next = self.orders[ord_pos as usize].next;
            self.orders[next as usize].prev = next;
            self.head = next;
        } else if self.is_tail(ord_pos) {
            let prev = self.orders[ord_pos as usize].prev;
            self.orders[prev as usize].next = prev;
            self.tail = prev;
        } else {
            let prev = self.orders[ord_pos as usize].prev;
            let next = self.orders[ord_pos as usize].next;

            self.orders[prev as usize].next = next;
            self.orders[next as usize].prev = prev;
        }

        self.new_tombstone(ord_pos);
    }

    pub fn insert_order(&mut self, new_order: Order, is_buy: bool) {
        // Insert in constant time O(1)
        if new_order.is_partially_filed() {
            match self.next_tombstone() {
                None => {
                    // Cancel tail
                }
                Some(pos) => {
                    self.orders[pos as usize].order = new_order;
                    self.new_head(pos);
                }
            };
        }
        // Insert in constant time O(1)
        else if self.is_worse_than_worst(new_order, is_buy) {
            // We reject the order if there are no next tombstone
            let pos = self.next_tombstone().unwrap();
            self.orders[pos as usize].order = new_order;
            self.new_tail(pos);
        }
        // Insert in linear time O(n)
        else {
            let pos = self.head;
            loop {
                let other = &self.orders[pos as usize].order;
                if new_order.is_better_than(other, is_buy) {
                    match self.next_tombstone() {
                        None => {
                            // Cancel tail
                        }
                        Some(tombstone_pos) => {
                            self.orders[tombstone_pos as usize].order = new_order;
                            self.new_before(tombstone_pos, pos);
                        }
                    }
                    break;
                }
            }
        }
    }
}

pub struct NewOrderSingle {
    pub is_buy: bool,
    pub limit: u64,
    pub size: u64,
}

impl NewOrderSingle {
    pub fn into_order(&self, maker: Pubkey) -> Order {
        Order::new(self.limit, self.size, maker)
    }
}

#[cfg(test)]
mod test {
    use std::mem::{self, size_of};

    use anchor_lang::prelude::Pubkey;

    use super::Book;
    #[test]
    fn it_should_add_orders() {
        let price: u64 = 7461734;
        for i in 1..31 {
            let value = (price + i) % 30;
            println!("VAL = {:?}", value);
        }
    }
}
