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
fn easy_tree() -> ConstLru<u8, u8, 10, u8> {
    let mut c: ConstLru<u8, u8, 10, u8> = ConstLru::new();
    // c.keys = [1, 6, 8, 11, 13, 15, 17, 22, 25, 27].map(MaybeUninit::new);
    c.root = 4; // 13
    c.rb_colors = [Black, Red, Red, Black, Black, Black, Red, Red, Black, Red];
    c.parents = [2, 0, 4, 2, 10, 6, 4, 8, 6, 8];
    c.lefts = [10, 10, 0, 10, 2, 10, 5, 10, 7, 10];
    c.rights = [1, 10, 3, 10, 6, 10, 8, 10, 9, 10];
    c
}

/// Delete 6 from easy tree
#[test]
fn delete_easy_1() {
    let mut c = easy_tree();

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

/// Delete 1 from easy tree
#[test]
fn delete_easy_2() {
    let mut c = easy_tree();

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

/// Delete 17 from easy tree
#[test]
fn delete_easy_3() {
    let mut c = easy_tree();

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

/// Delete 25 from easy tree
#[test]
fn delete_easy_4() {
    let mut c = easy_tree();

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
