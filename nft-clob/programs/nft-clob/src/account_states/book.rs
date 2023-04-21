use anchor_lang::prelude::*;

use crate::{enums::OrderType, instructions::new_order_single::NewOrderSingleIx};

use super::{Crank, Order, RingBufferCrank, RingBufferFilledExecReport, Side};

pub const MAX_ORDERS: u16 = 2048;

/// Central Limit Order Book
#[account(zero_copy)]
pub struct Book {
    pub instrmt: Pubkey, // Instrument book belongs to
    pub ask_min: u64,    // Best ask
    pub bid_max: u64,    // Best bid
    pub asks: Side,      // Ask side
    pub bids: Side,      // Bid side
}

#[cfg(test)]
impl Book {
    pub fn new() -> Self {
        Book {
            instrmt: Pubkey::default(),
            ask_min: 0,
            bid_max: 0,
            asks: Side::new(),
            bids: Side::new(),
        }
    }
}

impl Book {
    /// When buying you deposit quote to create a bid
    pub fn get_quote_vault(&self) -> Pubkey {
        self.bids.vault
    }
    pub fn set_quote_vault(&mut self, vault: Pubkey) {
        self.bids.vault = vault;
    }
    /// When selling you deposit base to create an ask
    pub fn get_base_vault(&self) -> Pubkey {
        self.asks.vault
    }
    pub fn set_base_vault(&mut self, vault: Pubkey) {
        self.asks.vault = vault;
    }

    pub fn new_limit_from_nos_ix(
        &mut self,
        nos: &NewOrderSingleIx,
        maker: Pubkey,
        payout_acc: Pubkey,
        payment_acc: Pubkey,
        rb_filled_exec_report: &mut RingBufferFilledExecReport,
        rb_crank: &mut RingBufferCrank,
    ) -> Order {
        let new_order = nos.into_order(maker, payout_acc, payment_acc);
        self.new_limit(
            new_order,
            nos.order_type,
            nos.is_buy,
            rb_filled_exec_report,
            rb_crank,
        )
    }

    /// Process an incoming new order single.
    pub fn new_limit(
        &mut self,
        mut new_order: Order,
        order_type: OrderType,
        is_buy: bool,
        rb_filled_exec_report: &mut RingBufferFilledExecReport,
        rb_crank: &mut RingBufferCrank,
    ) -> Order {
        let is_match = match is_buy {
            true => |order_price: u64, nos_limit: u64| -> bool { order_price <= nos_limit },
            false => |order_price: u64, nos_limit: u64| -> bool { order_price >= nos_limit },
        };

        let crank_vault = match is_buy {
            true => self.get_quote_vault(),
            false => self.get_base_vault(),
        };

        let (match_side, has_matches) = match is_buy {
            true => (&mut self.asks, new_order.limit >= self.ask_min),
            false => (&mut self.bids, new_order.limit <= self.bid_max),
        };

        if !match_side.is_empty() {
            if has_matches {
                let mut pos = match_side.head;
                loop {
                    if !is_match(match_side.orders[pos as usize].order.limit, new_order.limit) {
                        break; // new order outside price range
                    }
                    let filled_exec_report = match_side.orders[pos as usize]
                        .order
                        .execute_trade(&mut new_order, is_buy)
                        .unwrap();
                    rb_filled_exec_report.insert(filled_exec_report);

                    rb_crank.insert(
                        crank_vault,
                        match_side.orders[pos as usize].order.payout_acc,
                        is_buy,
                        match_side.orders[pos as usize].order.maker,
                        filled_exec_report.quantity,
                        filled_exec_report.price,
                    );

                    let next_pos = match_side.next_order(pos);

                    if match_side.orders[pos as usize].order.is_filled() {
                        match_side.remove_order(pos);
                        match is_buy {
                            true => self.ask_min = match_side.best_offer(),
                            false => self.bid_max = match_side.best_offer(),
                        };
                    }

                    if new_order.is_filled() {
                        return new_order; // new order is filled
                    }

                    pos = match next_pos {
                        None => break,
                        Some(next_pos) => next_pos,
                    };
                }
            }
        }

        if order_type == OrderType::IOC {
            new_order.clear_leaves_qty();
            return new_order;
        }

        match is_buy {
            true => {
                self.bids.insert_order(new_order, is_buy, rb_crank);
                if new_order.limit > self.bid_max {
                    self.bid_max = new_order.limit;
                }
            }
            false => {
                self.asks.insert_order(new_order, is_buy, rb_crank);
                if self.ask_min == 0 || new_order.limit < self.ask_min {
                    self.ask_min = new_order.limit;
                }
            }
        }
        return new_order;
    }
}

