use const_lru::{ConstLru, Entry};

#[test]
#[should_panic]
fn zero_cap_entry_panic() {
    let mut c: ConstLru<u8, u8, 0, u8> = ConstLru::new();
    c.entry(1);
}

#[test]
fn vacant_insert_alloc() {
    let k = 2;
    let v = 2;
    let mut c: ConstLru<u8, u8, 3, u8> = ConstLru::new();

    c.insert(1, 1);
    assert!(c.get(&k).is_none());

    let Entry::Vacant(entry) = c.entry(k) else { panic!("not vacant") };
    let (m, opt) = entry.insert(v);

    assert_eq!(*m, v);
    assert!(opt.is_none());
    assert_eq!(*c.get(&k).unwrap(), v);
    assert_eq!(c.len(), 2);
}

#[test]
fn vacant_insert_evict_1() {
    let evicted_k = 1;
    let evicted_v = 3;
    let k = 2;
    let v = 2;
    let mut c: ConstLru<u8, u8, 1, u8> = ConstLru::new();

    c.insert(evicted_k, evicted_v);
    assert!(c.get(&k).is_none());

    let Entry::Vacant(entry) = c.entry(k) else { panic!("not vacant") };
    let (m, opt) = entry.insert(v);

    assert_eq!(*m, v);
    assert_eq!(opt.unwrap(), (evicted_k, evicted_v));
    assert_eq!(*c.get(&k).unwrap(), v);
    assert!(c.get(&evicted_k).is_none());
    assert_eq!(c.len(), 1);
}

#[test]
fn vacant_insert_evict_3() {
    let evicted_k = 1;
    let evicted_v = 1;
    let k = 4;
    let v = 4;
    let mut c: ConstLru<u8, u8, 3, u8> = ConstLru::new();

    c.insert(evicted_k, evicted_v);
    c.insert(2, 2);
    c.insert(3, 3);
    assert!(c.get(&k).is_none());

    let Entry::Vacant(entry) = c.entry(k) else { panic!("not vacant") };
    let (m, opt) = entry.insert(v);

    assert_eq!(*m, v);
    assert_eq!(opt.unwrap(), (evicted_k, evicted_v));
    assert_eq!(*c.get(&k).unwrap(), v);
    assert!(c.get(&evicted_k).is_none());
    assert_eq!(c.len(), 3);
}

#[test]
fn occupied_get() {
    let k = 1;
    let v = 2;
    let new_v = 3;
    let mut c: ConstLru<u8, u8, 3, u8> = ConstLru::new();
    c.insert(k, v);

    let Entry::Occupied(mut entry) = c.entry(k) else { panic!("not occupied") };

    assert_eq!(*entry.get(), v);
    assert_eq!(*entry.get_mut(), v);
    assert_eq!(*entry.get_untouched(), v);
    assert_eq!(*entry.get_mut_untouched(), v);

    *entry.get_mut() = new_v;

    assert_eq!(*entry.get(), new_v);
    assert_eq!(*entry.get_mut(), new_v);
    assert_eq!(*entry.get_untouched(), new_v);
    assert_eq!(*entry.get_mut_untouched(), new_v);

    assert_eq!(*c.get(&k).unwrap(), new_v);
    assert_eq!(c.len(), 1);
}

#[test]
fn occupied_insert() {
    let k = 1;
    let v = 2;
    let new_v = 5;
    let mut c: ConstLru<u8, u8, 3, u8> = ConstLru::new();
    c.insert(k, v);
    c.insert(3, 4);

    let Entry::Occupied(mut entry) = c.entry(k) else { panic!("not occupied") };

    assert_eq!(entry.insert(new_v), v);
    assert_eq!(*c.get(&k).unwrap(), new_v);
    assert_eq!(c.len(), 2);
}

#[test]
fn occupied_remove() {
    let k = 1;
    let v = 2;
    let mut c: ConstLru<u8, u8, 3, u8> = ConstLru::new();
    c.insert(k, v);
    c.insert(3, 4);

    let Entry::Occupied(entry) = c.entry(k) else { panic!("not occupied") };

    assert_eq!(entry.remove(), v);
    assert!(c.get(&k).is_none());
    assert_eq!(c.len(), 1);
}
