use const_lru::{ConstLru, InsertReplaced};
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
fn inserted_is_mru() {
    const ENTRIES: [(u8, u16); 3] = [(1, 2), (3, 4), (5, 6)];
    let mut c: ConstLru<u8, u16, 3, u8> = ConstLru::new();
    for (k, v) in ENTRIES {
        c.insert(k, v);
        assert_eq!(mru_not_empty(&c), (&k, &v));
        assert_eq!(lru_not_empty(&c), (&ENTRIES[0].0, &ENTRIES[0].1));
    }
}

#[test]
fn evict_bs_2_replace_bs_0() {
    const ENTRY_EVICT: (u32, u64) = (5, 6);
    const ENTRIES: [(u32, u64); 2] = [(1, 2), (3, 4)];
    const ENTRY_REPLACE: (u32, u64) = (0, 7);

    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    c.insert(ENTRY_EVICT.0, ENTRY_EVICT.1);
    for (k, v) in ENTRIES {
        c.insert(k, v);
    }
    assert_eq!(
        c.insert(ENTRY_REPLACE.0, ENTRY_REPLACE.1).unwrap(),
        InsertReplaced::LruEvicted(ENTRY_EVICT.0, ENTRY_EVICT.1)
    );
    // make sure all get() works properly
    for (k, v) in ENTRIES {
        assert_eq!(*c.get(&k).unwrap(), v);
    }
    assert_eq!(*c.get(&ENTRY_REPLACE.0).unwrap(), ENTRY_REPLACE.1);
}

#[test]
fn evict_bs_0_replace_bs_2() {
    const ENTRY_EVICT: (u32, u64) = (0, 7);
    const ENTRIES: [(u32, u64); 2] = [(1, 2), (3, 4)];
    const ENTRY_REPLACE: (u32, u64) = (5, 6);

    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    c.insert(ENTRY_EVICT.0, ENTRY_EVICT.1);
    for (k, v) in ENTRIES {
        c.insert(k, v);
    }
    assert_eq!(
        c.insert(ENTRY_REPLACE.0, ENTRY_REPLACE.1).unwrap(),
        InsertReplaced::LruEvicted(ENTRY_EVICT.0, ENTRY_EVICT.1)
    );
    // make sure all get() works properly
    for (k, v) in ENTRIES {
        assert_eq!(*c.get(&k).unwrap(), v);
    }
    assert_eq!(*c.get(&ENTRY_REPLACE.0).unwrap(), ENTRY_REPLACE.1);
}

#[test]
fn evict_bs_1_replace_bs_1_replace_gt() {
    const ENTRY_EVICT: (u32, u64) = (2, 3);
    const ENTRIES: [(u32, u64); 2] = [(1, 2), (4, 5)];
    const ENTRY_REPLACE: (u32, u64) = (3, 4);

    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    c.insert(ENTRY_EVICT.0, ENTRY_EVICT.1);
    for (k, v) in ENTRIES {
        c.insert(k, v);
    }
    assert_eq!(
        c.insert(ENTRY_REPLACE.0, ENTRY_REPLACE.1).unwrap(),
        InsertReplaced::LruEvicted(ENTRY_EVICT.0, ENTRY_EVICT.1)
    );
    // make sure all get() works properly
    for (k, v) in ENTRIES {
        assert_eq!(*c.get(&k).unwrap(), v);
    }
    assert_eq!(c.len(), 3);
    assert_eq!(*c.get(&ENTRY_REPLACE.0).unwrap(), ENTRY_REPLACE.1);
}

#[test]
fn evict_bs_1_replace_bs_1_replace_lt() {
    const ENTRY_EVICT: (u32, u64) = (3, 4);
    const ENTRIES: [(u32, u64); 2] = [(1, 2), (4, 5)];
    const ENTRY_REPLACE: (u32, u64) = (2, 3);

    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    c.insert(ENTRY_EVICT.0, ENTRY_EVICT.1);
    for (k, v) in ENTRIES {
        c.insert(k, v);
    }
    assert_eq!(
        c.insert(ENTRY_REPLACE.0, ENTRY_REPLACE.1).unwrap(),
        InsertReplaced::LruEvicted(ENTRY_EVICT.0, ENTRY_EVICT.1)
    );
    // make sure all get() works properly
    for (k, v) in ENTRIES {
        assert_eq!(*c.get(&k).unwrap(), v);
    }
    assert_eq!(c.len(), 3);
    assert_eq!(*c.get(&ENTRY_REPLACE.0).unwrap(), ENTRY_REPLACE.1);
}

