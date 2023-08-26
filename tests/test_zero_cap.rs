use const_lru::ConstLru;

#[test]
fn zero_cap_insert_get_remove() {
    const ENTRY: (u8, u64) = (1, 2);

    let mut c: ConstLru<u8, u64, 0, u8> = ConstLru::new();
    assert!(c.is_empty());
    assert!(c.len() == 0);
    assert!(c.is_full());

    assert!(c.insert(ENTRY.0, ENTRY.1).is_none());
    assert!(c.get(&ENTRY.0).is_none());
    assert!(c.remove(&ENTRY.0).is_none());
}

#[test]
fn zero_cap_iters() {
    let mut c: ConstLru<u8, u64, 0, u8> = ConstLru::new();
    assert!(c.iter().next().is_none());
    assert!(c.iter().next_back().is_none());
    assert!(c.iter_mut().next().is_none());
    assert!(c.iter_mut().next_back().is_none());
    assert!(c.iter_key_order().next().is_none());
    assert!(c.iter_key_order().next_back().is_none());
    assert!(c.iter_key_order_mut().next().is_none());
    assert!(c.iter_key_order_mut().next_back().is_none());
    let c2 = c.clone();
    assert!(c.into_iter().next().is_none());
    assert!(c2.into_iter().next_back().is_none());
}
