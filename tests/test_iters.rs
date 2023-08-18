use const_lru::ConstLru;

#[test]
fn test_empty_iter() {
    let c: ConstLru<u8, u64, 1, u8> = ConstLru::new();
    assert_eq!(c.iter().next(), None);
}

#[test]
fn test_empty_iter_mut() {
    let mut c: ConstLru<u8, u64, 1, u8> = ConstLru::new();
    assert_eq!(c.iter_mut().next(), None);
}
