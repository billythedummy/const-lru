use std::collections::HashMap;

use const_lru::ConstLru;
use criterion::Criterion;

use crate::common::{
    traits::{Get, Insert},
    utils::{fill_up_all_u8_keys, BigStruct},
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

// 1 ns
pub fn u8_get_mru_const_lru(c: &mut Criterion) {
    let container: ConstLru<u8, u64, 255, u8> = ConstLru::new();
    bench_get_mru(c, "u8 mru ConstLru", container);
}

// 2.5 ns
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

// 40ns
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
