#![no_std]
#![doc = include_str!("../README.md")]

use core::borrow::Borrow;
use core::cmp::Ordering;
use core::mem::MaybeUninit;
use num_traits::{PrimInt, Unsigned};

mod iters;

#[cfg(test)]
mod rbt_tests;

pub use iters::into_iter::IntoIter;
pub use iters::iter::Iter;
pub use iters::iter_key_order::IterKeyOrder;
pub use iters::iter_key_order_mut::IterKeyOrderMut;
pub use iters::iter_mut::IterMut;

use iters::iter::IterIndexed;
use iters::iter_maybe_uninit::IterMaybeUninit;

/// Constant capacity key-addressed LRU cache.
///
/// Generics:
/// - `K`. Type of key. `Ord` is used for lookup and to address entries.
/// - `V`. Type of value.
/// - `CAP`. Capacity of the cache.
/// - `I`. Type of the index used. Must be an unsigned primitive type with bitwidth <= `usize`'s bitwidth.
#[derive(Debug)]
pub struct ConstLru<K, V, const CAP: usize, I: PrimInt + Unsigned = usize> {
    /// TODO: 8x more space-efficient to use 1 bit to encode red-black tree color
    /// but until https://github.com/rust-lang/rust/issues/76560 generic_const_exprs
    /// and https://github.com/rust-lang/rust/issues/88581 int_roundings lands in stable
    /// we can't do `rb_colors: [u8; CAP.div_ceil(8)],`
    rb_colors: [RbColor; CAP],

    len: I,

    /// root of the bst
    ///
    /// CAP if empty
    root: I,

    /// head is index of most recently used
    ///
    /// can be any value if cache is empty
    head: I,

    /// tail is index of least recently used
    ///
    /// if cache is empty, tail is the first slot of unallocated memory / "free-list"
    /// else, next of the tail is the first slot of unallocated memory / "free-list"
    ///
    /// tail is always < CAP
    tail: I,

    /// disregard if value == CAP
    lefts: [I; CAP],

    /// disregard if value == CAP
    rights: [I; CAP],

    /// disregard if value == CAP
    /// Saving parent links results in memory overhead but
    /// enables in-order traversal without either
    /// - recursion, which might stack overflow
    /// - use of a stack, which requires dynamic allocation,
    /// - morris traversal, which requires mut reference to change right links
    parents: [I; CAP],

    /// disregard if value == CAP
    nexts: [I; CAP],

    /// disregard if value == CAP
    prevs: [I; CAP],

    keys: [MaybeUninit<K>; CAP],

    values: [MaybeUninit<V>; CAP],
}

impl<K, V, const CAP: usize, I: PrimInt + Unsigned> ConstLru<K, V, CAP, I> {
    /// Creates a new empty `ConstLru` on the stack
    ///
    /// panics if
    /// - `CAP > I::MAX`
    /// - `I::MAX > usize::MAX`
    ///
    /// WARNING: this might result in runtime stack overflow errors for large CAP.
    /// Use [`Self::init_at_alloc`] to initialize larger variants at preallocated memory
    pub fn new() -> Self {
        let mut res: MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe {
            Self::init_at_alloc(res.as_mut_ptr());
            res.assume_init()
        }
    }

    /// Initializes the ConstLru at a region of allocated memory
    ///
    /// # Safety
    /// `ptr` must point to uninitialized memory, since init()
    /// overwrites the data at `ptr`
    ///
    /// panics if
    /// - `CAP > I::MAX`
    /// - `I::MAX > usize::MAX`
    ///
    /// Example:
    ///
    /// ```
    /// use const_lru::ConstLru;
    /// use std::alloc::{alloc, Layout};
    ///
    /// let layout = Layout::new::<ConstLru<u32, u16, 10_000, u16>>();
    /// let container: Box<ConstLru<u32, u16, 10_000, u16>> = unsafe {
    ///     let ptr = alloc(layout) as *mut ConstLru<u32, u16, 10_000, u16>;
    ///     ConstLru::init_at_alloc(ptr);
    ///     Box::from_raw(ptr)
    /// };
    /// ```
    pub unsafe fn init_at_alloc(ptr: *mut Self) {
        let mut_ref = &mut *ptr;
        mut_ref.init();
    }

