use std::collections::{HashMap, HashSet};

use collection_literals::{collection, hash};

#[test]
fn it_should_create_defaults() {
    let set0: HashSet<u8> = hash! {};
    let set1 = hash! { set of u8 };
    let set2 = hash! { set of u8 {} };
    let set3 = hash! { of u8 };
    let set4 = hash! { of u8 {} };

    let desired_set: HashSet<u8> = collection! {};

    let sets = vec![set0, set1, set2, set3, set4, desired_set];
    for i in 0..sets.len() {
        for j in i..sets.len() {
            assert_eq!(sets[i], sets[j])
        }
    }

    let map0: HashMap<u8, bool> = hash! {};
    let map1 = hash! { map of u8 => bool };
    let map2 = hash! { map of u8 => bool {} };
    let map3 = hash! { of u8 => bool };
    let map4 = hash! { of u8 => bool {} };

    let desired_map: HashMap<u8, bool> = collection! {};

    let maps = vec![map0, map1, map2, map3, map4, desired_map];
    for i in 0..maps.len() {
        for j in i..maps.len() {
            assert_eq!(maps[i], maps[j])
        }
    }
}

#[test]
fn it_should_properly_create_sets() {
    let set0 = hash! {1, 2, 3, 4, 5, 6, 7, 8, 9};
    let set1 = hash! { set of i32 {1, 2, 3, 4, 5, 6, 7, 8, 9} };
    let set2 = hash! { of i32 {1, 2, 3, 4, 5, 6, 7, 8, 9} };
    let desired_set: HashSet<i32> = collection! {1, 2, 3, 4, 5, 6, 7, 8, 9};

    let sets = vec![set0, set1, set2, desired_set];
    for i in 0..sets.len() {
        for j in i..sets.len() {
            assert_eq!(sets[i], sets[j])
        }
    }

    let set0 = hash! {1, 1, 1, 8, 8, 8};
    let set1 = hash! { set of i32 {1, 1, 1, 8, 8, 8} };
    let set2 = hash! { of i32 {1, 1, 1, 8, 8, 8} };
    let desired_set: HashSet<i32> = collection! {1, 8};

    let sets = vec![set0, set1, set2, desired_set];
    for i in 0..sets.len() {
        for j in i..sets.len() {
            assert_eq!(sets[i], sets[j])
        }
    }
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
    let map0 = hash! {
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
    let map1 = hash! { map of i32 => bool {
        1 => is_prime(1),
        2 => is_prime(2),
        3 => is_prime(3),
        4 => is_prime(4),
        5 => is_prime(5),
        6 => is_prime(6),
        7 => is_prime(7),
        8 => is_prime(8),
        9 => is_prime(9),
    }};
    let map2 = hash! { of i32 => bool {
        1 => is_prime(1),
        2 => is_prime(2),
        3 => is_prime(3),
        4 => is_prime(4),
        5 => is_prime(5),
        6 => is_prime(6),
        7 => is_prime(7),
        8 => is_prime(8),
        9 => is_prime(9),
    }};
    let desired_map: HashMap<i32, bool> = collection! {
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

    let maps = vec![map0, map1, map2, desired_map];
    for i in 0..maps.len() {
        for j in i..maps.len() {
            assert_eq!(maps[i], maps[j])
        }
    }

    let map0 = hash! {
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
    let map1 = hash! { map of i32 => bool {
        1 => true,
        1 => true,
        3 => true,
        3 => false,
        5 => true,
        5 => false,
        7 => true,
        7 => false,
        9 => false,
    }};
    let map2 = hash! { of i32 => bool {
        1 => true,
        1 => true,
        3 => true,
        3 => false,
        5 => true,
        5 => false,
        7 => true,
        7 => false,
        9 => false,
    }};
    let desired_map: HashMap<i32, bool> = collection! {
        1 => true,
        3 => false,
        5 => false,
        7 => false,
        9 => false,
    };

    let maps = vec![map0, map1, map2, desired_map];
    for i in 0..maps.len() {
        for j in i..maps.len() {
            assert_eq!(maps[i], maps[j])
        }
    }
}
