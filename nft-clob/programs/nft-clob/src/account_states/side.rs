use anchor_lang::zero_copy;

use super::{Order, OrderHeader, MAX_ORDERS};

#[zero_copy]
pub struct Side {
    pub orders: [OrderHeader; MAX_ORDERS as usize],
    tombstone: u16,
    pub head: u16,
    pub tail: u16,
    padding: [u8; 10],
}

impl Side {
    pub fn new() -> Self {
        Self {
            orders: [OrderHeader::new(); MAX_ORDERS as usize],
            tombstone: 0,
            head: 0,
            tail: 0,
            padding: [0; 10],
        }
    }

    pub fn get_tombstone(&self) -> u16 {
        self.tombstone
    }

    pub fn best_offer(&self) -> u64 {
        self.orders[self.head as usize].order.price
    }

    pub fn is_better_than_best(&self, new_order: &Order, is_buy: bool) -> bool {
        let price = self.orders[self.head as usize].order.price;
        if price == 0 {
            return true;
        }
        if is_buy {
            new_order.price > price
        } else {
            new_order.price < price
        }
    }

    pub fn is_worse_than_worst(&self, new_order: Order, is_buy: bool) -> bool {
        let price = self.orders[self.tail as usize].order.price;
        if price == 0 {
            return false;
        }
        if is_buy {
            price >= new_order.price
        } else {
            price <= new_order.price
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
        // No more tombstones available
        if self.tombstone == MAX_ORDERS {
            return None;
        }
        let ret = self.tombstone;
        let tombstone = self.orders[self.tombstone as usize];

        // We can't assign next tombstone to what we return.
        if ret == tombstone.next {
            self.tombstone += 1;
        }
        // A tombstone can only be set to an actual tombstone.
        else if self.orders[tombstone.next as usize].order.is_tombstone() {
            self.tombstone = tombstone.next;
        }
        // We need to climb the ladder.
        else {
            self.tombstone += 1;
        }

        assert!(self.orders[ret as usize].order.is_tombstone());

        return Some(ret);
    }

    pub fn new_tombstone(&mut self, pos: u16) {
        self.orders[pos as usize].next = self.tombstone;
        self.tombstone = pos;
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
    pub fn remove_order(&mut self, ord_pos: u16) {
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

        self.orders[ord_pos as usize].order.clear();
    }

    pub fn insert_order(&mut self, new_order: Order, is_buy: bool) {
        // Insert in constant time O(1)
        if new_order.is_partially_filed() || self.is_better_than_best(&new_order, is_buy) {
            let new_order_pos = match self.next_tombstone() {
                None => {
                    // Cancel tail
                    let tail = self.tail;
                    self.remove_order(self.tail);
                    tail
                }
                Some(pos) => pos,
            };
            self.orders[new_order_pos as usize].order = new_order;
            self.new_head(new_order_pos);
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
            let mut pos = self.head;
            loop {
                let other = &self.orders[pos as usize].order;
                if new_order.is_better_than(other, is_buy) {
                    let new_order_pos = match self.next_tombstone() {
                        None => {
                            // Cancel tail
                            let tail = self.tail;
                            self.remove_order(self.tail);
                            tail
                        }
                        Some(tombstone_pos) => tombstone_pos,
                    };
                    self.orders[new_order_pos as usize].order = new_order;
                    self.new_before(new_order_pos, pos);
                    break;
                }

                pos = match self.next_order(pos) {
                    None => panic!("Failed to find pos for new order in linear time!"),
                    Some(next_pos) => next_pos,
                };
            }
        }
    }
}
