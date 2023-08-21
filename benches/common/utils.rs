use super::traits::Insert;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct BigStruct {
    pub a1: [usize; 32],
    pub a2: [usize; 32],
    pub a3: [usize; 32],
    pub a4: [usize; 32],
}

impl From<u8> for BigStruct {
    fn from(value: u8) -> Self {
        let mut s: Self = Default::default();
        s.a3[0] = value.into();
        s
    }
}

pub fn fill_up_all_u8_keys<C: Insert<K, V>, K: From<u8>, V: From<u8>>(container: &mut C) {
    for k in 0..u8::MAX {
        container.insert_no_ret(k.into(), k.into());
    }
}
