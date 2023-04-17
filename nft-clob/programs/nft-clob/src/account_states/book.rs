use anchor_lang::prelude::*;

use super::{Order, Side};

pub const MAX_ORDERS: u16 = 2000;

/// Central Limit Order Book
#[account(zero_copy)]
pub struct Book {
    pub ask_min: u64, // Best ask
    pub bid_max: u64, // Best bid
    pub asks: Side,   // Ask side
    pub bids: Side,   // Bid side
}

impl Book {
    pub fn new() -> Self {
        Book {
            ask_min: 0,
            bid_max: 0,
            asks: Side::new(),
            bids: Side::new(),
        }
    }
    /// Process an incoming new order single.
    pub fn new_limit(&mut self, nos: &NewOrderSingle, maker: Pubkey) {
        let mut new_order = nos.into_order(maker);
        if nos.is_buy {
            if nos.limit >= self.ask_min {
                if !self.asks.is_empty() {
                    let mut pos = self.asks.head;
                    loop {
                        if self.asks.orders[pos as usize].order.price > nos.limit {
                            break; // new order outside price range
                        }
                        self.asks.orders[pos as usize]
                            .order
                            .execute_trade(&mut new_order);

                        if self.asks.orders[pos as usize].order.is_filled() {
                            self.asks.remove_order(pos);
                            self.ask_min = self.asks.best_offer();
                        }

                        pos = match self.asks.next_order(pos) {
                            None => break,
                            Some(next_pos) => next_pos,
                        };

                        if new_order.is_filled() {
                            return; // new order is filled
                        }
                        println!("1");
                    }
                }
            }
            self.bids.insert_order(new_order, nos.is_buy);
            if nos.limit > self.bid_max {
                self.bid_max = nos.limit;
            }
        } else {
            if nos.limit <= self.bid_max {
                if !self.bids.is_empty() {
                    let mut pos = self.bids.head;
                    loop {
                        if self.bids.orders[pos as usize].order.price < nos.limit {
                            break; // outside price range
                        }

                        self.bids.orders[pos as usize]
                            .order
                            .execute_trade(&mut new_order);

                        if self.bids.orders[pos as usize].order.is_filled() {
                            self.bids.remove_order(pos);
                            self.bid_max = self.bids.best_offer();
                        }

                        pos = match self.bids.next_order(pos) {
                            None => break, // No more orders
                            Some(next_pos) => next_pos,
                        };

                        if new_order.is_filled() {
                            return; // filled
                        }
                    }
                }
            }
            self.asks.insert_order(new_order, nos.is_buy);
            if self.ask_min == 0 || nos.limit < self.ask_min {
                self.ask_min = nos.limit;
            }
        }
    }
}

impl Book {
    pub fn space(bids_size: usize) -> usize {
        8
    }
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct NewOrderSingle {
    pub is_buy: bool,
    pub limit: u64,
    pub size: u64,
}

impl NewOrderSingle {
    pub fn new(is_buy: bool, limit: u64, size: u64) -> Self {
        NewOrderSingle {
            is_buy,
            limit,
            size,
        }
    }
    pub fn into_order(&self, maker: Pubkey) -> Order {
        Order::new(self.limit, self.size, maker)
    }
}

#[cfg(test)]
mod test {
    use std::mem::{self, size_of};

    use super::{Book, NewOrderSingle, Side, MAX_ORDERS};
    use anchor_lang::prelude::Pubkey;
    use quickcheck_macros::quickcheck;

    #[test]
    fn it_should_add_single_order_to_both_sides() {
        let mut book = Book::new();

        let sell_maker = Pubkey::new_unique();
        let buy_maker = Pubkey::new_unique();

        let sell_nos = NewOrderSingle::new(false, 10, 3);
        let buy_nos = NewOrderSingle::new(true, 9, 2);

        book.new_limit(&sell_nos, sell_maker);
        book.new_limit(&buy_nos, buy_maker);

        assert_eq!(book.ask_min, 10);
        assert_eq!(book.asks.orders[0].order.price, 10);
        assert_eq!(book.asks.orders[0].order.get_leaves_qty(), 3);
        assert_eq!(book.asks.orders[0].order.get_cum_qty(), 0);
        assert_eq!(book.asks.orders[0].order.maker, sell_maker);

        assert_eq!(book.bid_max, 9);
        assert_eq!(book.bids.orders[0].order.price, 9);
        assert_eq!(book.bids.orders[0].order.get_leaves_qty(), 2);
        assert_eq!(book.bids.orders[0].order.get_cum_qty(), 0);
        assert_eq!(book.bids.orders[0].order.maker, buy_maker);
    }

