// using Rc.strong_count to test dropping behaviour

use std::rc::Rc;

use const_lru::{ConstLru, InsertReplaced};

#[test]
fn drops() {
    let k = Rc::new(0);
    let v = Rc::new(1);
    {
        let mut c: ConstLru<Rc<u8>, Rc<u16>, 1, u8> = ConstLru::new();
        c.insert(k.clone(), v.clone());
        assert_eq!(Rc::strong_count(&k), 2);
        assert_eq!(Rc::strong_count(&v), 2);
    }
    assert_eq!(Rc::strong_count(&k), 1);
    assert_eq!(Rc::strong_count(&v), 1);
}

#[test]
fn clear_drops() {
    let k = Rc::new(0);
    let v = Rc::new(1);
    let mut c: ConstLru<Rc<u8>, Rc<u16>, 1, u8> = ConstLru::new();
    c.insert(k.clone(), v.clone());
    assert_eq!(Rc::strong_count(&k), 2);
    assert_eq!(Rc::strong_count(&v), 2);
    c.clear();
    assert_eq!(Rc::strong_count(&k), 1);
    assert_eq!(Rc::strong_count(&v), 1);
}

#[test]
fn remove_drops() {
    const K_VAL: u8 = 0;
    const V_VAL: u16 = 1;
    let k = Rc::new(K_VAL);
    let v = Rc::new(V_VAL);
    let mut c: ConstLru<Rc<u8>, Rc<u16>, 1, u8> = ConstLru::new();
    c.insert(k.clone(), v.clone());
    assert_eq!(Rc::strong_count(&k), 2);
    assert_eq!(Rc::strong_count(&v), 2);
    {
        let val = c.remove(&k).unwrap();
        assert_eq!(val, Rc::new(V_VAL));
        assert_eq!(Rc::strong_count(&k), 1);
        assert_eq!(Rc::strong_count(&v), 2);
    }
    assert_eq!(Rc::strong_count(&v), 1);
}

#[test]
fn mut_write_drops() {
    let k = Rc::new(0);
    let v = Rc::new(1);
    let mut c: ConstLru<Rc<u8>, Rc<u16>, 1, u8> = ConstLru::new();
    c.insert(k.clone(), v.clone());
    assert_eq!(Rc::strong_count(&k), 2);
    assert_eq!(Rc::strong_count(&v), 2);
    *c.get_mut(&k).unwrap() = Rc::new(2);
    assert_eq!(Rc::strong_count(&k), 2);
    assert_eq!(Rc::strong_count(&v), 1);
}

#[test]
fn eviction_no_double_free() {
    const K_VAL: u8 = 0;
    const V_VAL: u16 = 1;
    let k = Rc::new(K_VAL);
    let v = Rc::new(V_VAL);
    let mut c: ConstLru<Rc<u8>, Rc<u16>, 1, u8> = ConstLru::new();
    c.insert(k.clone(), v.clone());
    assert_eq!(Rc::strong_count(&k), 2);
    assert_eq!(Rc::strong_count(&v), 2);
    {
        let evicted = c.insert(Rc::new(2), Rc::new(3)).unwrap();
        assert_eq!(
            InsertReplaced::LruEvicted(Rc::new(K_VAL), Rc::new(V_VAL)),
            evicted
        );
        assert_eq!(Rc::strong_count(&k), 2);
        assert_eq!(Rc::strong_count(&v), 2);
    }
    assert_eq!(Rc::strong_count(&k), 1);
    assert_eq!(Rc::strong_count(&v), 1);
}

#[test]
fn insert_return_old_val_no_double_free() {
    const K_VAL: u8 = 0;
    const V_VAL: u16 = 1;
    let k = Rc::new(K_VAL);
    let v = Rc::new(V_VAL);
    let mut c: ConstLru<Rc<u8>, Rc<u16>, 1, u8> = ConstLru::new();
    c.insert(k.clone(), v.clone());
    assert_eq!(Rc::strong_count(&k), 2);
    assert_eq!(Rc::strong_count(&v), 2);
    {
        let k3 = k.clone();
        assert_eq!(Rc::strong_count(&k), 3);
        let old_val = c.insert(k3, Rc::new(3)).unwrap();
        assert_eq!(InsertReplaced::OldValue(Rc::new(V_VAL)), old_val);
        assert_eq!(Rc::strong_count(&k), 2);
        assert_eq!(Rc::strong_count(&v), 2);
    }
    assert_eq!(Rc::strong_count(&k), 2);
    assert_eq!(Rc::strong_count(&v), 1);
}

#[test]
fn clone_no_double_free() {
    let entries: [(Rc<u8>, Rc<u16>); 2] = [(Rc::new(0), Rc::new(1)), (Rc::new(2), Rc::new(3))];
    let mut c: ConstLru<Rc<u8>, Rc<u16>, 2, u8> = ConstLru::new();
    c.insert(entries[0].0.clone(), entries[0].1.clone());
    c.insert(entries[1].0.clone(), entries[1].1.clone());
    assert_eq!(Rc::strong_count(&entries[0].0), 2);
    assert_eq!(Rc::strong_count(&entries[0].1), 2);
    assert_eq!(Rc::strong_count(&entries[1].0), 2);
    assert_eq!(Rc::strong_count(&entries[1].1), 2);
    {
        let _cloned = c.clone();
        assert_eq!(Rc::strong_count(&entries[0].0), 3);
        assert_eq!(Rc::strong_count(&entries[0].1), 3);
        assert_eq!(Rc::strong_count(&entries[1].0), 3);
        assert_eq!(Rc::strong_count(&entries[1].1), 3);
    }
    assert_eq!(Rc::strong_count(&entries[0].0), 2);
    assert_eq!(Rc::strong_count(&entries[0].1), 2);
    assert_eq!(Rc::strong_count(&entries[1].0), 2);
    assert_eq!(Rc::strong_count(&entries[1].1), 2);
}

