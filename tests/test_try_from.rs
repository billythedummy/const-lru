use const_lru::ConstLru;

#[test]
fn try_from_zero_cap() {
    const ENTRIES: [(u8, u16); 0] = [];
    let c: ConstLru<u8, u16, 0, u8> = ENTRIES.try_into().unwrap();
    assert!(c.is_empty());
    assert!(c.is_full());
}

#[test]
fn try_from_one_cap() {
    const ENTRIES: [(u8, u16); 1] = [(1, 2)];
    let mut c: ConstLru<u8, u16, 1, u8> = ENTRIES.try_into().unwrap();
    assert!(c.is_full());

    assert_eq!(*c.get(&ENTRIES[0].0).unwrap(), ENTRIES[0].1);

    let mut iter = c.iter();
    assert_eq!(iter.next().unwrap(), (&ENTRIES[0].0, &ENTRIES[0].1));
    assert!(iter.next().is_none());

    assert_eq!(c.remove(&ENTRIES[0].0).unwrap(), ENTRIES[0].1);

    assert!(c.is_empty());
}

#[test]
fn try_from_three_cap() {
    const ENTRIES: [(u8, u16); 3] = [(1, 2), (3, 4), (5, 6)];
    let mut c: ConstLru<u8, u16, 3, u8> = ENTRIES.try_into().unwrap();
    assert!(c.is_full());

    // check lru order
    for (i, tup) in c.iter().enumerate() {
        assert_eq!(tup, (&ENTRIES[i].0, &ENTRIES[i].1));
    }

    // check get works
    for (k, v) in ENTRIES {
        assert_eq!(*c.get(&k).unwrap(), v);
    }

    // check remove works
    for (new_len, (k, v)) in ENTRIES.iter().enumerate().rev() {
        assert_eq!(c.remove(k).unwrap(), *v);
        assert_eq!(c.len(), u8::try_from(new_len).unwrap());
    }
    assert!(c.is_empty());
}
