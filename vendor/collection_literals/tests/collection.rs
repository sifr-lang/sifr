use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use collection_literals::collection;

#[test]
fn it_should_create_defaults() {
    let vector: Vec<u8> = collection![];
    assert_eq!(vector, Vec::<u8>::default());

    let hash_set: HashSet<u8> = collection! {};
    assert_eq!(hash_set, HashSet::<u8>::default());

    let btree_set: BTreeSet<u8> = collection! {};
    assert_eq!(btree_set, BTreeSet::<u8>::default());

    let hash_map: HashMap<u8, bool> = collection! {};
    assert_eq!(hash_map, HashMap::<u8, bool>::default());

    let btree_map: BTreeMap<u8, bool> = collection! {};
    assert_eq!(btree_map, BTreeMap::<u8, bool>::default());

    let vector = collection!(Vec<u8>);
    assert_eq!(vector, Vec::<u8>::default());

    let hash_set = collection!(HashSet<u8>);
    assert_eq!(hash_set, HashSet::<u8>::default());

    let btree_set = collection!(BTreeSet<u8>);
    assert_eq!(btree_set, BTreeSet::<u8>::default());

    let hash_map = collection!(HashMap<u8, bool>);
    assert_eq!(hash_map, HashMap::<u8, bool>::default());

    let btree_map = collection!(BTreeMap<u8, bool>);
    assert_eq!(btree_map, BTreeMap::<u8, bool>::default());
}

#[test]
fn it_should_properly_create_hash_sets() {
    let tested_set: HashSet<u8> = collection! {1, 2, 3, 4, 5, 6, 7, 8, 9};
    let desired_set = HashSet::<u8>::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(tested_set, desired_set);

    let tested_set: HashSet<u8> = collection! {1, 1, 1, 4, 4, 4, 8, 8, 8};
    let desired_set = HashSet::<u8>::from([1, 4, 8]);
    assert_eq!(tested_set, desired_set);
}

#[test]
fn it_should_properly_create_btree_sets() {
    let tested_set: BTreeSet<u8> = collection! {1, 2, 3, 4, 5, 6, 7, 8, 9};
    let desired_set = BTreeSet::<u8>::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(tested_set, desired_set);

    let tested_set: BTreeSet<u8> = collection! {1, 1, 1, 4, 4, 4, 8, 8, 8};
    let desired_set = BTreeSet::<u8>::from([1, 4, 8]);
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
fn it_should_properly_create_hash_maps() {
    let tested_map: HashMap<u8, bool> = collection! {
        1 => true,
        2 => true,
        3 => true,
        4 => false,
        5 => true,
        6 => false,
        7 => true,
        8 => false,
        9 => false,
    };
    let mut desired_map = HashMap::<u8, bool>::new();
    for i in 1..=9 {
        desired_map.insert(i, is_prime(i));
    }

    assert_eq!(tested_map, desired_map);

    let tested_map: HashMap<u8, bool> = collection! {
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
    let desired_map =
        HashMap::<u8, bool>::from([(1, true), (3, false), (5, false), (7, false), (9, false)]);

    assert_eq!(tested_map, desired_map);
}

#[test]
fn it_should_properly_create_btree_maps() {
    let tested_map: BTreeMap<u8, bool> = collection! {
        1 => true,
        2 => true,
        3 => true,
        4 => false,
        5 => true,
        6 => false,
        7 => true,
        8 => false,
        9 => false,
    };
    let mut desired_map = BTreeMap::<u8, bool>::new();
    for i in 1..=9 {
        desired_map.insert(i, is_prime(i));
    }

    assert_eq!(tested_map, desired_map);

    let tested_map: BTreeMap<u8, bool> = collection! {
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
    let desired_map =
        BTreeMap::<u8, bool>::from([(1, true), (3, false), (5, false), (7, false), (9, false)]);

    assert_eq!(tested_map, desired_map);
}