impl NewOrderSingleIx {
    pub fn new(is_buy: bool, limit: u64, size: u64) -> Self {
        NewOrderSingleIx {
            is_buy,
            limit,
            size,
            order_type: crate::enums::OrderType::GTC,
        }
    }
    pub fn into_order(&self, maker: Pubkey, payout_acc: Pubkey, payment_acc: Pubkey) -> Order {
        Order::new(self.limit, self.size, maker, payout_acc, payment_acc)
    }
}

#[cfg(test)]
mod test {

    use crate::{
        account_states::{Order, RingBufferCrank, RingBufferFilledExecReport},
        enums::OrderType,
    };

    use super::{Book, NewOrderSingleIx, MAX_ORDERS};
    use anchor_lang::prelude::Pubkey;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    const GTC: OrderType = OrderType::GTC;
    #[test]
    fn it_should_add_single_order_to_both_sides() {
        let mut book = Book::new();
        let mut rb = RingBufferFilledExecReport::new();
        let mut rb_crank = RingBufferCrank::new();
        let sell_maker = Pubkey::new_unique();
        let buy_maker = Pubkey::new_unique();

        let sell_nos = Order::new_test(10, 3);
        let buy_nos = Order::new_test(9, 2);

        book.new_limit(sell_nos, GTC, false, &mut rb, &mut rb_crank);
        book.new_limit(buy_nos, GTC, true, &mut rb, &mut rb_crank);

        assert_eq!(book.ask_min, 10);
        assert_eq!(book.asks.orders[0].order.limit, 10);
        assert_eq!(book.asks.orders[0].order.get_leaves_qty(), 3);
        assert_eq!(book.asks.orders[0].order.get_cum_qty(), 0);

        assert_eq!(book.bid_max, 9);
        assert_eq!(book.bids.orders[0].order.limit, 9);
        assert_eq!(book.bids.orders[0].order.get_leaves_qty(), 2);
        assert_eq!(book.bids.orders[0].order.get_cum_qty(), 0);
    }

    #[quickcheck]
    fn it_should_add_many_orders_to_asks_side(mut sell_limits: Vec<u64>) -> bool {
        let mut book = Book::new();
        let mut rb = RingBufferFilledExecReport::new();
        let mut rb_crank = RingBufferCrank::new();
        let maker = Pubkey::new_unique();
        let recv_funds = Pubkey::new_unique();
        let size = 2;
        for i in sell_limits.iter() {
            if *i == 0 {
                continue;
            }
            let sell_nos = Order::new_test(i.clone(), size);
            book.new_limit(sell_nos, GTC, false, &mut rb, &mut rb_crank);
        }

        sell_limits.sort();
        let mut cur_order_pos = book.asks.head;
        for i in sell_limits.into_iter() {
            if i == 0 {
                continue;
            }
            let order = book.asks.orders[cur_order_pos as usize];
            assert_eq!(order.order.limit, i);
            cur_order_pos = order.next;
        }
        true
    }

    #[test]
    fn it_should_add_many_orders_to_bids_side() {
        let mut book = Book::new();
        let mut rb = RingBufferFilledExecReport::new();
        let mut rb_crank = RingBufferCrank::new();
        let maker = Pubkey::new_unique();
        let recv_funds = Pubkey::new_unique();
        let size = 2;

        let mut buy_limits = [10, 11, 12, 13, 14, 15, 16, 9, 25, 12, 8, 7, 8, 6, 6, 19];
        for i in buy_limits {
            let buy_nos = Order::new_test(i, size);
            book.new_limit(buy_nos, GTC, true, &mut rb, &mut rb_crank);
        }

        buy_limits.sort_by(|a, b| b.cmp(a));
        let mut cur_order_pos = book.bids.head;
        for i in buy_limits {
            let order = book.bids.orders[cur_order_pos as usize];
            assert_eq!(order.order.limit, i);
            cur_order_pos = order.next;
        }
    }
    #[test]
    fn it_should_add_many_orders_with_incremental_price() {
        let mut book = Book::new();
        let mut rb = RingBufferFilledExecReport::new();
        let mut rb_crank = RingBufferCrank::new();
        let maker = Pubkey::new_unique();
        let recv_funds = Pubkey::new_unique();
        let size = 2;

        for i in 1..MAX_ORDERS {
            let buy_nos = Order::new_test(i as u64, size);
            book.new_limit(buy_nos, GTC, true, &mut rb, &mut rb_crank);
        }

        for i in 1..MAX_ORDERS {
            let sell_nos = Order::new_test(i as u64, size);
            book.new_limit(sell_nos, GTC, false, &mut rb, &mut rb_crank);
        }
    }

