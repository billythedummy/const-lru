use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InOrderTraversalState {
    /// traversal has completed the left subtree
    Left,
    /// traversal has completed for self
    This,
    /// traversal has completed for right subtree
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InOrderTraverser<I> {
    current: I,
    state: InOrderTraversalState,
}

impl<I: PrimInt + Unsigned> InOrderTraverser<I> {
    pub fn from_largest<K: Ord, V, const CAP: usize>(const_lru: &ConstLru<K, V, CAP, I>) -> Self {
        Self {
            current: const_lru.cap(),
            state: InOrderTraversalState::Right,
        }
    }

    pub fn from_smallest<K: Ord, V, const CAP: usize>(const_lru: &ConstLru<K, V, CAP, I>) -> Self {
        if const_lru.root == const_lru.cap() {
            Self::from_largest(const_lru)
        } else {
            Self {
                current: const_lru.find_leftmost(const_lru.root),
                state: InOrderTraversalState::Left,
            }
        }
    }

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
                    // in-order successor
                    // guaranteed to != current since right is valid
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

    /// Handles current == CAP: goes to rightmost of tree, state::Right
    /// end state of reverse iteration is current = leftmost node, state = Left
    pub fn prev<K: Ord, V, const CAP: usize>(&mut self, const_lru: &ConstLru<K, V, CAP, I>) {
        match self.state {
            InOrderTraversalState::Left => {
                let left = const_lru.lefts[self.current.to_usize().unwrap()];
                if left == const_lru.cap() {
                    // never root if left, so current always has a parent
                    let predecessor = const_lru.find_in_order_predecessor_ancestor(self.current);
                    // assert!(predecessor != const_lru.cap()) iter would have ended
                    self.current = predecessor;
                    self.state = InOrderTraversalState::This;
                } else {
                    self.current = left;
                    self.state = InOrderTraversalState::Right;
                }
            }
            InOrderTraversalState::This => self.state = InOrderTraversalState::Left,
            InOrderTraversalState::Right => {
                if self.current == const_lru.cap() {
                    self.current = const_lru.root;
                } else {
                    let c = self.current.to_usize().unwrap();
                    let right = const_lru.rights[c];
                    if right == const_lru.cap() {
                        self.state = InOrderTraversalState::This;
                    } else {
                        self.current = right;
                    }
                }
            }
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
        Self {
            from_largest: InOrderTraverser::from_largest(const_lru),
            from_smallest: InOrderTraverser::from_smallest(const_lru),
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

    pub fn retreat_from_largest<K: Ord, V, const CAP: usize>(
        &mut self,
        const_lru: &ConstLru<K, V, CAP, I>,
    ) {
        self.from_largest.prev(const_lru);
    }

    pub fn has_ended(&self) -> bool {
        self.from_smallest == self.from_largest
    }
}
