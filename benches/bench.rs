use std::collections::HashMap;

use common::traits::{Get, Insert};
use const_lru::ConstLru;
use criterion::{criterion_group, criterion_main, Criterion};

mod common;

fn fill_up_all_u8_keys<C: Insert<K, V>, K: From<u8>, V: From<u8>>(container: &mut C) {
    for k in 0..u8::MAX {
        container.insert_no_ret(k.into(), k.into());
    }
}

fn get_all_u8_keys<C: Get<K, V>, K: From<u8>, V>(container: &mut C) {
    for k in 0..u8::MAX {
        container.get_by_key(&k.into());
    }
}

fn bench_lru_to_mru<C: Insert<K, V> + Get<K, V>, K: From<u8>, V: From<u8>>(
    c: &mut Criterion,
    bench_name: &str,
    mut container: C,
) {
    fill_up_all_u8_keys(&mut container);

    c.bench_function(bench_name, |bencher| {
        bencher.iter(|| get_all_u8_keys(&mut container))
    });
}

// 90 us
fn u8_get_lru_to_mru_const_lru(c: &mut Criterion) {
    let container: ConstLru<u8, u64, 255, u8> = ConstLru::new();
    bench_lru_to_mru(c, "u8 lru to mru ConstLru", container);
}

// 2.5 us
fn u8_get_lru_to_mru_hashmap(c: &mut Criterion) {
    let container: HashMap<u8, u64> = HashMap::new();
    bench_lru_to_mru(c, "u8 lru to mru HashMap", container);
}

// around same as u8
fn usize_get_lru_to_mru_const_lru(c: &mut Criterion) {
    let container: ConstLru<usize, u64, 255, u8> = ConstLru::new();
    bench_lru_to_mru(c, "usize lru to mru ConstLru", container);
}

// around same as u8
fn usize_get_lru_to_mru_hashmap(c: &mut Criterion) {
    let container: HashMap<usize, u64> = HashMap::new();
    bench_lru_to_mru(c, "usize lru to mru HashMap", container);
}

fn bench_get_mru<C: Insert<K, V> + Get<K, V>, K: From<u8>, V: From<u8>>(
    c: &mut Criterion,
    bench_name: &str,
    mut container: C,
) {
    fill_up_all_u8_keys(&mut container);

    c.bench_function(bench_name, |bencher| {
        bencher.iter(|| {
            container.get_by_key(&(u8::MAX - 1).into());
        })
    });
}

// 1 ns
fn u8_get_mru_const_lru(c: &mut Criterion) {
    let container: ConstLru<u8, u64, 255, u8> = ConstLru::new();
    bench_get_mru(c, "u8 mru ConstLru", container);
}

// 11 ns
fn u8_get_mru_hashmap(c: &mut Criterion) {
    let container: HashMap<u8, u64> = HashMap::new();
    bench_get_mru(c, "u8 mru HashMap", container);
}

// around same as u8
fn usize_get_mru_const_lru(c: &mut Criterion) {
    let container: ConstLru<usize, u64, 255, u8> = ConstLru::new();
    bench_get_mru(c, "usize mru ConstLru", container);
}

// around same as u8
fn usize_get_mru_hashmap(c: &mut Criterion) {
    let container: HashMap<usize, u64> = HashMap::new();
    bench_get_mru(c, "usize mru HashMap", container);
}

criterion_group!(
    u8_get_lru_to_mru,
    u8_get_lru_to_mru_const_lru,
    u8_get_lru_to_mru_hashmap
);
criterion_group!(
    usize_get_lru_to_mru,
    usize_get_lru_to_mru_const_lru,
    usize_get_lru_to_mru_hashmap
);

criterion_group!(u8_get_mru, u8_get_mru_const_lru, u8_get_mru_hashmap);
criterion_group!(
    usize_get_mru,
    usize_get_mru_const_lru,
    usize_get_mru_hashmap
);

criterion_main!(
    u8_get_lru_to_mru,
    usize_get_lru_to_mru,
    u8_get_mru,
    usize_get_mru
);