    /// Initializes the ConstLru. Doing it as a method instead of
    /// creating inline avoids running into stack size limits for large CAP
    fn init(&mut self) {
        let i_max = I::max_value()
            .to_usize()
            .unwrap_or_else(|| panic!("I::MAX > usize::MAX"));
        if CAP > i_max {
            panic!("CAP > I::MAX");
        }

        let cap = I::from(CAP).unwrap();

        self.rb_colors = [RbColor::Black; CAP];
        self.len = I::zero();
        self.root = cap;
        self.head = cap;
        self.tail = I::zero();
        self.lefts = [cap; CAP];
        self.rights = [cap; CAP];
        self.parents = [cap; CAP];

        // [1, 2, ..., cap-1, cap]
        for i in 0..CAP {
            self.nexts[i] = I::from(i + 1).unwrap();
        }

        // [cap, 0, 1, ..., cap-2]
        if CAP > 0 {
            self.prevs[0] = cap;
            for i in 1..CAP {
                self.prevs[i] = I::from(i - 1).unwrap();
            }
        }

        // keys and values should remain uninit
    }

    /// private helper fn.
    ///
    /// Unlinks the node at `index` from the doubly-linked list,
    /// patching its previous and next nodes, as well as self.head and self.tail if required.
    ///
    /// Can be used on both valid and invalid nodes.
    ///
    /// When this fn returns, `index`'s next and prev should be treated as invalid
    ///
    /// `self.head` and `self.tail` are not modified if only 1 elem in list
    ///
    /// Requirements:
    /// - index < CAP
    fn unlink_node(&mut self, index: I) {
        let i = index.to_usize().unwrap();
        let next = self.nexts[i];
        let prev = self.prevs[i];

        // index.next.prev = index.prev
        if next != self.cap() {
            self.prevs[next.to_usize().unwrap()] = prev;
        }

        // index.prev.next = index.next
        if prev != self.cap() {
            self.nexts[prev.to_usize().unwrap()] = next;
        }

        let is_one_elem_list = self.head == self.tail;

        if self.head == index && !is_one_elem_list {
            self.head = next;
        }

        if self.tail == index && !is_one_elem_list {
            self.tail = prev;
        }
    }

    /// private helper fn.
    ///
    /// Moves the element at index to the most-recently-used position.
    ///
    /// Requirements:
    /// - !self.is_empty()
    /// - index must be that of a valid node
    fn move_to_head(&mut self, index: I) {
        if self.head == index {
            return;
        }

        self.unlink_node(index);
        let i = index.to_usize().unwrap();

        // since self.head != index
        // and index is valid,
        // head must be valid
        let head = self.head;
        self.prevs[i] = self.cap();
        self.nexts[i] = head;

        self.prevs[head.to_usize().unwrap()] = index;

        self.head = index;
    }

    /// Returns rb color of the node at index, black if index == CAP
    /// TODO: refactor this to use bitwise operators for 1-bit RbColor. See self.rb_colors doc comment.
    fn get_rb_color(&self, index: I) -> RbColor {
        if index == self.cap() {
            return RbColor::Black;
        }
        self.rb_colors[index.to_usize().unwrap()]
    }

    /// Assumes index is valid
    /// TODO: refactor this to use bitwise operators for 1-bit RbColor. See self.rb_colors doc comment.
    fn set_rb_color(&mut self, index: I, new_color: RbColor) {
        self.rb_colors[index.to_usize().unwrap()] = new_color;
    }

    /// Assumes parent is valid.
    /// Inserts the node as the `parent_dir` child of parent.
    /// This overwrites the parent links of node and the left/right link of parent
    ///
    /// Node can be:
    /// - newly initialized leaf
    /// - unlinked node
    /// - CAP
    ///
    /// `(parent_index, parent_dir)` should be that returned by find_in_bst() or unlink_bst_node_from_parent()
    ///
    /// if `parent_index == CAP`, inserts at root
    ///
    /// Does not modify the node's own left + right indices
    fn insert_bst_node(&mut self, node_index: I, (parent_index, parent_dir): (I, BstChild)) {
        if node_index != self.cap() {
            let node_i = node_index.to_usize().unwrap();
            self.parents[node_i] = parent_index;
        }
        // if parent_index is CAP, replace root
        if parent_index == self.cap() {
            self.root = node_index;
            return;
        }
        let p = parent_index.to_usize().unwrap();
        match parent_dir {
            BstChild::Left => self.lefts[p] = node_index,
            BstChild::Right => self.rights[p] = node_index,
        }
    }

