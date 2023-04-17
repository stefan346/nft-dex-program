use std::collections::LinkedList;
use std::mem::size_of;

use anchor_lang::prelude::Pubkey;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nft_clob::account_states::{Book, NewOrderSingle, Order, MAX_ORDERS};
use nft_clob::sort_arr;
use rand::Rng;
use slice_rbtree::tree::{tree_size, RBTree, TreeParams};

fn sort_arr_benchmark(c: &mut Criterion) {
    c.bench_function("insert 2000 orders", |b| {
        b.iter(|| {
            let mut book = Book::new();
            let maker = Pubkey::new_unique();
            let mut rng = rand::thread_rng();
            for i in 1..512 {
                let nos = NewOrderSingle::new(true, rng.gen_range(1..150), 1);
                book.new_limit(&nos, maker);
            }
        })
    });

    let mut book = Book::new();
    let maker = Pubkey::new_unique();
    for i in 1..2000 {
        let nos = NewOrderSingle::new(true, 6000 - i, 1);
        book.new_limit(&nos, maker);
    }

    c.bench_function("remove order", |b| {
        b.iter(|| {
            book.bids.remove_order(872);
        })
    });

    c.bench_function("rbtree: insert 2000 orders", |b| {
        b.iter(|| {
            let size = tree_size(
                TreeParams {
                    k_size: 8,
                    v_size: size_of::<NewOrderSingle>(),
                },
                MAX_ORDERS as usize,
            );

            let mut buffer = vec![0; size];
            const mem_size: usize = size_of::<NewOrderSingle>();
            let mut rbtree: RBTree<u64, NewOrderSingle, 8, mem_size> =
                RBTree::init_slice(&mut buffer).unwrap();
            let maker = Pubkey::new_unique();

            let mut rng = rand::thread_rng();
            for i in 1..512 {
                let nos = NewOrderSingle::new(true, rng.gen_range(1..150), 1);
                rbtree.insert(i, nos).unwrap();
            }
        })
    });


    let size = tree_size(
        TreeParams {
            k_size: 8,
            v_size: size_of::<NewOrderSingle>(),
        },
        MAX_ORDERS as usize,
    );

    let mut buffer = vec![0; size];
    const MEM_SIZE: usize = size_of::<NewOrderSingle>();
    let mut rbtree: RBTree<u64, NewOrderSingle, 8, MEM_SIZE> =
        RBTree::init_slice(&mut buffer).unwrap();
    let maker = Pubkey::new_unique();

    let mut rng = rand::thread_rng();
    for i in 1..512 {
        let nos = NewOrderSingle::new(true, rng.gen_range(1..150), 1);
        rbtree.insert(i, nos).unwrap();
    }

    c.bench_function("remove order", |b| {
        b.iter(|| {
            let val = 247;
            rbtree.remove_entry(&val);
        })
    });

    // // let mut arr = black_box([6, 2, 4, 1, 9, -2, 5]);
    // let mut book = Book {
    //     bids: RBTree::new(),
    //     asks: RBTree::new(),
    // };
    // let price: u64 = 10;
    // for i in 1..2000 {
    //     let maker = Pubkey::new_unique();
    //     let order = Order::new(price, i, maker);
    //     book.asks.insert(i, order);
    // }

    // let mut linked_list = LinkedList::new();
    // for i in 1..2000 {
    //     linked_list.push_back(i);
    // }
    // // Pop first
    // c.bench_function("pop first rb tree", |b| b.iter(|| book.asks.pop_first()));
    // c.bench_function("pop first linked list", |b| {
    //     b.iter(|| linked_list.pop_front())
    // });
    // // Search
    // let search: u64 = 500;
    // c.bench_function("search rb tree", |b| b.iter(|| book.asks.get(&search)));
    // c.bench_function("search linked list", |b| {
    //     b.iter(|| linked_list.iter().find(|&&x| x == search))
    // });
}

criterion_group!(benches, sort_arr_benchmark);
criterion_main!(benches);
