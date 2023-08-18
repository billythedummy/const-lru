use const_lru::ConstLru;
use num_traits::{PrimInt, Unsigned};

fn mru_not_empty<K: Eq, V, const CAP: usize, I: Unsigned + PrimInt>(
    const_lru: &ConstLru<K, V, CAP, I>,
) -> (&K, &V) {
    const_lru.iter().next().unwrap()
}

fn lru_not_empty<K: Eq, V, const CAP: usize, I: Unsigned + PrimInt>(
    const_lru: &ConstLru<K, V, CAP, I>,
) -> (&K, &V) {
    const_lru.iter().next_back().unwrap()
}

#[test]
fn test_inserted_is_mru() {
    const ENTRIES: [(u8, u16); 3] = [(1, 2), (3, 4), (5, 6)];
    let mut c: ConstLru<u8, u16, 3, u8> = ConstLru::new();
    for (k, v) in ENTRIES {
        c.insert(k, v);
        assert_eq!(mru_not_empty(&c), (&k, &v));
        assert_eq!(lru_not_empty(&c), (&ENTRIES[0].0, &ENTRIES[0].1));
    }
}

#[test]
fn test_fill_shuffle_empty_fill() {
    const ENTRIES: [(u32, u64); 3] = [(1, 2), (3, 4), (5, 6)];

    // head -> 2 <-> 1 <-> 0 <- tail
    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    for (k, v) in ENTRIES {
        c.insert(k, v);
    }

    // head -> 0 <-> 1 <-> 2 <- tail
    c.get(&ENTRIES[1].0).unwrap();
    c.get(&ENTRIES[0].0).unwrap();
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[0].0, &ENTRIES[0].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[1].0, &ENTRIES[1].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());

    // head -> 0 <-> 2 <- tail
    c.remove(&ENTRIES[1].0);
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[0].0, &ENTRIES[0].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());

    // head -> 2 <- tail
    c.remove(&ENTRIES[0].0);
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());

    // empty
    c.remove(&ENTRIES[2].0);
    assert!(c.is_empty());
    let mut iter = c.iter();
    assert!(iter.next().is_none());

    // head -> 2 <- tail
    c.insert(ENTRIES[2].0, ENTRIES[2].1);
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());

    // head -> 1 <-> 2 <- tail
    c.insert(ENTRIES[1].0, ENTRIES[1].1);
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[1].0, &ENTRIES[1].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());

    // head -> 0 <-> 1 <-> 2 <- tail
    c.insert(ENTRIES[0].0, ENTRIES[0].1);
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[0].0, &ENTRIES[0].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[1].0, &ENTRIES[1].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());
}