#[test]
fn into_iter_no_double_free() {
    let k = Rc::new(0);
    let v = Rc::new(1);
    let mut c: ConstLru<Rc<u8>, Rc<u16>, 1, u8> = ConstLru::new();
    c.insert(k.clone(), v.clone());
    assert_eq!(Rc::strong_count(&k), 2);
    assert_eq!(Rc::strong_count(&v), 2);
    let mut iter = c.into_iter();
    assert_eq!(Rc::strong_count(&k), 2);
    assert_eq!(Rc::strong_count(&v), 2);
    iter.next().unwrap(); // drop immediately
    assert_eq!(Rc::strong_count(&k), 1);
    assert_eq!(Rc::strong_count(&v), 1);
}

#[test]
fn into_iter_partially_consumed_no_double_free() {
    let entries: [(Rc<u8>, Rc<u16>); 2] = [(Rc::new(0), Rc::new(1)), (Rc::new(2), Rc::new(3))];

    {
        let mut c: ConstLru<Rc<u8>, Rc<u16>, 2, u8> = ConstLru::new();
        c.insert(entries[0].0.clone(), entries[0].1.clone());
        c.insert(entries[1].0.clone(), entries[1].1.clone());

        assert_eq!(Rc::strong_count(&entries[0].0), 2);
        assert_eq!(Rc::strong_count(&entries[0].1), 2);
        assert_eq!(Rc::strong_count(&entries[1].0), 2);
        assert_eq!(Rc::strong_count(&entries[1].1), 2);

        let mut iter = c.into_iter();

        assert_eq!(Rc::strong_count(&entries[0].0), 2);
        assert_eq!(Rc::strong_count(&entries[0].1), 2);
        assert_eq!(Rc::strong_count(&entries[1].0), 2);
        assert_eq!(Rc::strong_count(&entries[1].1), 2);

        iter.next().unwrap(); // drop [1] immediately

        assert_eq!(Rc::strong_count(&entries[0].0), 2);
        assert_eq!(Rc::strong_count(&entries[0].1), 2);
        assert_eq!(Rc::strong_count(&entries[1].0), 1);
        assert_eq!(Rc::strong_count(&entries[1].1), 1);
    }
    assert_eq!(Rc::strong_count(&entries[0].0), 1);
    assert_eq!(Rc::strong_count(&entries[0].1), 1);
    assert_eq!(Rc::strong_count(&entries[1].0), 1);
    assert_eq!(Rc::strong_count(&entries[1].1), 1);
}

#[test]
fn try_from_takes_ownership_of_entries() {
    let entries: [(Rc<u8>, Rc<u16>); 2] = [(Rc::new(0), Rc::new(1)), (Rc::new(2), Rc::new(3))];
    let cloned = entries.clone();
    assert_eq!(Rc::strong_count(&entries[0].0), 2);
    assert_eq!(Rc::strong_count(&entries[0].1), 2);
    assert_eq!(Rc::strong_count(&entries[1].0), 2);
    assert_eq!(Rc::strong_count(&entries[1].1), 2);
    {
        let _c: ConstLru<Rc<u8>, Rc<u16>, 2, u8> = ConstLru::try_from(cloned).unwrap();
        assert_eq!(Rc::strong_count(&entries[0].0), 2);
        assert_eq!(Rc::strong_count(&entries[0].1), 2);
        assert_eq!(Rc::strong_count(&entries[1].0), 2);
        assert_eq!(Rc::strong_count(&entries[1].1), 2);
    }
    assert_eq!(Rc::strong_count(&entries[0].0), 1);
    assert_eq!(Rc::strong_count(&entries[0].1), 1);
    assert_eq!(Rc::strong_count(&entries[1].0), 1);
    assert_eq!(Rc::strong_count(&entries[1].1), 1);
}

#[test]
fn try_from_no_double_free_on_failure() {
    let entries: [(Rc<u8>, Rc<u16>); 2] = [(Rc::new(0), Rc::new(1)), (Rc::new(0), Rc::new(2))];
    let cloned = entries.clone();
    assert_eq!(Rc::strong_count(&entries[0].0), 2);
    assert_eq!(Rc::strong_count(&entries[0].1), 2);
    assert_eq!(Rc::strong_count(&entries[1].0), 2);
    assert_eq!(Rc::strong_count(&entries[1].1), 2);
    {
        let err = ConstLru::<Rc<u8>, Rc<u16>, 2, u8>::try_from(cloned).unwrap_err();
        assert_eq!(err.0, Rc::new(0));
        assert_eq!(Rc::strong_count(&entries[0].0), 2);
        assert_eq!(Rc::strong_count(&entries[0].1), 1);
        assert_eq!(Rc::strong_count(&entries[1].0), 1);
        assert_eq!(Rc::strong_count(&entries[1].1), 1);
    }
    assert_eq!(Rc::strong_count(&entries[0].0), 1);
    assert_eq!(Rc::strong_count(&entries[0].1), 1);
    assert_eq!(Rc::strong_count(&entries[1].0), 1);
    assert_eq!(Rc::strong_count(&entries[1].1), 1);
}
