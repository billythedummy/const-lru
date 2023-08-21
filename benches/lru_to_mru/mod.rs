use std::collections::HashMap;

use const_lru::ConstLru;
use criterion::Criterion;

use crate::common::{
    traits::{Get, Insert},
    utils::{fill_up_all_u8_keys, BigStruct},
};

// if keys were inserted from 0..255,
// this results in container geting the LRU at every iteration
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

// 2.5 us
pub fn u8_get_lru_to_mru_const_lru(c: &mut Criterion) {
    let container: ConstLru<u8, u64, 255, u8> = ConstLru::new();
    bench_lru_to_mru(c, "u8 lru to mru ConstLru", container);
}

// around same as I=u8
pub fn u8_get_lru_to_mru_const_lru_i_usize(c: &mut Criterion) {
    let container: ConstLru<u8, u64, 255, usize> = ConstLru::new();
    bench_lru_to_mru(c, "u8 lru to mru ConstLru I=usize", container);
}

// 2.5 us
pub fn u8_get_lru_to_mru_hashmap(c: &mut Criterion) {
    let container: HashMap<u8, u64> = HashMap::new();
    bench_lru_to_mru(c, "u8 lru to mru HashMap", container);
}

// usize keys perform similarly to u8, so removed the benchmark

// 71 us
pub fn bigstruct_get_lru_to_mru_const_lru(c: &mut Criterion) {
    let container: ConstLru<BigStruct, BigStruct, 255, u8> = ConstLru::new();
    bench_lru_to_mru(c, "bigstruct lru to mru ConstLru", container);
}

// around same as I=u8
pub fn bigstruct_get_lru_to_mru_const_lru_i_usize(c: &mut Criterion) {
    let container: ConstLru<BigStruct, BigStruct, 255, usize> = ConstLru::new();
    bench_lru_to_mru(c, "bigstruct lru to mru ConstLru I=usize", container);
}

// 80 us
pub fn bigstruct_get_lru_to_mru_hashmap(c: &mut Criterion) {
    let container: HashMap<BigStruct, BigStruct> = HashMap::new();
    bench_lru_to_mru(c, "bigstruct lru to mru HashMap", container);
}
