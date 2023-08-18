use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DoubleEndedIterCursors<I: PrimInt + Unsigned, const CAP: usize> {
    /// from_head == CAP is used as sentinel value to end iteration
    from_head: I,
    from_tail: I,
}

impl<I: PrimInt + Unsigned, const CAP: usize> DoubleEndedIterCursors<I, CAP> {
    pub fn new<K: Eq, V>(const_lru: &ConstLru<K, V, CAP, I>) -> Self {
        Self {
            from_head: const_lru.head,
            from_tail: const_lru.tail,
        }
    }

    pub fn advance_from_head<K: Eq, V>(&mut self, const_lru: &ConstLru<K, V, CAP, I>) {
        if self.from_head == self.from_tail {
            self.from_head = const_lru.cap();
        } else {
            self.from_head = const_lru.nexts[self.get_from_head_idx()];
        }
    }

    pub fn retreat_from_tail<K: Eq, V>(&mut self, const_lru: &ConstLru<K, V, CAP, I>) {
        if self.from_head == self.from_tail {
            self.from_head = const_lru.cap();
        } else {
            self.from_tail = const_lru.prevs[self.get_from_tail_idx()];
        }
    }

    pub fn get_from_head(&self) -> I {
        self.from_head
    }

    pub fn get_from_tail(&self) -> I {
        self.from_tail
    }

    pub fn get_from_head_idx(&self) -> usize {
        self.from_head.to_usize().unwrap()
    }

    pub fn get_from_tail_idx(&self) -> usize {
        self.from_tail.to_usize().unwrap()
    }

    pub fn has_ended(&self) -> bool {
        self.get_from_head_idx() == CAP
    }
}
