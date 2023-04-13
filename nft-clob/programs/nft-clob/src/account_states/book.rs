use anchor_lang::prelude::*;

use super::{Order, RBTree};

/// Central Limit Order Book
#[account(zero_copy)]
pub struct Book {
    pub bids: RBTree<u64, Order>, // Buy orders.
    pub asks: RBTree<u64, Order>, // Sell orders.
}

impl Book {
    pub fn space(bids_size: usize) -> usize {
        8 + (4 + Order::space() * bids_size)
    }
}

#[cfg(test)]
mod test {
    use std::mem::{self, size_of};

    use anchor_lang::prelude::Pubkey;

    use crate::account_states::{NodePtr, Order, RBTree, RBTreeNode};

    use super::Book;

    #[test]
    fn it_should_add_orders() {
        let mut book = Book {
            bids: RBTree::new(),
            asks: RBTree::new(),
        };
        // let encoded: Vec<u8> = bincode::serialize(&book).unwrap();
        let buffer_size = mem::size_of::<RBTree<u64, Order>>();
        println!("rb-tree mem size: {:?}", buffer_size);

        let node_ptr_size = mem::size_of::<NodePtr<u64, Order>>();
        println!("node ptr node mem size: {:?}", node_ptr_size);

        let node_size = mem::size_of::<RBTreeNode<u64, Order>>();
        println!("rb-tree node mem size: {:?}", node_size);

        let order_size = mem::size_of::<Order>();
        println!("order mem size: {:?}", order_size);

        let price: u64 = 10;
        let mut first_maker;
        for i in 1..1000 {
            let maker = Pubkey::new_unique();
            if i == 1 {
                first_maker = maker.clone();
                println!("maker = {:?}", first_maker);
            }
            let order = Order::new(price, i, maker);
            book.asks.insert(price, order);
        }

        println!("Book first: {:?}", book.asks.get_first().unwrap().1.maker);
        book.asks.pop_first();
        println!("Book first: {:?}", book.asks.get_first().unwrap().1.maker);
    }
}