#[test]
fn replace_old_val_full() {
    const ENTRY_OLD: (u32, u64) = (1, 2);
    const ENTRIES: [(u32, u64); 2] = [(3, 4), (5, 6)];
    const VAL_REPLACE: u64 = 3;
    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    c.insert(ENTRY_OLD.0, ENTRY_OLD.1);
    for (k, v) in ENTRIES {
        c.insert(k, v);
    }
    assert_eq!(
        c.insert(ENTRY_OLD.0, VAL_REPLACE).unwrap(),
        InsertReplaced::OldValue(ENTRY_OLD.1)
    );
    assert_eq!(*c.get(&ENTRY_OLD.0).unwrap(), VAL_REPLACE);
}

#[test]
fn replace_old_val_not_full() {
    const ENTRY_OLD: (u32, u64) = (1, 2);
    const VAL_REPLACE: u64 = 3;
    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    c.insert(ENTRY_OLD.0, ENTRY_OLD.1);
    assert_eq!(
        c.insert(ENTRY_OLD.0, VAL_REPLACE).unwrap(),
        InsertReplaced::OldValue(ENTRY_OLD.1)
    );
    assert_eq!(*c.get(&ENTRY_OLD.0).unwrap(), VAL_REPLACE);
}

#[test]
fn remove_empty() {
    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    assert!(c.remove(&1).is_none());
}

#[test]
fn remove_only() {
    const ENTRY: (u32, u64) = (1, 2);
    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    c.insert(ENTRY.0, ENTRY.1);
    assert_eq!(c.remove(&ENTRY.0).unwrap(), ENTRY.1);
    assert!(c.get(&ENTRY.0).is_none());
    assert!(c.is_empty());
}

#[test]
fn remove_not_full() {
    const ENTRY: (u32, u64) = (1, 2);
    const OTHER: (u32, u64) = (3, 4);
    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    c.insert(ENTRY.0, ENTRY.1);
    c.insert(OTHER.0, OTHER.1);
    assert_eq!(c.remove(&ENTRY.0).unwrap(), ENTRY.1);
    assert!(c.get(&ENTRY.0).is_none());
    assert_eq!(*c.get(&OTHER.0).unwrap(), OTHER.1);
}

#[test]
fn remove_full() {
    const ENTRY: (u32, u64) = (1, 2);
    const OTHERS: [(u32, u64); 2] = [(3, 4), (5, 6)];
    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    c.insert(ENTRY.0, ENTRY.1);
    for (k, v) in OTHERS {
        c.insert(k, v);
    }
    assert_eq!(c.remove(&ENTRY.0).unwrap(), ENTRY.1);
    assert!(c.get(&ENTRY.0).is_none());
    for (k, v) in OTHERS {
        assert_eq!(*c.get(&k).unwrap(), v);
    }
}

#[test]
fn fill_shuffle_empty_fill() {
    const ENTRIES: [(u32, u64); 3] = [(1, 2), (5, 6), (3, 4)];

    // head -> 2 <-> 1 <-> 0 <- tail
    let mut c: ConstLru<u32, u64, 3, u8> = ConstLru::new();
    for (k, v) in ENTRIES {
        c.insert(k, v);
    }

    // head -> 0 <-> 1 <-> 2 <- tail
    assert_eq!(c.get(&ENTRIES[1].0).unwrap(), &ENTRIES[1].1);
    assert_eq!(c.get(&ENTRIES[0].0).unwrap(), &ENTRIES[0].1);
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[0].0, &ENTRIES[0].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[1].0, &ENTRIES[1].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());

    // head -> 0 <-> 2 <- tail
    assert_eq!(c.remove(&ENTRIES[1].0).unwrap(), ENTRIES[1].1);
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[0].0, &ENTRIES[0].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());

    // head -> 2 <- tail
    assert_eq!(c.remove(&ENTRIES[0].0).unwrap(), ENTRIES[0].1);
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());

    // empty
    assert_eq!(c.remove(&ENTRIES[2].0).unwrap(), ENTRIES[2].1);
    assert!(c.is_empty());
    let mut iter = c.iter();
    assert!(iter.next().is_none());

    // head -> 2 <- tail
    assert!(c.insert(ENTRIES[2].0, ENTRIES[2].1).is_none());
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());

    // head -> 1 <-> 2 <- tail
    assert!(c.insert(ENTRIES[1].0, ENTRIES[1].1).is_none());
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[1].0, &ENTRIES[1].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());

    // head -> 0 <-> 1 <-> 2 <- tail
    assert!(c.insert(ENTRIES[0].0, ENTRIES[0].1).is_none());
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[0].0, &ENTRIES[0].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[1].0, &ENTRIES[1].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert!(iter.next().is_none());
}
