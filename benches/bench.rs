use criterion::{criterion_group, criterion_main};
use get_mru::{
    bigstruct_get_mru_const_lru, bigstruct_get_mru_const_lru_i_usize, bigstruct_get_mru_hashmap,
    ten_k_bigstruct_get_mru_const_lru, ten_k_bigstruct_get_mru_hashmap, ten_k_get_mru_const_lru,
    ten_k_get_mru_hashmap, u8_get_mru_const_lru, u8_get_mru_const_lru_i_usize, u8_get_mru_hashmap,
};
use insert::{
    bigstruct_insert_const_lru, bigstruct_insert_const_lru_i_usize, bigstruct_insert_hashmap,
    ten_k_bigstruct_insert_const_lru, ten_k_bigstruct_insert_hashmap, ten_k_insert_const_lru,
    ten_k_insert_hashmap, u8_insert_const_lru, u8_insert_const_lru_i_usize, u8_insert_hashmap,
};
use lru_to_mru::{
    bigstruct_get_lru_to_mru_const_lru, bigstruct_get_lru_to_mru_const_lru_i_usize,
    bigstruct_get_lru_to_mru_hashmap, ten_k_bigstruct_get_lru_to_mru_const_lru,
    ten_k_bigstruct_get_lru_to_mru_hashmap, ten_k_get_lru_to_mru_const_lru,
    ten_k_get_lru_to_mru_hashmap, u8_get_lru_to_mru_const_lru, u8_get_lru_to_mru_const_lru_i_usize,
    u8_get_lru_to_mru_hashmap,
};
use remove::{
    bigstruct_remove_const_lru, bigstruct_remove_const_lru_i_usize, bigstruct_remove_hashmap,
    ten_k_bigstruct_remove_const_lru, ten_k_bigstruct_remove_hashmap, ten_k_remove_const_lru,
    ten_k_remove_hashmap, u8_remove_const_lru, u8_remove_const_lru_i_usize, u8_remove_hashmap,
};

mod common;
mod get_mru;
mod insert;
mod lru_to_mru;
mod remove;

criterion_group!(
    u8_get_lru_to_mru,
    u8_get_lru_to_mru_const_lru,
    u8_get_lru_to_mru_const_lru_i_usize,
    u8_get_lru_to_mru_hashmap
);
criterion_group!(
    bigstruct_get_lru_to_mru,
    bigstruct_get_lru_to_mru_const_lru,
    bigstruct_get_lru_to_mru_const_lru_i_usize,
    bigstruct_get_lru_to_mru_hashmap
);
criterion_group!(
    ten_k_lru_to_mru,
    ten_k_get_lru_to_mru_const_lru,
    ten_k_get_lru_to_mru_hashmap
);
criterion_group!(
    ten_k_bigstruct_lru_to_mru,
    ten_k_bigstruct_get_lru_to_mru_const_lru,
    ten_k_bigstruct_get_lru_to_mru_hashmap
);

criterion_group!(
    u8_get_mru,
    u8_get_mru_const_lru,
    u8_get_mru_const_lru_i_usize,
    u8_get_mru_hashmap,
);
criterion_group!(
    bigstruct_get_mru,
    bigstruct_get_mru_const_lru,
    bigstruct_get_mru_const_lru_i_usize,
    bigstruct_get_mru_hashmap
);
criterion_group!(
    ten_k_get_mru,
    ten_k_get_mru_const_lru,
    ten_k_get_mru_hashmap
);
criterion_group!(
    ten_k_bigstruct_get_mru,
    ten_k_bigstruct_get_mru_const_lru,
    ten_k_bigstruct_get_mru_hashmap
);

criterion_group!(
    u8_insert,
    u8_insert_const_lru,
    u8_insert_const_lru_i_usize,
    u8_insert_hashmap
);
criterion_group!(
    bigstruct_insert,
    bigstruct_insert_const_lru,
    bigstruct_insert_const_lru_i_usize,
    bigstruct_insert_hashmap
);
criterion_group!(ten_k_insert, ten_k_insert_const_lru, ten_k_insert_hashmap);
criterion_group!(
    ten_k_bigstruct_insert,
    ten_k_bigstruct_insert_const_lru,
    ten_k_bigstruct_insert_hashmap
);

criterion_group!(
    u8_remove,
    u8_remove_const_lru,
    u8_remove_const_lru_i_usize,
    u8_remove_hashmap
);
criterion_group!(
    bigstruct_remove,
    bigstruct_remove_const_lru,
    bigstruct_remove_const_lru_i_usize,
    bigstruct_remove_hashmap
);
criterion_group!(ten_k_remove, ten_k_remove_const_lru, ten_k_remove_hashmap);
criterion_group!(
    ten_k_bigstruct_remove,
    ten_k_bigstruct_remove_const_lru,
    ten_k_bigstruct_remove_hashmap
);

criterion_main!(
    u8_get_lru_to_mru,
    bigstruct_get_lru_to_mru,
    ten_k_lru_to_mru,
    ten_k_bigstruct_lru_to_mru,
    u8_get_mru,
    bigstruct_get_mru,
    ten_k_get_mru,
    ten_k_bigstruct_get_mru,
    u8_insert,
    bigstruct_insert,
    ten_k_insert,
    ten_k_bigstruct_insert,
    u8_remove,
    bigstruct_remove,
    ten_k_remove,
    ten_k_bigstruct_remove
);
