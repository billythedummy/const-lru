use std::collections::HashMap;

use const_lru::ConstLru;
use criterion::Criterion;

use crate::common::{
    traits::{CreateNew, Insert, Remove},
    utils::{fill_up_all_u8_keys, BigStruct},
};

// remove in key order: worst-case for ConstLru
fn bench_remove<C: Remove<K, V> + CreateNew + Insert<K, V>, K: From<u8>, V: From<u8>>(
    c: &mut Criterion,
    bench_name: &str,
) {
    c.bench_function(bench_name, move |bencher| {
        bencher.iter_batched(
            || {
                let mut c = C::create_new();
                fill_up_all_u8_keys(&mut c);
                c
            },
            |mut container| {
                for k in 0..u8::MAX {
                    container.remove_by_key(&k.into());
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

// 5 us
pub fn u8_remove_const_lru(c: &mut Criterion) {
    bench_remove::<ConstLru<u8, u64, 255, u8>, _, _>(c, "u8 remove ConstLru");
}

// 6.5 us
pub fn u8_remove_const_lru_i_usize(c: &mut Criterion) {
    bench_remove::<ConstLru<u8, u64, 255, usize>, _, _>(c, "u8 remove ConstLru I=usize");
}

// 4.7 us
pub fn u8_remove_hashmap(c: &mut Criterion) {
    bench_remove::<HashMap<u8, u64>, _, _>(c, "u8 remove HashMap");
}

// Must Box<ConstLru> else compilation fails with
// SIGSEGV: invalid memory reference
// Probably out of stack

// 102 us
pub fn bigstruct_remove_const_lru(c: &mut Criterion) {
    bench_remove::<Box<ConstLru<BigStruct, BigStruct, 255, u8>>, _, _>(
        c,
        "bigstruct remove ConstLru",
    );
}

// 104 us
pub fn bigstruct_remove_const_lru_i_usize(c: &mut Criterion) {
    bench_remove::<Box<ConstLru<BigStruct, BigStruct, 255, usize>>, _, _>(
        c,
        "bigstruct remove ConstLru I=usize",
    );
}

// 122 us
pub fn bigstruct_remove_hashmap(c: &mut Criterion) {
    bench_remove::<HashMap<BigStruct, BigStruct>, _, _>(c, "bigstruct remove HashMap");
}
