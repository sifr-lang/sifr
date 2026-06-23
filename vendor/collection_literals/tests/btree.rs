use std::collections::{BTreeMap, BTreeSet};

use collection_literals::{btree, collection};

#[test]
fn it_should_create_defaults() {
    let tested_set: BTreeSet<u8> = btree! {};
    let desired_set: BTreeSet<u8> = collection! {};
    assert_eq!(tested_set, desired_set);

    let tested_set: BTreeMap<u8, bool> = btree! {};
    let desired_set: BTreeMap<u8, bool> = collection! {};
    assert_eq!(tested_set, desired_set);
}

#[test]
fn it_should_properly_create_sets() {
    let tested_set = btree! {1, 2, 3, 4, 5, 6, 7, 8, 9};
    let desired_set: BTreeSet<i32> = collection! {1, 2, 3, 4, 5, 6, 7, 8, 9};
    assert_eq!(tested_set, desired_set);

    let tested_set = btree! {1, 1, 1, 8, 8, 8};
    let desired_set: BTreeSet<i32> = collection! {1, 8};
    assert_eq!(tested_set, desired_set);
}

fn is_prime<T: Into<i64>>(number: T) -> bool {
    let number = number.into();
    let float = number as f64;
    let s = float.sqrt().trunc() as i64;

    for d in 2..=s {
        if number % d == 0 {
            return false;
        }
    }

    true
}

#[test]
fn it_should_properly_create_maps() {
    let tested_map = btree! {
        1 => is_prime(1),
        2 => is_prime(2),
        3 => is_prime(3),
        4 => is_prime(4),
        5 => is_prime(5),
        6 => is_prime(6),
        7 => is_prime(7),
        8 => is_prime(8),
        9 => is_prime(9),
    };
    let desired_map: BTreeMap<i32, bool> = collection! {
        1 => is_prime(1),
        2 => is_prime(2),
        3 => is_prime(3),
        4 => is_prime(4),
        5 => is_prime(5),
        6 => is_prime(6),
        7 => is_prime(7),
        8 => is_prime(8),
        9 => is_prime(9),
    };
    assert_eq!(tested_map, desired_map);

    let tested_map = btree! {
        1 => true,
        1 => true,
        3 => true,
        3 => false,
        5 => true,
        5 => false,
        7 => true,
        7 => false,
        9 => false,
    };
    let desired_map: BTreeMap<u8, bool> = collection! {
        1 => true,
        3 => false,
        5 => false,
        7 => false,
        9 => false,
    };
    assert_eq!(tested_map, desired_map);
}

#[test]
fn it_should_create_the_same_sets_with_various_syntax() {
    let set1 = btree! { set of u8 };
    let set2 = btree!(set of u8);
    let set3 = btree![set of u8];

    let set4 = btree! { of u8 };
    let set5 = btree!(of u8);
    let set6 = btree![of u8];

    let sets = vec![set1, set2, set3, set4, set5, set6];

    for i in 0..sets.len() {
        for j in i..sets.len() {
            assert_eq!(sets[i], sets[j])
        }
    }
}