    /// Assumes node has a non-empty right-subtree
    /// Returns the node's in-order successor: leftmost child of right subtree
    /// return value always guaranteed to != CAP
    fn find_in_order_successor_right_subtree(&self, node_index: I) -> I {
        let right_subtree_root = self.rights[node_index.to_usize().unwrap()];
        self.find_leftmost(right_subtree_root)
    }

    /// Assumes node is a valid node
    /// Returns itself if no left children
    fn find_leftmost(&self, node_index: I) -> I {
        let mut curr = node_index;
        let mut left = self.lefts[curr.to_usize().unwrap()];
        while left != self.cap() {
            curr = left;
            left = self.lefts[curr.to_usize().unwrap()];
        }
        curr
    }

    /// Assumes node has a valid parent
    /// Returns CAP if no predecessor ie node is leftmost node
    fn find_in_order_predecessor_ancestor(&self, node_index: I) -> I {
        let mut curr = node_index;
        let mut parent = self.parents[curr.to_usize().unwrap()];
        while self.rights[parent.to_usize().unwrap()] != curr && parent != self.cap() {
            curr = parent;
            parent = self.parents[curr.to_usize().unwrap()];
        }
        parent
    }

    /// Assumes node is valid.
    ///
    /// Set node's parent's left/right link pointing to node to CAP, and
    /// set node's parent link to CAP
    ///
    /// Returns (parent_kv_i, was_deleted_node_parent's_left_or_right_child)
    ///
    /// parent_kv_i == CAP if node was root
    fn unlink_bst_node_from_parent(&mut self, node_index: I) -> (I, BstChild) {
        let node_i = node_index.to_usize().unwrap();
        let parent = self.parents[node_i];
        self.parents[node_i] = self.cap();
        // node is root node
        if parent == self.cap() {
            return (parent, BstChild::Left);
        }
        let p = parent.to_usize().unwrap();
        let parent_dir = if self.lefts[p] == node_index {
            self.lefts[p] = self.cap();
            BstChild::Left
        } else {
            self.rights[p] = self.cap();
            BstChild::Right
        };
        (parent, parent_dir)
    }

    /// x          o      
    ///  \        / \
    ///   o  ->  x   o
    ///    \
    ///     o
    /// Assumes node has valid right child
    fn left_rotate(&mut self, index: I) {
        let i = index.to_usize().unwrap();
        let right = self.rights[i];
        let r = right.to_usize().unwrap();
        let left_of_right = self.lefts[r];

        if left_of_right != self.cap() {
            self.unlink_bst_node_from_parent(left_of_right);
        }
        self.unlink_bst_node_from_parent(right);
        let (parent, parent_dir) = self.unlink_bst_node_from_parent(index);

        self.insert_bst_node(right, (parent, parent_dir));
        self.insert_bst_node(left_of_right, (index, BstChild::Right));
        self.insert_bst_node(index, (right, BstChild::Left));
    }

    /// hi cargo please stop parsing my 'o'
    /// and throwing expected one of 8 possible tokens
    ///     x       o      
    ///    /       / \
    ///   o   ->  o   x
    ///  /
    /// o   
    /// Assumes node has valid right child
    fn right_rotate(&mut self, index: I) {
        let i = index.to_usize().unwrap();
        let left = self.lefts[i];
        let l = left.to_usize().unwrap();
        let right_of_left = self.rights[l];

        if right_of_left != self.cap() {
            self.unlink_bst_node_from_parent(right_of_left);
        }
        self.unlink_bst_node_from_parent(left);
        let (parent, parent_dir) = self.unlink_bst_node_from_parent(index);

        self.insert_bst_node(left, (parent, parent_dir));
        self.insert_bst_node(right_of_left, (index, BstChild::Left));
        self.insert_bst_node(index, (left, BstChild::Right));
    }

    ///   x     B          
    ///  /     / \
    /// A  -> A   x
    ///  \
    ///   B
    fn left_right_rotate(&mut self, index: I) {
        let left = self.lefts[index.to_usize().unwrap()];
        self.left_rotate(left);
        self.right_rotate(index);
    }

    /// x        B
    ///  \      / \
    ///   A -> x   A
    ///  /
    /// B
    fn right_left_rotate(&mut self, index: I) {
        let right = self.rights[index.to_usize().unwrap()];
        self.right_rotate(right);
        self.left_rotate(index);
    }

    /// Insert a new leaf node into the red black tree
    fn insert_rb(&mut self, node_index: I, parent_info: (I, BstChild)) {
        self.insert_bst_node(node_index, parent_info);
        self.set_rb_color(node_index, RbColor::Red);
        let mut node = Some(node_index);
        while let Some(n) = node {
            node = self.insert_rb_fixup(n);
        }
    }

