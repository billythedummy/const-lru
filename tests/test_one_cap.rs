use const_lru::{ConstLru, InsertReplaced};

#[test]
fn test_one_cap_simple() {
    const ENTRY: (u8, u64) = (1, 2);
    const NON_KEY: u8 = 3;

    let mut c: ConstLru<u8, u64, 1, u8> = ConstLru::new();
    assert!(c.is_empty());
    assert!(c.len() == 0);

    assert!(c.insert(ENTRY.0, ENTRY.1).is_none());

    assert!(c.is_full());
    assert!(c.len() == 1);
    assert_eq!(c.get(&ENTRY.0).unwrap(), &ENTRY.1);
    assert!(c.get(&NON_KEY).is_none());
    assert!(c.remove(&NON_KEY).is_none());

    assert_eq!(c.remove(&ENTRY.0).unwrap(), ENTRY.1);
    assert!(c.get(&ENTRY.0).is_none());
    assert!(c.is_empty());
    assert!(c.len() == 0);
    assert!(c.remove(&ENTRY.0).is_none());
}

#[test]
fn test_one_cap_evict() {
    const ENTRIES: [(u32, u16); 3] = [(1, 2), (3, 4), (5, 6)];

    let mut c: ConstLru<u32, u16, 1, u8> = ConstLru::new();
    c.insert(ENTRIES[0].0, ENTRIES[0].1);
    for (prev, (k, v)) in ENTRIES.into_iter().skip(1).enumerate() {
        assert!(c.is_full());
        assert_eq!(
            c.insert(k, v).unwrap(),
            InsertReplaced::LruEvicted(ENTRIES[prev].0, ENTRIES[prev].1)
        );
        assert_eq!(c.get(&k).unwrap(), &v);
    }
}

#[test]
fn test_one_cap_get_mut() {
    const K: u16 = 1;
    const V_OLD: u64 = 2;
    const V_NEW: u64 = 3;

    let mut c: ConstLru<u16, u64, 1, u8> = ConstLru::new();
    c.insert(K, V_OLD);
    assert_eq!(c.get(&K).unwrap(), &V_OLD);
    *c.get_mut(&K).unwrap() = V_NEW;
    assert_eq!(c.get(&K).unwrap(), &V_NEW);
}
