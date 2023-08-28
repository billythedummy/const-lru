use std::collections::HashMap;

use const_lru::ConstLru;
use criterion::Criterion;

use crate::common::{
    traits::{Get, Insert},
    utils::{boxed_const_lru, fill_up_all_10k_keys, fill_up_all_u8_keys, BigStruct},
};

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

fn bench_get_mru_10k<C: Insert<K, V> + Get<K, V>, K: From<u16>, V: From<u16>>(
    c: &mut Criterion,
    bench_name: &str,
    mut container: C,
) {
    fill_up_all_10k_keys(&mut container);

    c.bench_function(bench_name, |bencher| {
        bencher.iter(|| {
            container.get_by_key(&10_000.into());
        })
    });
}

// 12 ns
pub fn u8_get_mru_const_lru(c: &mut Criterion) {
    let container: ConstLru<u8, u64, 255, u8> = ConstLru::new();
    bench_get_mru(c, "u8 mru ConstLru", container);
}

// 13 ns
pub fn u8_get_mru_const_lru_i_usize(c: &mut Criterion) {
    let container: ConstLru<u8, u64, 255, usize> = ConstLru::new();
    bench_get_mru(c, "u8 mru ConstLru I=usize", container);
}

// 11 ns
pub fn u8_get_mru_hashmap(c: &mut Criterion) {
    let container: HashMap<u8, u64> = HashMap::new();
    bench_get_mru(c, "u8 mru HashMap", container);
}

// usize keys perform similarly to u8, so removed the benchmark

// 330ns
pub fn bigstruct_get_mru_const_lru(c: &mut Criterion) {
    let container: ConstLru<BigStruct, BigStruct, 255, u8> = ConstLru::new();
    bench_get_mru(c, "bigstruct mru ConstLru", container);
}

// around same as I=u8
pub fn bigstruct_get_mru_const_lru_i_usize(c: &mut Criterion) {
    let container: ConstLru<BigStruct, BigStruct, 255, usize> = ConstLru::new();
    bench_get_mru(c, "bigstruct mru ConstLru I=usize", container);
}

// 300ns
pub fn bigstruct_get_mru_hashmap(c: &mut Criterion) {
    let container: HashMap<BigStruct, BigStruct> = HashMap::new();
    bench_get_mru(c, "bigstruct mru HashMap", container);
}

// 40 ns
pub fn ten_k_get_mru_const_lru(c: &mut Criterion) {
    let container: ConstLru<u16, u64, 10_000, u16> = ConstLru::new();
    bench_get_mru_10k(c, "10k mru ConstLru", container);
}

// 11 ns
pub fn ten_k_get_mru_hashmap(c: &mut Criterion) {
    let container: HashMap<u16, u64> = HashMap::new();
    bench_get_mru_10k(c, "10k mru hashmap", container);
}

// 886 ns
pub fn ten_k_bigstruct_get_mru_const_lru(c: &mut Criterion) {
    let container: Box<ConstLru<BigStruct, BigStruct, 10_000, u16>> = boxed_const_lru();
    bench_get_mru_10k(c, "10k bigstruct mru ConstLru", container);
}

// 300 ns
pub fn ten_k_bigstruct_get_mru_hashmap(c: &mut Criterion) {
    let container: HashMap<BigStruct, BigStruct> = HashMap::new();
    bench_get_mru_10k(c, "10k bigstruct mru hashmap", container);
}