    /// Returns next node to examine and potentially fixup (the grandparent)
    /// Returns None if completed
    fn insert_rb_fixup(&mut self, node: I) -> Option<I> {
        // case-0: root
        if node == self.root {
            self.set_rb_color(node, RbColor::Black);
            return None;
        }
        // not root, parent must exist
        let n = node.to_usize().unwrap();
        let parent = self.parents[n];
        let parent_color = self.get_rb_color(parent);
        let node_color = self.get_rb_color(node);
        // no violation, we're done
        if !(parent_color == RbColor::Red && node_color == RbColor::Red) {
            return None;
        }

        // root always black -> if parent-child red violation then parent != root
        // -> grandparent must exist
        let p = parent.to_usize().unwrap();
        let parent_dir = if self.lefts[p] == node {
            BstChild::Left
        } else {
            BstChild::Right
        };

        let grandparent = self.parents[p];

        let g = grandparent.to_usize().unwrap();
        let (grandparent_dir, uncle) = if self.lefts[g] == parent {
            (BstChild::Left, self.rights[g])
        } else {
            (BstChild::Right, self.lefts[g])
        };
        let uncle_color = self.get_rb_color(uncle);

        // case-1: parent & uncle red -> recolor and check grandparent
        if uncle_color == RbColor::Red {
            self.set_rb_color(uncle, RbColor::Black);
            self.set_rb_color(parent, RbColor::Black);
            self.set_rb_color(grandparent, RbColor::Red);
            return Some(grandparent);
        }
        // uncle is black

        // case-2: inserted node is "inner grandchild" (triangle)
        let is_right_left_inner_grandchild =
            grandparent_dir == BstChild::Right && parent_dir == BstChild::Left;
        let is_left_right_inner_grandchild =
            grandparent_dir == BstChild::Left && parent_dir == BstChild::Right;
        if is_right_left_inner_grandchild || is_left_right_inner_grandchild {
            if is_right_left_inner_grandchild {
                self.right_left_rotate(grandparent);
            } else {
                self.left_right_rotate(grandparent);
            }
            self.set_rb_color(node, RbColor::Black);
            self.set_rb_color(grandparent, RbColor::Red);
            return None;
        }

        // case-3: inserted node is "outer grandchild" (line)
        let is_right_outer_grandchild =
            grandparent_dir == BstChild::Right && parent_dir == BstChild::Right;
        if is_right_outer_grandchild {
            self.left_rotate(grandparent);
        } else {
            self.right_rotate(grandparent);
        }
        self.set_rb_color(parent, RbColor::Black);
        self.set_rb_color(grandparent, RbColor::Red);
        None
    }

