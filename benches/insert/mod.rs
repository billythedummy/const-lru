use std::collections::HashMap;

use const_lru::ConstLru;
use criterion::Criterion;

use crate::common::{
    traits::{CreateNew, Insert},
    utils::BigStruct,
};

// insert in reverse key order: worst-case for ConstLru
fn bench_insert<C: Insert<K, V> + CreateNew, K: From<u8>, V: From<u8>>(
    c: &mut Criterion,
    bench_name: &str,
) {
    c.bench_function(bench_name, move |bencher| {
        bencher.iter_batched(
            || C::create_new(),
            |mut container| {
                for k in u8::MAX..0 {
                    container.insert_no_ret(k.into(), k.into());
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

// insert in reverse key order: worst-case for ConstLru
fn bench_ten_k_insert<C: Insert<K, V> + CreateNew, K: From<u16>, V: From<u16>>(
    c: &mut Criterion,
    bench_name: &str,
) {
    c.bench_function(bench_name, move |bencher| {
        bencher.iter_batched(
            || C::create_new(),
            |mut container| {
                for k in 9_999..0 {
                    container.insert_no_ret(k.into(), k.into());
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

// 126 ns
pub fn u8_insert_const_lru(c: &mut Criterion) {
    bench_insert::<ConstLru<u8, u64, 255, u8>, _, _>(c, "u8 insert ConstLru");
}

// 337 ns
pub fn u8_insert_const_lru_i_usize(c: &mut Criterion) {
    bench_insert::<ConstLru<u8, u64, 255, usize>, _, _>(c, "u8 insert ConstLru I=usize");
}

// 2 ns
pub fn u8_insert_hashmap(c: &mut Criterion) {
    bench_insert::<HashMap<u8, u64>, _, _>(c, "u8 insert HashMap");
}

// Must Box<ConstLru> else compilation fails with
// SIGSEGV: invalid memory reference
// Probably out of stack

// TODO: figure out why BigStruct insert is 2x faster than u8

// 57 ns
pub fn bigstruct_insert_const_lru(c: &mut Criterion) {
    bench_insert::<Box<ConstLru<BigStruct, BigStruct, 255, u8>>, _, _>(
        c,
        "bigstruct insert ConstLru",
    );
}

// 62 ns
pub fn bigstruct_insert_const_lru_i_usize(c: &mut Criterion) {
    bench_insert::<Box<ConstLru<BigStruct, BigStruct, 255, usize>>, _, _>(
        c,
        "bigstruct insert ConstLru I=usize",
    );
}

// 2 ns
pub fn bigstruct_insert_hashmap(c: &mut Criterion) {
    bench_insert::<HashMap<BigStruct, BigStruct>, _, _>(c, "bigstruct insert HashMap");
}

// 2.9 us
pub fn ten_k_insert_const_lru(c: &mut Criterion) {
    bench_ten_k_insert::<Box<ConstLru<u16, u64, 10_000, u16>>, _, _>(c, "10k insert ConstLru");
}

// 2.1 ns
pub fn ten_k_insert_hashmap(c: &mut Criterion) {
    bench_ten_k_insert::<HashMap<u16, u64>, _, _>(c, "10k insert HashMap");
}

// 3.3 us
pub fn ten_k_bigstruct_insert_const_lru(c: &mut Criterion) {
    bench_ten_k_insert::<Box<ConstLru<BigStruct, BigStruct, 10_000, u16>>, _, _>(
        c,
        "10k bigstruct insert ConstLru",
    );
}

// 2.2 ns
pub fn ten_k_bigstruct_insert_hashmap(c: &mut Criterion) {
    bench_ten_k_insert::<HashMap<BigStruct, BigStruct>, _, _>(c, "10k bigstruct insert HashMap");
}
