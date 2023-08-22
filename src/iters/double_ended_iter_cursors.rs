use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

/// assumes:
/// from_head: consume then increment
/// from_tail: decrement then consume
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DoubleEndedIterCursors<I, const CAP: usize> {
    /// from_head == from_tail means ended
    from_head: I,
    from_tail: I,
}

impl<I: PrimInt + Unsigned, const CAP: usize> DoubleEndedIterCursors<I, CAP> {
    pub fn new<K, V>(const_lru: &ConstLru<K, V, CAP, I>) -> Self {
        let (from_head, from_tail) = if const_lru.is_empty() {
            (I::zero(), I::zero())
        } else if const_lru.is_full() {
            (const_lru.head, const_lru.cap())
        } else {
            (
                const_lru.head,
                const_lru.nexts[const_lru.tail.to_usize().unwrap()],
            )
        };
        Self {
            from_head,
            from_tail,
        }
    }

    /// assumes next is valid
    pub fn advance_from_head<K, V>(&mut self, const_lru: &ConstLru<K, V, CAP, I>) {
        self.from_head = const_lru.nexts[self.get_from_head_idx()];
    }

    pub fn retreat_from_tail<K, V>(&mut self, const_lru: &ConstLru<K, V, CAP, I>) {
        self.from_tail = if self.from_tail == const_lru.cap() {
            const_lru.tail
        } else {
            const_lru.prevs[self.get_from_tail_idx()]
        };
    }

    pub fn get_from_head_idx(&self) -> usize {
        self.from_head.to_usize().unwrap()
    }

    pub fn get_from_tail_idx(&self) -> usize {
        self.from_tail.to_usize().unwrap()
    }

    pub fn has_ended(&self) -> bool {
        self.from_head == self.from_tail
    }

    pub fn get_from_head(&self) -> I {
        self.from_head
    }

    pub fn get_from_tail(&self) -> I {
        self.from_tail
    }
}