    /// Assumes node is valid
    ///
    /// Deletes the given node from a bst, replacing it with
    ///
    /// - CAP if no children
    /// - its only child if only 1 child
    /// - its in-order successor if both children exist
    ///
    /// Returns ((parent, direction), deleted_node_color, replacement_node_color)
    ///
    /// (parent, direction) is that of the lowest modified parent. This is
    ///
    /// - CAP if node was root
    /// - parent of node if no children or only 1 child and direction of the node wrt parent
    /// - parent of in order successor if 2 children and direction of in order successor wrt its parent
    ///
    /// deleted_node_color is
    /// - color of deleted node if 1 or 0 children
    /// - color of in order successor if 2 children
    fn remove_rbt_node(&mut self, node_index: I) -> ((I, BstChild), RbColor) {
        let node_i = node_index.to_usize().unwrap();
        let parent = self.parents[node_i];
        let left = self.lefts[node_i];
        let right = self.rights[node_i];
        let original_color = self.get_rb_color(node_index);
        let lowest_modified_parent: (I, BstChild);
        let deleted_color: RbColor;
        if left == self.cap() && right == self.cap() {
            // case-1: no children, just clear
            lowest_modified_parent = self.unlink_bst_node_from_parent(node_index);
            deleted_color = original_color;
            if node_index == self.root {
                self.root = self.cap();
            }
        } else {
            let (replacement_index, parent_dir) = if left != self.cap() && right != self.cap() {
                let in_order_successor = self.find_in_order_successor_right_subtree(node_index);
                let (ios_parent, ios_parent_dir) =
                    self.unlink_bst_node_from_parent(in_order_successor);
                // in_order_successor must have no left children,
                // so replace ios with its right child
                let ios = in_order_successor.to_usize().unwrap();
                let ios_right = self.rights[ios];
                if ios_right != self.cap() {
                    self.unlink_bst_node_from_parent(ios_right);
                    self.insert_bst_node(ios_right, (ios_parent, ios_parent_dir));
                }
                // at this point ios has no left, right, parent
                // replace ios' left and right with node's left and right
                self.insert_bst_node(left, (in_order_successor, BstChild::Left));
                // must compute right again since mightve changed with ios_right modification above
                self.insert_bst_node(self.rights[node_i], (in_order_successor, BstChild::Right));

                let (_, parent_dir) = self.unlink_bst_node_from_parent(node_index);

                // recolor in_order_successor with original color for correct remove repairs
                let ios_color = self.get_rb_color(in_order_successor);
                self.set_rb_color(in_order_successor, original_color);

                lowest_modified_parent = if ios_parent == node_index {
                    // edge: in-order successor is node's right child
                    // then lowest_modified_parent should be the right child itself
                    (in_order_successor, BstChild::Right)
                } else {
                    (ios_parent, ios_parent_dir)
                };
                deleted_color = ios_color;

                (in_order_successor, parent_dir)
            } else {
                // only 1 child, replace node with child
                let (parent, parent_dir) = self.unlink_bst_node_from_parent(node_index);
                let replacement_index = if left != self.cap() { left } else { right };

                lowest_modified_parent = (parent, parent_dir);
                deleted_color = original_color;
                (replacement_index, parent_dir)
            };
            self.insert_bst_node(replacement_index, (parent, parent_dir));
        }

        // clear the removed node's left, right, parent, reset to black
        self.parents[node_i] = self.cap();
        self.lefts[node_i] = self.cap();
        self.rights[node_i] = self.cap();
        self.set_rb_color(node_index, RbColor::Black);
        (lowest_modified_parent, deleted_color)
    }

    /// Assumes node is valid
    /// Removes a node from the red-black tree
    fn remove_rb(&mut self, node_index: I) {
        let (lowest_modified_parent, deleted_color) = self.remove_rbt_node(node_index);
        if deleted_color == RbColor::Red {
            return;
        }
        let mut lowest_modified_parent = Some(lowest_modified_parent);
        while let Some(lmp) = lowest_modified_parent {
            lowest_modified_parent = self.remove_rb_fixup(lmp);
        }
    }

    fn remove_rb_fixup(&mut self, (parent, parent_dir): (I, BstChild)) -> Option<(I, BstChild)> {
        // case: root -> color root black and done
        if parent == self.cap() {
            if self.root != self.cap() {
                self.set_rb_color(self.root, RbColor::Black);
            }
            return None;
        }

        // not root, parent is a valid node
        let p = parent.to_usize().unwrap();
        let mut sibling = if let BstChild::Left = parent_dir {
            self.rights[p]
        } else {
            self.lefts[p]
        };

        // case: replacement is red -> color replacement black, finish
        let replacement = if parent_dir == BstChild::Left {
            self.lefts[p]
        } else {
            self.rights[p]
        };
        if self.get_rb_color(replacement) == RbColor::Red {
            // red means replacement != CAP
            self.set_rb_color(replacement, RbColor::Black);
            return None;
        }

        // case: red sibling ->
        // recolor sibling and parent then rotate then fallthrough
        if let RbColor::Red = self.get_rb_color(sibling) {
            self.set_rb_color(sibling, RbColor::Black);
            self.set_rb_color(parent, RbColor::Red);
            if let BstChild::Left = parent_dir {
                self.left_rotate(parent);
            } else {
                self.right_rotate(parent);
            }
            // update sibling and fallthrough since it will be one of cases below
            sibling = if let BstChild::Left = parent_dir {
                self.rights[p]
            } else {
                self.lefts[p]
            };
        }

        // case: black sibling with 2 black children
        let mut s = sibling.to_usize().unwrap();
        let mut right_of_sibling = self.rights[s];
        let mut left_of_sibling = self.lefts[s];
        if self.get_rb_color(right_of_sibling) == RbColor::Black
            && self.get_rb_color(left_of_sibling) == RbColor::Black
        {
            self.set_rb_color(sibling, RbColor::Red);

            // case: red parent -> recolor parent black and we're done
            if let RbColor::Red = self.get_rb_color(parent) {
                self.set_rb_color(parent, RbColor::Black);
                return None;
            }

            // case: black parent -> check parent
            let grandparent = self.parents[p];
            let grandparent_dir = if grandparent == self.cap()
                || self.lefts[grandparent.to_usize().unwrap()] == parent
            {
                BstChild::Left
            } else {
                BstChild::Right
            };
            return Some((grandparent, grandparent_dir));
        }

        // case: black sibling with at least 1 red child

        // case: outer newphew is black, inner nephew is red (therefore valid node)
        // -> recolor inner nephew black, sibling red, then rotate sibling in opposite dir,
        // then fallthrough to perform same coloring and roation as case-5
        if parent_dir == BstChild::Left && self.get_rb_color(right_of_sibling) == RbColor::Black {
            self.set_rb_color(left_of_sibling, RbColor::Black);
            self.set_rb_color(sibling, RbColor::Red);
            self.right_rotate(sibling);
            sibling = self.rights[p];
        } else if parent_dir == BstChild::Right
            && self.get_rb_color(left_of_sibling) == RbColor::Black
        {
            self.set_rb_color(right_of_sibling, RbColor::Black);
            self.set_rb_color(sibling, RbColor::Red);
            self.left_rotate(sibling);
            sibling = self.lefts[p];
        }
        s = sibling.to_usize().unwrap();
        right_of_sibling = self.rights[s];
        left_of_sibling = self.lefts[s];

        // case: outer nephew is red
        // -> recolor sibling in parent color,
        // color parent + outer nephew black,
        // rotate parent in same dir
        self.set_rb_color(sibling, self.get_rb_color(parent));
        self.set_rb_color(parent, RbColor::Black);
        if parent_dir == BstChild::Left {
            self.set_rb_color(right_of_sibling, RbColor::Black);
            self.left_rotate(parent);
        } else {
            self.set_rb_color(left_of_sibling, RbColor::Black);
            self.right_rotate(parent);
        }
        None
    }

