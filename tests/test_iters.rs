use const_lru::ConstLru;

const ENTRIES: [(u8, u64); 3] = [(5, 6), (1, 2), (3, 4)];

fn entries_sorted_by_key() -> [(u8, u64); 3] {
    let mut copy = ENTRIES.clone();
    copy.sort_unstable_by_key(|(k, _v)| *k);
    copy
}

fn create_const_lru() -> ConstLru<u8, u64, 3, u8> {
    let mut c: ConstLru<u8, u64, 3, u8> = ConstLru::new();
    for (k, v) in ENTRIES {
        assert!(c.insert(k, v).is_none());
    }
    c
}

#[test]
fn empty_iter() {
    let c: ConstLru<u8, u64, 1, u8> = ConstLru::new();
    let mut iter = c.iter();
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn iter_fwd_only() {
    let c = create_const_lru();
    for (i, (k, v)) in c.iter().enumerate() {
        let rev_i = ENTRIES.len() - i - 1;
        assert_eq!(*k, ENTRIES[rev_i].0);
        assert_eq!(*v, ENTRIES[rev_i].1);
    }
}

#[test]
fn iter_rev_only() {
    let c = create_const_lru();
    for (i, (k, v)) in c.iter().rev().enumerate() {
        assert_eq!(*k, ENTRIES[i].0);
        assert_eq!(*v, ENTRIES[i].1);
    }
}

#[test]
fn iter_mitm() {
    let c = create_const_lru();
    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[2].0, &ENTRIES[2].1));
    assert_eq!(iter.next_back().unwrap(), (&ENTRIES[0].0, &ENTRIES[0].1));
    assert_eq!(iter.next().unwrap(), (&ENTRIES[1].0, &ENTRIES[1].1));
    assert!(iter.next().is_none());
    assert!(iter.next_back().is_none());
}

#[test]
fn empty_iter_mut() {
    let mut c: ConstLru<u8, u64, 1, u8> = ConstLru::new();
    let mut iter = c.iter_mut();
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn iter_mut_fwd_only() {
    let mut c = create_const_lru();
    for (i, (k, v)) in c.iter_mut().enumerate() {
        let rev_i = ENTRIES.len() - i - 1;
        assert_eq!(*k, ENTRIES[rev_i].0);
        assert_eq!(*v, ENTRIES[rev_i].1);
        *v = *v + 1;
    }
    for (k, v) in ENTRIES {
        assert_eq!(*c.get_untouched(&k).unwrap(), v + 1);
    }
}

#[test]
fn iter_mut_rev_only() {
    let mut c = create_const_lru();
    for (i, (k, v)) in c.iter_mut().rev().enumerate() {
        assert_eq!(*k, ENTRIES[i].0);
        assert_eq!(*v, ENTRIES[i].1);
        *v = *v + 1;
    }
    for (k, v) in ENTRIES {
        assert_eq!(*c.get_untouched(&k).unwrap(), v + 1);
    }
}

#[test]
fn iter_mut_mitm() {
    let mut c = create_const_lru();
    let mut iter = c.iter_mut();
    let (k, v) = iter.next().unwrap();
    assert_eq!(k, &ENTRIES[2].0);
    assert_eq!(v, &ENTRIES[2].1);
    *v = *v + 1;
    let (k, v) = iter.next_back().unwrap();
    assert_eq!(k, &ENTRIES[0].0);
    assert_eq!(v, &ENTRIES[0].1);
    *v = *v + 1;
    let (k, v) = iter.next().unwrap();
    assert_eq!(k, &ENTRIES[1].0);
    assert_eq!(v, &ENTRIES[1].1);
    *v = *v + 1;
    assert!(iter.next().is_none());
    assert!(iter.next_back().is_none());
    for (k, v) in ENTRIES {
        assert_eq!(*c.get_untouched(&k).unwrap(), v + 1);
    }
}

#[test]
fn empty_iter_key_order() {
    let c: ConstLru<u8, u64, 1, u8> = ConstLru::new();
    let mut iter = c.iter_key_order();
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn iter_key_order_fwd_only() {
    let c = create_const_lru();
    let sorted = entries_sorted_by_key();
    for (i, (k, v)) in c.iter_key_order().enumerate() {
        assert_eq!(*k, sorted[i].0);
        assert_eq!(*v, sorted[i].1);
    }
}

#[test]
fn iter_key_order_rev_only() {
    let c = create_const_lru();
    let sorted = entries_sorted_by_key();
    for (i, (k, v)) in c.iter_key_order().rev().enumerate() {
        let rev_i = sorted.len() - i - 1;
        assert_eq!(*k, sorted[rev_i].0);
        assert_eq!(*v, sorted[rev_i].1);
    }
}

#[test]
fn iter_key_order_mitm() {
    let c = create_const_lru();
    let sorted = entries_sorted_by_key();
    let mut iter = c.iter_key_order();
    assert_eq!(iter.next().unwrap(), (&sorted[0].0, &sorted[0].1));
    assert_eq!(iter.next_back().unwrap(), (&sorted[2].0, &sorted[2].1));
    assert_eq!(iter.next().unwrap(), (&sorted[1].0, &sorted[1].1));
    assert!(iter.next().is_none());
    assert!(iter.next_back().is_none());
}

#[test]
fn empty_iter_key_order_mut() {
    let mut c: ConstLru<u8, u64, 1, u8> = ConstLru::new();
    let mut iter = c.iter_key_order_mut();
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn iter_key_order_mut_fwd_only() {
    let mut c = create_const_lru();
    let sorted = entries_sorted_by_key();
    for (i, (k, v)) in c.iter_key_order_mut().enumerate() {
        assert_eq!(*k, sorted[i].0);
        assert_eq!(*v, sorted[i].1);
        *v = *v + 1;
    }
    for (k, v) in ENTRIES {
        assert_eq!(*c.get_untouched(&k).unwrap(), v + 1);
    }
}

#[test]
fn iter_key_order_mut_rev_only() {
    let mut c = create_const_lru();
    let sorted = entries_sorted_by_key();
    for (i, (k, v)) in c.iter_key_order_mut().rev().enumerate() {
        let rev_i = sorted.len() - i - 1;
        assert_eq!(*k, sorted[rev_i].0);
        assert_eq!(*v, sorted[rev_i].1);
        *v = *v + 1;
    }
    for (k, v) in ENTRIES {
        assert_eq!(*c.get_untouched(&k).unwrap(), v + 1);
    }
}

#[test]
fn iter_key_order_mut_mitm() {
    let mut c = create_const_lru();
    let sorted = entries_sorted_by_key();
    let mut iter = c.iter_key_order_mut();
    let (k, v) = iter.next().unwrap();
    assert_eq!(k, &sorted[0].0);
    assert_eq!(v, &sorted[0].1);
    *v = *v + 1;
    let (k, v) = iter.next_back().unwrap();
    assert_eq!(k, &sorted[2].0);
    assert_eq!(v, &sorted[2].1);
    *v = *v + 1;
    let (k, v) = iter.next().unwrap();
    assert_eq!(k, &sorted[1].0);
    assert_eq!(v, &sorted[1].1);
    *v = *v + 1;
    assert!(iter.next().is_none());
    assert!(iter.next_back().is_none());
    for (k, v) in ENTRIES {
        assert_eq!(*c.get_untouched(&k).unwrap(), v + 1);
    }
}