    #[test]
    fn it_should_match_a_few_orders() {
        let mut book = Book::new();
        let mut rb = RingBufferFilledExecReport::new();
        let mut rb_crank = RingBufferCrank::new();
        let maker = Pubkey::new_unique();
        let recv_funds = Pubkey::new_unique();

        let buy_nos_1 = Order::new_test(11 as u64, 2);
        book.new_limit(buy_nos_1, GTC, true, &mut rb, &mut rb_crank);
        let buy_nos_2 = Order::new_test(10, 4);
        book.new_limit(buy_nos_2, GTC, true, &mut rb, &mut rb_crank);

        let sell_nos_1 = Order::new_test(10, 1);
        book.new_limit(sell_nos_1, GTC, false, &mut rb, &mut rb_crank);

        assert_eq!(book.asks.orders[0].order.get_leaves_qty(), 0);
        assert_eq!(book.asks.orders[0].order.get_cum_qty(), 0);

        assert_eq!(book.bids.orders[0].order.get_leaves_qty(), 1);
        assert_eq!(book.bids.orders[0].order.get_cum_qty(), 1);
        assert_eq!(book.bids.orders[0].order.limit, 11);

        assert_eq!(book.bids.orders[1].order.get_leaves_qty(), 4);
        assert_eq!(book.bids.orders[1].order.get_cum_qty(), 0);
        assert_eq!(book.bids.orders[1].order.limit, 10);

        let sell_nos_2 = Order::new_test(10, 2);
        book.new_limit(sell_nos_2, GTC, false, &mut rb, &mut rb_crank);

        assert_eq!(book.bids.orders[0].order.get_leaves_qty(), 0);
        assert_eq!(book.bids.orders[0].order.get_cum_qty(), 0);
        assert_eq!(book.bids.orders[0].order.limit, 0);

        assert_eq!(book.bids.orders[1].order.get_leaves_qty(), 3);
        assert_eq!(book.bids.orders[1].order.get_cum_qty(), 1);
        assert_eq!(book.bids.orders[1].order.limit, 10);
    }

    #[test]
    fn it_should_place_a_few_orders_1() {
        let mut book = Book::new();
        let mut rb = RingBufferFilledExecReport::new();
        let mut rb_crank = RingBufferCrank::new();
        let maker = Pubkey::new_unique();
        let recv_funds = Pubkey::new_unique();

        let buy_nos_1 = Order::new_test(182, 123);
        let buy_nos_2 = Order::new_test(255, 184);
        let sell_nos_1 = Order::new_test(23, 33);
        let sell_nos_2 = Order::new_test(189, 31);

        book.new_limit(buy_nos_1, GTC, true, &mut rb, &mut rb_crank);
        book.new_limit(sell_nos_1, GTC, false, &mut rb, &mut rb_crank);
        book.new_limit(buy_nos_2, GTC, true, &mut rb, &mut rb_crank);
        book.new_limit(sell_nos_2, GTC, false, &mut rb, &mut rb_crank);

        assert_eq!(book.bids.tail, 0);
        assert_eq!(book.bids.head, 1);
        assert_eq!(book.bids.orders[0].order.limit, 182);
        assert_eq!(book.bids.orders[0].order.get_leaves_qty(), 90);
        assert_eq!(book.bids.orders[1].order.limit, 255);
        assert_eq!(book.bids.orders[1].order.get_leaves_qty(), 153);
    }