    /// Creates an iterator that iterates through the keys and values of the `ConstLru` from most-recently-used to least-recently-used
    ///
    /// Does not change the LRU order of the elements.
    ///
    /// Double-ended: reversing iterates from least-recently-used to most-recently-used
    pub fn iter(&self) -> Iter<K, V, CAP, I> {
        Iter::new(self)
    }

    /// Creates an iterator that iterates through the keys and mutable values of the `ConstLru` from most-recently-used to least-recently-used
    ///
    /// Does not change the LRU order of the elements, even if mutated.
    ///
    /// Double-ended: reversing iterates from least-recently-used to most-recently-used
    pub fn iter_mut(&mut self) -> IterMut<K, V, CAP, I> {
        IterMut::new(self)
    }

    /// Clears the `ConstLru`, removing all key-value pairs.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Returns the maximum number of elements this `ConstLru` can hold
    pub fn cap(&self) -> I {
        I::from(CAP).unwrap()
    }

    /// Returns `true` if the `ConstLru` contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == I::zero()
    }

    /// Returns `true` if the `ConstLru` has reached max capacity.
    pub fn is_full(&self) -> bool {
        self.len() == self.cap()
    }

    /// Returns the number of elements in the `ConstLru`.
    pub fn len(&self) -> I {
        self.len
    }
}

impl<K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> ConstLru<K, V, CAP, I> {
    /// Inserts a key-value pair into the map. The entry is moved to the most-recently-used slot
    ///
    /// If `CAP == 0`, `None` is returned.
    ///
    /// If the map did not have this key present and is not full, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned in a [`InsertReplaced::OldValue`].
    /// The key is not updated, though; this matters for types that can be `==` without being identical.
    ///
    /// If the map is full, the least-recently used key-value pair is evicted and returned in a [`InsertReplaced::LruEvicted`].
    pub fn insert(&mut self, k: K, v: V) -> Option<InsertReplaced<K, V>> {
        if CAP == 0 {
            return None;
        }

        // case-1: existing
        let (parent_index, parent_dir) = match self.find_in_bst(&k) {
            Ok(index) => {
                let old_v = unsafe { self.values[index.to_usize().unwrap()].assume_init_mut() };
                let old_v_out = core::mem::replace(old_v, v);
                self.move_to_head(index);
                return Some(InsertReplaced::OldValue(old_v_out));
            }
            Err(i) => i,
        };

        // case-2: full, evict LRU
        if self.is_full() {
            // N > 0, tail must be valid
            let t = self.tail.to_usize().unwrap();
            let evicted_k = unsafe { self.keys[t].assume_init_read() };
            let evicted_v = unsafe { self.values[t].assume_init_read() };

            self.remove_rb(self.tail);

            // recalculate (parent_index, parent_dir) since tail deletion wouldve modified tree
            let (parent_index_recalc, parent_dir_recalc) = self.find_in_bst(&k).err().unwrap();
            self.keys[t].write(k);
            self.values[t].write(v);

            self.insert_rb(self.tail, (parent_index_recalc, parent_dir_recalc));

            self.move_to_head(self.tail);
            return Some(InsertReplaced::LruEvicted(evicted_k, evicted_v));
        }

        // case-3: alloc new node
        let free_index = if self.is_empty() {
            self.head = self.tail;
            self.tail
        } else {
            self.nexts[self.tail.to_usize().unwrap()]
        };
        self.tail = free_index;
        let f = free_index.to_usize().unwrap();
        self.keys[f].write(k);
        self.values[f].write(v);

        self.insert_rb(self.tail, (parent_index, parent_dir));

        self.move_to_head(self.tail);
        self.len = self.len + I::one();
        None
    }

