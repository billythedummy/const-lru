use std::collections::HashMap;

use const_lru::ConstLru;
use criterion::Criterion;

use crate::common::{
    traits::{CreateNew, Insert},
    utils::BigStruct,
};

// insert in reverse key order: worst-case for ConstLru
// since new entries go to [0] and push everything right in bs-index
fn bench_insert<C: Insert<K, V> + CreateNew, K: From<u8>, V: From<u8>>(
    c: &mut Criterion,
    bench_name: &str,
) {
    c.bench_function(bench_name, move |bencher| {
        bencher.iter_batched(
            || C::create_new(),
            |mut container| {
                for k in (0..u8::MAX).rev() {
                    container.insert_no_ret(k.into(), k.into());
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

// insert in reverse key order: worst-case for ConstLru
// since new entries go to [0] and push everything right in bs-index
fn bench_ten_k_insert<C: Insert<K, V> + CreateNew, K: From<u16>, V: From<u16>>(
    c: &mut Criterion,
    bench_name: &str,
) {
    c.bench_function(bench_name, move |bencher| {
        bencher.iter_batched(
            || C::create_new(),
            |mut container| {
                for k in (0..9_999).rev() {
                    container.insert_no_ret(k.into(), k.into());
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

// 3.6 us
pub fn u8_insert_const_lru(c: &mut Criterion) {
    bench_insert::<ConstLru<u8, u64, 255, u8>, _, _>(c, "u8 insert ConstLru");
}

// 5.8 us
pub fn u8_insert_const_lru_i_usize(c: &mut Criterion) {
    bench_insert::<ConstLru<u8, u64, 255, usize>, _, _>(c, "u8 insert ConstLru I=usize");
}

// 11.6 us
pub fn u8_insert_hashmap(c: &mut Criterion) {
    bench_insert::<HashMap<u8, u64>, _, _>(c, "u8 insert HashMap");
}

// Must Box<ConstLru> else compilation fails with
// SIGSEGV: invalid memory reference
// Probably out of stack

// TODO: figure out why BigStruct insert is 2x faster than u8

// 100.7 us
pub fn bigstruct_insert_const_lru(c: &mut Criterion) {
    bench_insert::<Box<ConstLru<BigStruct, BigStruct, 255, u8>>, _, _>(
        c,
        "bigstruct insert ConstLru",
    );
}

// 97 us
pub fn bigstruct_insert_const_lru_i_usize(c: &mut Criterion) {
    bench_insert::<Box<ConstLru<BigStruct, BigStruct, 255, usize>>, _, _>(
        c,
        "bigstruct insert ConstLru I=usize",
    );
}

// 239 us
pub fn bigstruct_insert_hashmap(c: &mut Criterion) {
    bench_insert::<HashMap<BigStruct, BigStruct>, _, _>(c, "bigstruct insert HashMap");
}

// 1.2 ms
pub fn ten_k_insert_const_lru(c: &mut Criterion) {
    bench_ten_k_insert::<Box<ConstLru<u16, u64, 10_000, u16>>, _, _>(c, "10k insert ConstLru");
}

// 375 us
pub fn ten_k_insert_hashmap(c: &mut Criterion) {
    bench_ten_k_insert::<HashMap<u16, u64>, _, _>(c, "10k insert HashMap");
}

// 6.5 ms
pub fn ten_k_bigstruct_insert_const_lru(c: &mut Criterion) {
    bench_ten_k_insert::<Box<ConstLru<BigStruct, BigStruct, 10_000, u16>>, _, _>(
        c,
        "10k bigstruct insert ConstLru",
    );
}

// 18.8 ms
pub fn ten_k_bigstruct_insert_hashmap(c: &mut Criterion) {
    bench_ten_k_insert::<HashMap<BigStruct, BigStruct>, _, _>(c, "10k bigstruct insert HashMap");
}
