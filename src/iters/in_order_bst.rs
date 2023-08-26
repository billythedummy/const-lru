use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InOrderTraversalState {
    Left,
    This,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InOrderTraverser<I> {
    current: I,
    state: InOrderTraversalState,
}

impl<I: PrimInt + Unsigned> InOrderTraverser<I> {
    /// Assumes current is valid
    /// end state of fwd iteration is current = CAP, state = Right
    pub fn next<K: Ord, V, const CAP: usize>(&mut self, const_lru: &ConstLru<K, V, CAP, I>) {
        match self.state {
            InOrderTraversalState::Left => self.state = InOrderTraversalState::This,
            InOrderTraversalState::This => {
                let right = const_lru.rights[self.current.to_usize().unwrap()];
                if right == const_lru.cap() {
                    self.state = InOrderTraversalState::Right;
                } else {
                    self.current = const_lru.find_leftmost(right);
                    self.state = InOrderTraversalState::Left;
                }
            }
            InOrderTraversalState::Right => {
                let parent = const_lru.parents[self.current.to_usize().unwrap()];
                if parent != const_lru.cap() {
                    let p = parent.to_usize().unwrap();
                    self.state = if const_lru.lefts[p] == self.current {
                        InOrderTraversalState::Left
                    } else {
                        InOrderTraversalState::Right
                    };
                }
                self.current = parent;
            }
        }
    }

    /// Assumes current is valid
    /// end state of reverse iteration is current = leftmost node, state = Left
    pub fn prev<K: Ord, V, const CAP: usize>(&mut self, const_lru: &ConstLru<K, V, CAP, I>) {
        match self.state {
            InOrderTraversalState::Left => {
                let parent = const_lru.parents[self.current.to_usize().unwrap()];
                if parent != const_lru.cap() {
                    let p = parent.to_usize().unwrap();
                    self.state = if const_lru.lefts[p] == self.current {
                        InOrderTraversalState::Left
                    } else {
                        InOrderTraversalState::Right
                    };
                }
                self.current = parent;
            }
            InOrderTraversalState::This => {
                let left = const_lru.lefts[self.current.to_usize().unwrap()];
                if left == const_lru.cap() {
                    self.state = InOrderTraversalState::Left;
                } else {
                    self.current = const_lru.find_rightmost(left);
                    self.state = InOrderTraversalState::Right;
                }
            }
            InOrderTraversalState::Right => self.state = InOrderTraversalState::This,
        }
    }
}

/// assumes:
/// from_head: consume then advance
/// from_tail: retreat then consume
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleEndedInOrderTraverser<I> {
    from_smallest: InOrderTraverser<I>,
    from_largest: InOrderTraverser<I>,
}

impl<I: PrimInt + Unsigned> DoubleEndedInOrderTraverser<I> {
    pub fn new<K: Ord, V, const CAP: usize>(const_lru: &ConstLru<K, V, CAP, I>) -> Self {
        let from_largest = InOrderTraverser {
            current: const_lru.cap(),
            state: InOrderTraversalState::Right,
        };
        let from_smallest = if const_lru.root == const_lru.cap() {
            from_largest
        } else {
            InOrderTraverser {
                current: const_lru.find_leftmost(const_lru.root),
                state: InOrderTraversalState::Left,
            }
        };
        Self {
            from_largest,
            from_smallest,
        }
    }

    pub fn get_from_smallest_current(&self) -> I {
        self.from_smallest.current
    }

    pub fn get_from_smallest_state(&self) -> InOrderTraversalState {
        self.from_smallest.state
    }

    pub fn get_from_largest_current(&self) -> I {
        self.from_largest.current
    }

    pub fn get_from_largest_state(&self) -> InOrderTraversalState {
        self.from_largest.state
    }

    pub fn advance_from_smallest<K: Ord, V, const CAP: usize>(
        &mut self,
        const_lru: &ConstLru<K, V, CAP, I>,
    ) {
        self.from_smallest.next(const_lru);
    }

    pub fn retreate_from_largest<K: Ord, V, const CAP: usize>(
        &mut self,
        const_lru: &ConstLru<K, V, CAP, I>,
    ) {
        self.from_largest.prev(const_lru);
    }

    pub fn has_ended(&self) -> bool {
        self.from_smallest == self.from_largest
    }
}