    #[quickcheck]
    fn it_should_add_many_orders_to_asks_side(mut sell_limits: Vec<u64>) -> bool {
        let mut book = Book::new();

        let maker = Pubkey::new_unique();
        let size = 2;
        for i in sell_limits.iter() {
            if *i == 0 {
                continue;
            }
            let sell_nos = NewOrderSingle::new(false, i.clone(), size);
            book.new_limit(&sell_nos, maker);
        }

        sell_limits.sort();
        let mut cur_order_pos = book.asks.head;
        for i in sell_limits.into_iter() {
            if i == 0 {
                continue;
            }
            let order = book.asks.orders[cur_order_pos as usize];
            assert_eq!(order.order.price, i);
            cur_order_pos = order.next;
        }
        true
    }

    #[test]
    fn it_should_add_many_orders_to_bids_side() {
        let mut book = Book::new();

        let maker = Pubkey::new_unique();
        let size = 2;

        let mut buy_limits = [10, 11, 12, 13, 14, 15, 16, 9, 25, 12, 8, 7, 8, 6, 6, 19];
        for i in buy_limits {
            let buy_nos = NewOrderSingle::new(true, i, size);
            book.new_limit(&buy_nos, maker);
        }

        buy_limits.sort_by(|a, b| b.cmp(a));
        let mut cur_order_pos = book.bids.head;
        for i in buy_limits {
            let order = book.bids.orders[cur_order_pos as usize];
            assert_eq!(order.order.price, i);
            cur_order_pos = order.next;
        }
    }
    #[test]
    fn it_should_add_many_orders_with_incremental_price() {
        let mut book = Book::new();

        let maker = Pubkey::new_unique();
        let size = 2;

        for i in 1..MAX_ORDERS {
            let buy_nos = NewOrderSingle::new(true, i as u64, size);
            book.new_limit(&buy_nos, maker);
        }

        for i in 1..MAX_ORDERS {
            let sell_nos = NewOrderSingle::new(false, i as u64, size);
            book.new_limit(&sell_nos, maker);
        }
    }

    #[test]
    fn it_should_match_a_few_orders() {
        let mut book = Book::new();

        let maker = Pubkey::new_unique();

        let buy_nos_1 = NewOrderSingle::new(true, 11 as u64, 2);
        book.new_limit(&buy_nos_1, maker);
        let buy_nos_2 = NewOrderSingle::new(true, 10 as u64, 4);
        book.new_limit(&buy_nos_2, maker);

        let sell_nos_1 = NewOrderSingle::new(false, 10, 1);
        book.new_limit(&sell_nos_1, maker);

        assert_eq!(book.asks.orders[0].order.get_leaves_qty(), 0);
        assert_eq!(book.asks.orders[0].order.get_cum_qty(), 0);

        assert_eq!(book.bids.orders[0].order.get_leaves_qty(), 1);
        assert_eq!(book.bids.orders[0].order.get_cum_qty(), 1);
        assert_eq!(book.bids.orders[0].order.price, 11);

        assert_eq!(book.bids.orders[1].order.get_leaves_qty(), 4);
        assert_eq!(book.bids.orders[1].order.get_cum_qty(), 0);
        assert_eq!(book.bids.orders[1].order.price, 10);

        let sell_nos_2 = NewOrderSingle::new(false, 10, 2);
        book.new_limit(&sell_nos_2, maker);

        // assert_eq!(book.bids.orders[0].order.get_leaves_qty(), 0);
        // assert_eq!(book.bids.orders[0].order.get_cum_qty(), 0);
        // assert_eq!(book.bids.orders[0].order.price, 0);

        // assert_eq!(book.bids.orders[1].order.get_leaves_qty(), 3);
        // assert_eq!(book.bids.orders[1].order.get_cum_qty(), 1);
        // assert_eq!(book.bids.orders[1].order.price, 10);

        println!(
            "Book bids: {:?} {:?} {:?}",
            book.bids.orders[0], book.bids.orders[1], book.bids.orders[2]
        );
        println!(
            "Book asks: {:?} {:?} {:?}",
            book.asks.orders[0], book.asks.orders[1], book.asks.orders[2]
        );
    }

