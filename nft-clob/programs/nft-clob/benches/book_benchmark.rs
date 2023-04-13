use std::collections::LinkedList;

use anchor_lang::prelude::Pubkey;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nft_clob::account_states::{Book, Order, RBTree};
use nft_clob::sort_arr;

fn sort_arr_benchmark(c: &mut Criterion) {
    // let mut arr = black_box([6, 2, 4, 1, 9, -2, 5]);
    let mut book = Book {
        bids: RBTree::new(),
        asks: RBTree::new(),
    };
    let price: u64 = 10;
    for i in 1..2000 {
        let maker = Pubkey::new_unique();
        let order = Order::new(price, i, maker);
        book.asks.insert(i, order);
    }

    let mut linked_list = LinkedList::new();
    for i in 1..2000 {
        linked_list.push_back(i);
    }
    // Pop first
    c.bench_function("pop first rb tree", |b| b.iter(|| book.asks.pop_first()));
    c.bench_function("pop first linked list", |b| {
        b.iter(|| linked_list.pop_front())
    });
    // Search
    let search: u64 = 500;
    c.bench_function("search rb tree", |b| b.iter(|| book.asks.get(&search)));
    c.bench_function("search linked list", |b| {
        b.iter(|| linked_list.iter().find(|&&x| x == search))
    });
}

criterion_group!(benches, sort_arr_benchmark);
criterion_main!(benches);