    #[test]
    fn it_should_place_a_few_orders_2() {
        let mut book = Book::new();
        let mut rb = RingBufferFilledExecReport::new();
        let mut rb_crank = RingBufferCrank::new();
        let mut buy_orders = [(12, 216), (179, 98)].to_vec();
        let mut sell_orders = [(22, 100), (51, 147)].to_vec();

        loop {
            let (price, size) = buy_orders.remove(0);
            println!("price {}", price);
            let buy_nos = Order::new_test(price, size);
            book.new_limit(buy_nos, GTC, true, &mut rb, &mut rb_crank);

            let (price, size) = sell_orders.remove(0);
            let sell_nos = Order::new_test(price, size);
            book.new_limit(sell_nos, GTC, false, &mut rb, &mut rb_crank);

            if buy_orders.len() == 0 && sell_orders.len() == 0 {
                break;
            }
        }

        assert_eq!(book.bids.tail, 0);
        assert_eq!(book.bids.head, 0);
        assert_eq!(book.bids.orders[0].order.limit, 12);
        assert_eq!(book.bids.orders[0].order.get_leaves_qty(), 216);

        assert_eq!(book.asks.tail, 1);
        assert_eq!(book.asks.head, 0);
        assert_eq!(book.asks.orders[0].order.limit, 22);
        assert_eq!(book.asks.orders[0].order.get_leaves_qty(), 2);
        assert_eq!(book.asks.orders[1].order.limit, 51);
        assert_eq!(book.asks.orders[1].order.get_leaves_qty(), 147);
    }

    #[test]
    fn it_should_place_a_few_orders_3() {
        let mut book = Book::new();
        let mut rb = RingBufferFilledExecReport::new();
        let mut rb_crank = RingBufferCrank::new();
        let mut buy_orders = [(255, 95), (197, 236)].to_vec();
        let mut sell_orders = [(199, 196), (91, 3)].to_vec();

        loop {
            let (price, size) = buy_orders.remove(0);
            println!("price {}", price);
            let buy_nos = Order::new_test(price, size);
            book.new_limit(buy_nos, GTC, true, &mut rb, &mut rb_crank);

            let (price, size) = sell_orders.remove(0);
            let sell_nos = Order::new_test(price, size);
            book.new_limit(sell_nos, GTC, false, &mut rb, &mut rb_crank);

            if buy_orders.len() == 0 && sell_orders.len() == 0 {
                break;
            }
        }

        assert_eq!(book.bids.tail, 0);
        assert_eq!(book.bids.head, 0);
        assert_eq!(book.bids.orders[0].order.limit, 197);
        assert_eq!(book.bids.orders[0].order.get_leaves_qty(), 233);

        assert_eq!(book.asks.tail, 0);
        assert_eq!(book.asks.head, 0);
        assert_eq!(book.asks.orders[0].order.limit, 199);
        assert_eq!(book.asks.orders[0].order.get_leaves_qty(), 101);
    }

    #[test]
    fn it_should_place_a_few_orders_4() {
        let mut book = Book::new();
        let mut rb = RingBufferFilledExecReport::new();
        let mut rb_crank = RingBufferCrank::new();
        let mut buy_orders = [(226, 135), (183, 46)].to_vec();
        let mut sell_orders = [(38, 157), (1, 148)].to_vec();

        loop {
            let (price, size) = buy_orders.remove(0);
            println!("price {}", price);
            let buy_nos = Order::new_test(price, size);
            book.new_limit(buy_nos, GTC, true, &mut rb, &mut rb_crank);

            let (price, size) = sell_orders.remove(0);
            let sell_nos = Order::new_test(price, size);
            book.new_limit(sell_nos, GTC, false, &mut rb, &mut rb_crank);

            if buy_orders.len() == 0 && sell_orders.len() == 0 {
                break;
            }
        }

        assert_eq!(book.bids.tail, 0);
        assert_eq!(book.bids.head, 0);
        assert_eq!(book.bids.orders[0].order.limit, 0);
        assert_eq!(book.bids.orders[0].order.get_leaves_qty(), 0);

        assert_eq!(book.asks.tail, 0);
        assert_eq!(book.asks.head, 0);
        assert_eq!(book.asks.orders[0].order.limit, 1);
        assert_eq!(book.asks.orders[0].order.get_leaves_qty(), 124);

        println!(
            "{:?},{:?},{:?}",
            book.bids.orders[0], book.bids.orders[1], book.bids.orders[2]
        );
    }