    /// Removes a key from the `ConstLru`, returning the value at the key if the key was previously in the `ConstLru`.
    pub fn remove<Q: Ord>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        let index = self.find_in_bst(k).ok()?;
        let i = index.to_usize().unwrap();

        unsafe {
            self.keys[i].assume_init_drop();
        }
        let val = unsafe { self.values[i].assume_init_read() };

        // if len == 1, correct links are already in place
        if self.len() > I::one() {
            // len > 1
            // move to front of free list
            self.unlink_node(index);
            let t = self.tail.to_usize().unwrap();
            let first_free = self.nexts[t];

            if first_free < self.cap() {
                self.prevs[first_free.to_usize().unwrap()] = index;
            }
            self.nexts[i] = first_free;

            self.prevs[i] = self.tail;
            self.nexts[t] = index;
        }

        self.remove_rb(index);
        self.len = self.len - I::one();
        Some(val)
    }

    /// Returns a reference to the value corresponding to the key and moves entry to most-recently-used slot.
    ///
    /// To not update to most-recently-used, use [`Self::get_untouched`]
    pub fn get<Q: Ord>(&mut self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let index = self.find_in_bst(k).ok()?;
        self.move_to_head(index);
        Some(unsafe { self.values[index.to_usize().unwrap()].assume_init_ref() })
    }

    /// Returns a mutable reference to the value corresponding to the key and moves entry to most-recently-used slot.
    ///
    /// To not update to most-recently-used, use [`Self::get_mut_untouched`]
    pub fn get_mut<Q: Ord>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        let index = self.find_in_bst(k).ok()?;
        self.move_to_head(index);
        Some(unsafe { self.values[index.to_usize().unwrap()].assume_init_mut() })
    }

    /// Ok(kv_i) if found
    ///
    /// Err(parent_leaf_kv_i, should_this_node_be_inserted_as_parent's_left_or_right_child) if not found
    /// parent_leaf_kv_i = CAP and .1 is disregarded if tree is empty
    fn find_in_bst<Q: Ord>(&self, k: &Q) -> Result<I, (I, BstChild)>
    where
        K: Borrow<Q>,
    {
        if self.root == self.cap() {
            return Err((self.cap(), BstChild::Left));
        }
        let mut curr = self.root;
        loop {
            let parent_dir;
            let parent_index = curr;
            let i = curr.to_usize().unwrap();
            let curr_k = unsafe { self.keys[i].assume_init_ref() };
            match curr_k.borrow().cmp(k) {
                Ordering::Equal => return Ok(curr),
                Ordering::Less => {
                    curr = self.rights[i];
                    parent_dir = BstChild::Right;
                }
                Ordering::Greater => {
                    curr = self.lefts[i];
                    parent_dir = BstChild::Left;
                }
            }
            if curr == self.cap() {
                return Err((parent_index, parent_dir));
            }
        }
    }

    /// Returns a reference to the value corresponding to the key without updating the entry to most-recently-used slot
    ///
    /// To update to most-recently-used, use [`Self::get`]
    pub fn get_untouched<Q: Ord>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let index = self.find_in_bst(k).ok()?;
        Some(unsafe { self.values[index.to_usize().unwrap()].assume_init_ref() })
    }

    /// Returns a mutable reference to the value corresponding to the key without updating the entry to most-recently-used slot
    ///
    /// To update to most-recently-used, use [`Self::get_mut`]
    pub fn get_mut_untouched<Q: Ord>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        let index = self.find_in_bst(k).ok()?;
        Some(unsafe { self.values[index.to_usize().unwrap()].assume_init_mut() })
    }

    /// Creates an iterator that iterates through the keys and values of the `ConstLru` in the order of its keys
    ///
    /// Does not change the LRU order of the elements.
    ///
    /// Double-ended: reversing iterates from descending order of its keys
    pub fn iter_key_order(&self) -> IterKeyOrder<K, V, CAP, I> {
        IterKeyOrder::new(self)
    }

    /// Creates an iterator that iterates through the keys and mutable values of the `ConstLru` in the order of its keys
    ///
    /// Does not change the LRU order of the elements, even if mutated.
    ///
    /// Double-ended: reversing iterates from descending order of its keys
    pub fn iter_key_order_mut(&mut self) -> IterKeyOrderMut<K, V, CAP, I> {
        IterKeyOrderMut::new(self)
    }
}