    #[quickcheck]
    fn it_should_match_many_orders(mut buy: Vec<(u64, u64)>, mut sell: Vec<(u64, u64)>) -> bool {
        buy = buy
            .into_iter()
            .filter(|(x, y)| *x != 0 && *y != 0)
            .collect();
        sell = sell
            .into_iter()
            .filter(|(x, y)| *x != 0 && *y != 0)
            .collect();
        println!("buy {}, sell {}", buy.len(), sell.len());
        let mut book = Book::new();
        let mut clone_buy = buy.clone();
        let mut clone_sell = sell.clone();

        let mut sort_buy: Vec<(u64, u64)> = Vec::new();
        let mut sort_sell: Vec<(u64, u64)> = Vec::new();

        loop {
            if let Some((bid_price, mut bid_size)) = clone_buy.pop() {
                for (ask_price, mut ask_size) in sort_sell.iter_mut() {
                    if bid_price >= *ask_price {
                        if ask_size >= bid_size {
                            ask_size -= bid_size;
                            bid_size = 0;
                        } else {
                            bid_size -= ask_size;
                            ask_size = 0;
                        }
                    } else {
                        break;
                    }
                    if bid_size == 0 {
                        break;
                    }
                }
                sort_buy.push((bid_price, bid_size));
                // Order sort_buy descending
                sort_buy.sort_by(|(ax, bx), (ay, by)| ay.cmp(ax));
            }

            if let Some((ask_price, mut ask_size)) = clone_sell.pop() {
                for (bid_price, mut bid_size) in sort_buy.iter_mut() {
                    if ask_price <= *bid_price {
                        if ask_size >= bid_size {
                            ask_size -= bid_size;
                            bid_size = 0;
                        } else {
                            bid_size -= ask_size;
                            ask_size = 0;
                        }
                    } else {
                        break;
                    }
                    if ask_size == 0 {
                        break;
                    }
                }
                sort_sell.push((ask_price, ask_size));
                // sort ascending
                sort_buy.sort_by(|(ax, bx), (ay, by)| ax.cmp(ay));
            }

            if clone_buy.len() == 0 && clone_sell.len() == 0 {
                break;
            }
        }

        loop {
            if let Some((bid_price, bid_size)) = buy.pop() {
                let buy_nos = NewOrderSingle::new(true, bid_price, bid_size);
                book.new_limit(&buy_nos, Pubkey::new_unique());
            }
            if let Some((asks_price, ask_size)) = sell.pop() {
                let buy_nos = NewOrderSingle::new(true, asks_price, ask_size);
                book.new_limit(&buy_nos, Pubkey::new_unique());
            }

            if buy.len() == 0 && sell.len() == 0 {
                break;
            }
        }
        let mut len = 0;
        let mut cur_bid_pos = book.bids.head;
        loop {
            let order = book.bids.orders[cur_bid_pos as usize];
            if order.order.is_tombstone() {
                break;
            }
            len += 1;
            cur_bid_pos = order.next;

            if book.bids.tail == cur_bid_pos {
                break;
            }
        }

        println!("len {} - {}", len, sort_buy.len());

        // sort_buy = sort_buy.into_iter().filter(|(_, size)| *size != 0).collect();
        // let mut cur_bid_pos = book.bids.head;
        // for (price, size) in sort_buy {
        //     let order = book.bids.orders[cur_bid_pos as usize];
        //     assert_eq!(order.order.price, price);
        //     cur_bid_pos = order.next;
        // }

        true
    }
}
