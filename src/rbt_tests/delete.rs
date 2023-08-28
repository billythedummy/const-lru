use crate::{
    ConstLru,
    RbColor::{Black, Red},
};

// Examples taken from https://www.youtube.com/watch?v=eoQpRtMpA9I

///       13B
///      /   \
///    8R     17R
///   / \     /  \
/// 1B  11B  15B 25B
///   \          / \
///    6R      22R  27R
fn tree_a() -> ConstLru<u8, u8, 10, u8> {
    let mut c: ConstLru<u8, u8, 10, u8> = ConstLru::new();
    // c.keys = [1, 6, 8, 11, 13, 15, 17, 22, 25, 27].map(MaybeUninit::new);
    c.root = 4; // 13
    c.rb_colors = [Black, Red, Red, Black, Black, Black, Red, Red, Black, Red];
    c.parents = [2, 0, 4, 2, 10, 6, 4, 8, 6, 8];
    c.lefts = [10, 10, 0, 10, 2, 10, 5, 10, 7, 10];
    c.rights = [1, 10, 3, 10, 6, 10, 8, 10, 9, 10];
    c
}

///    7B
///   /  \
/// 3B    18R
///      /   \
///     10B   22B
///    /   \    \
///   8R   11R   26R
fn tree_b() -> ConstLru<u8, u8, 8, u8> {
    let mut c: ConstLru<u8, u8, 8, u8> = ConstLru::new();
    // c.keys = [3, 7, 8, 10, 11, 18, 22, 26].map(MaybeUninit::new);
    c.root = 1; // 7
    c.rb_colors = [Black, Black, Red, Black, Red, Red, Black, Red];
    c.parents = [1, 8, 3, 5, 3, 1, 5, 6];
    c.lefts = [8, 0, 8, 2, 8, 3, 8, 8];
    c.rights = [8, 5, 8, 4, 8, 6, 7, 8];
    c
}

///       5B
///     /    \
///    2R     8B
///   /  \    / \
/// 1B    4B 7R  9R
fn tree_c() -> ConstLru<u8, u8, 7, u8> {
    let mut c: ConstLru<u8, u8, 7, u8> = ConstLru::new();
    // c.keys = [1, 2, 4, 5, 7, 8, 9].map(MaybeUninit::new);
    c.root = 3; // 5
    c.rb_colors = [Black, Red, Black, Black, Red, Black, Red];
    c.parents = [1, 3, 1, 7, 5, 3, 5];
    c.lefts = [7, 0, 7, 1, 7, 4, 7];
    c.rights = [7, 2, 7, 5, 7, 6, 7];
    c
}

/// Delete 6 from tree_a
#[test]
fn delete_easy_1() {
    let mut c = tree_a();

    c.remove_rb(1); // 6

    assert_eq!(c.root, 4);
    assert_eq!(
        c.rb_colors,
        [Black, Black, Red, Black, Black, Black, Red, Red, Black, Red]
    );
    assert_eq!(c.parents, [2, 10, 4, 2, 10, 6, 4, 8, 6, 8]);
    assert_eq!(c.lefts, [10, 10, 0, 10, 2, 10, 5, 10, 7, 10]);
    assert_eq!(c.rights, [10, 10, 3, 10, 6, 10, 8, 10, 9, 10]);
}

/// Delete 1 from tree_a
#[test]
fn delete_easy_2() {
    let mut c = tree_a();

    c.remove_rb(0); // 1

    assert_eq!(c.root, 4);
    assert_eq!(
        c.rb_colors,
        [Black, Black, Red, Black, Black, Black, Red, Red, Black, Red]
    );
    assert_eq!(c.parents, [10, 2, 4, 2, 10, 6, 4, 8, 6, 8]);
    assert_eq!(c.lefts, [10, 10, 1, 10, 2, 10, 5, 10, 7, 10]);
    assert_eq!(c.rights, [10, 10, 3, 10, 6, 10, 8, 10, 9, 10]);
}

/// Delete 17 from tree_a
#[test]
fn delete_easy_3() {
    let mut c = tree_a();

    c.remove_rb(6); // 17

    assert_eq!(c.root, 4);
    assert_eq!(
        c.rb_colors,
        [Black, Red, Red, Black, Black, Black, Black, Red, Black, Red]
    );
    assert_eq!(c.parents, [2, 0, 4, 2, 10, 7, 10, 4, 7, 8]);
    assert_eq!(c.lefts, [10, 10, 0, 10, 2, 10, 10, 5, 10, 10]);
    assert_eq!(c.rights, [1, 10, 3, 10, 7, 10, 10, 8, 9, 10]);
}

/// Delete 25 from tree_a
#[test]
fn delete_easy_4() {
    let mut c = tree_a();

    c.remove_rb(8); // 25

    assert_eq!(c.root, 4);
    assert_eq!(
        c.rb_colors,
        [Black, Red, Red, Black, Black, Black, Red, Red, Black, Black]
    );
    assert_eq!(c.parents, [2, 0, 4, 2, 10, 6, 4, 9, 10, 6]);
    assert_eq!(c.lefts, [10, 10, 0, 10, 2, 10, 5, 10, 10, 7]);
    assert_eq!(c.rights, [1, 10, 3, 10, 6, 10, 9, 10, 10, 10]);
}

// Delete 18 from tree_b
#[test]
fn delete_medium_5() {
    let mut c = tree_b();

    c.remove_rb(5); // 18

    assert_eq!(c.root, 1);
    assert_eq!(
        c.rb_colors,
        [Black, Black, Red, Black, Red, Black, Red, Black]
    );
    assert_eq!(c.parents, [1, 8, 3, 6, 3, 8, 1, 6]);
    assert_eq!(c.lefts, [8, 0, 8, 2, 8, 8, 3, 8]);
    assert_eq!(c.rights, [8, 6, 8, 4, 8, 8, 7, 8]);
}

// Delete 2 from tree_c
#[test]
fn delete_medium_6() {
    let mut c = tree_c();

    c.remove_rb(1); // 2

    assert_eq!(c.root, 3);
    assert_eq!(c.rb_colors, [Red, Black, Black, Black, Red, Black, Red]);
    assert_eq!(c.parents, [2, 7, 3, 7, 5, 3, 5]);
    assert_eq!(c.lefts, [7, 7, 0, 2, 7, 4, 7]);
    assert_eq!(c.rights, [7, 7, 7, 5, 7, 6, 7]);
}

// Delete 13 from tree_a
#[test]
fn delete_medium_7() {
    let mut c = tree_a();

    c.remove_rb(4); // 13

    assert_eq!(c.root, 5);
    assert_eq!(
        c.rb_colors,
        [Black, Red, Red, Black, Black, Black, Black, Red, Red, Black]
    );
    assert_eq!(c.parents, [2, 0, 5, 2, 10, 10, 8, 6, 5, 8]);
    assert_eq!(c.lefts, [10, 10, 0, 10, 10, 2, 10, 10, 6, 10]);
    assert_eq!(c.rights, [1, 10, 3, 10, 10, 8, 7, 10, 9, 10]);
}