    #[quickcheck]
    fn it_should_match_many_orders(
        mut buy: Vec<(u16, u32)>,
        mut sell: Vec<(u16, u32)>,
    ) -> TestResult {
        buy = buy
            .into_iter()
            .filter(|(x, y)| *x != 0 && *y != 0)
            .collect();
        sell = sell
            .into_iter()
            .filter(|(x, y)| *x != 0 && *y != 0)
            .collect();

        let mut book = Book::new();
        let mut rb = RingBufferFilledExecReport::new();
        let mut rb_crank = RingBufferCrank::new();
        let mut clone_buy = buy.clone();
        let mut clone_sell = sell.clone();

        let mut sort_buy: Vec<(u16, u32, u8)> = Vec::new();
        let mut sort_sell: Vec<(u16, u32, u8)> = Vec::new();
        let mut sort_buy_rank = 0;
        let mut sort_sell_rank = 0;
        loop {
            if clone_buy.len() > 0 {
                let (bid_price, mut bid_size) = clone_buy.remove(0);
                for (ask_price, ask_size, _) in &mut sort_sell.iter_mut() {
                    if bid_price >= *ask_price {
                        if *ask_size >= bid_size {
                            *ask_size -= bid_size;
                            bid_size = 0;
                        } else {
                            bid_size -= *ask_size;
                            *ask_size = 0;
                        }
                    } else {
                        break;
                    }
                    if bid_size == 0 {
                        break;
                    }
                }
                sort_buy.push((bid_price, bid_size, sort_buy_rank));
                sort_buy_rank += 1;
                // Order sort_buy descending
                sort_buy.sort_by(
                    |(ax, _, cx), (ay, _, cy)| if ax == ay { cx.cmp(cy) } else { ay.cmp(ax) },
                );
            }
            if clone_sell.len() > 0 {
                let (ask_price, mut ask_size) = clone_sell.remove(0);
                for (bid_price, bid_size, _) in &mut sort_buy.iter_mut() {
                    if ask_price <= *bid_price {
                        if ask_size >= *bid_size {
                            ask_size -= *bid_size;
                            *bid_size = 0;
                        } else {
                            *bid_size -= ask_size;
                            ask_size = 0;
                        }
                    } else {
                        break;
                    }
                    if ask_size == 0 {
                        break;
                    }
                }
                sort_sell.push((ask_price, ask_size, sort_sell_rank));
                sort_sell_rank += 1;
                // sort ascending
                sort_sell.sort_by(
                    |(ax, _, cx), (ay, _, cy)| if ax == ay { cx.cmp(cy) } else { ax.cmp(ay) },
                );
            }

            if clone_buy.len() == 0 && clone_sell.len() == 0 {
                break;
            }
        }

        loop {
            if buy.len() > 0 {
                let (bid_price, bid_size) = buy.remove(0);
                let buy_nos = Order::new_test(bid_price as u64, bid_size as u64);
                book.new_limit(buy_nos, GTC, true, &mut rb, &mut rb_crank);
            }
            if sell.len() > 0 {
                let (ask_price, ask_size) = sell.remove(0);
                let sell_nos = Order::new_test(ask_price as u64, ask_size as u64);
                book.new_limit(sell_nos, GTC, false, &mut rb, &mut rb_crank);
            }

            if buy.len() == 0 && sell.len() == 0 {
                break;
            }
        }

        sort_buy = sort_buy
            .into_iter()
            .filter(|(_, size, _)| *size != 0)
            .collect();

        sort_sell = sort_sell
            .into_iter()
            .filter(|(_, size, _)| *size != 0)
            .collect();

        let mut cur_bid_pos = book.bids.head;
        for (limit, size, _) in sort_buy.clone() {
            let order = book.bids.orders[cur_bid_pos as usize];
            assert_eq!(order.order.limit, limit as u64);
            assert_eq!(order.order.get_leaves_qty(), size as u64);
            cur_bid_pos = order.next;
        }

        let mut cur_ask_pos = book.asks.head;
        for (limit, size, _) in sort_sell {
            let order = book.asks.orders[cur_ask_pos as usize];
            assert_eq!(order.order.limit, limit as u64);
            assert_eq!(order.order.get_leaves_qty(), size as u64);
            cur_ask_pos = order.next;
        }
        TestResult::passed()
    }
}
