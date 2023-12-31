use const_lru::ConstLru;

#[test]
#[should_panic]
fn u128_i_panic() {
    let _c: ConstLru<u8, u8, 1, u128> = ConstLru::new();
}

#[test]
#[should_panic]
fn cap_oob_panic() {
    let _c: ConstLru<u8, u8, 256, u8> = ConstLru::new();
}