impl<K: Clone, V: Clone, const CAP: usize, I: PrimInt + Unsigned> Clone for ConstLru<K, V, CAP, I> {
    fn clone(&self) -> Self {
        let mut res = Self {
            rb_colors: self.rb_colors,
            len: self.len,
            root: self.root,
            head: self.head,
            tail: self.tail,
            lefts: self.lefts,
            rights: self.rights,
            parents: self.parents,
            nexts: self.nexts,
            prevs: self.prevs,
            keys: unsafe { MaybeUninit::uninit().assume_init() },
            values: unsafe { MaybeUninit::uninit().assume_init() },
        };
        for (i, k, v) in IterIndexed::new(self) {
            res.keys[i.to_usize().unwrap()].write(k.clone());
            res.values[i.to_usize().unwrap()].write(v.clone());
        }
        res
    }
}

impl<K, V, const CAP: usize, I: PrimInt + Unsigned> Default for ConstLru<K, V, CAP, I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const CAP: usize, I: PrimInt + Unsigned> Drop for ConstLru<K, V, CAP, I> {
    fn drop(&mut self) {
        for (k, v) in IterMaybeUninit::new(self) {
            unsafe {
                k.assume_init_drop();
                v.assume_init_drop();
            }
        }
    }
}

impl<K, V, const CAP: usize, I: PrimInt + Unsigned> IntoIterator for ConstLru<K, V, CAP, I> {
    type Item = <IntoIter<K, V, CAP, I> as Iterator>::Item;

    type IntoIter = IntoIter<K, V, CAP, I>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

/// Optional return type of [`ConstLru::insert`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertReplaced<K, V> {
    LruEvicted(K, V),
    OldValue(V),
}

/// Error type of `TryFrom<[(K, V); CAP]>`
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct DuplicateKeysError<K>(
    /// The first duplicate key found
    pub K,
);

/// Creates a full ConstLru cache from an `entries` array.
///
/// Assumes `entries` is in MRU -> LRU order.
///
/// Returns error if duplicate keys found.
impl<K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> TryFrom<[(K, V); CAP]>
    for ConstLru<K, V, CAP, I>
{
    type Error = DuplicateKeysError<K>;

    fn try_from(entries: [(K, V); CAP]) -> Result<Self, Self::Error> {
        let mut res = Self::new();
        res.len = res.cap();
        res.head = I::zero();
        res.tail = if CAP > 0 {
            res.len - I::one()
        } else {
            I::zero()
        };

        // write all values in first so that drop self cleans up correctly
        for (i, (k, v)) in entries.into_iter().enumerate() {
            res.keys[i].write(k);
            res.values[i].write(v);
        }

        // build the bst element-by-element
        for i in 0..CAP {
            let k = unsafe { res.keys[i].assume_init_ref() };
            let (parent_index, parent_dir) = match res.find_in_bst(k) {
                Ok(existing) => {
                    // remove from list so no double free
                    res.unlink_node(existing);
                    res.len = res.len - I::one();

                    // cleanup value
                    let e = existing.to_usize().unwrap();
                    unsafe { res.values[e].assume_init_drop() };
                    let k_copied_out = unsafe { res.keys[e].assume_init_read() };
                    return Err(DuplicateKeysError(k_copied_out));
                }
                Err(parent_info) => parent_info,
            };
            res.insert_rb(I::from(i).unwrap(), (parent_index, parent_dir));
        }

        Ok(res)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum BstChild {
    Left,
    Right,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum RbColor {
    Red,
    Black,
}
