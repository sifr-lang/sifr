# Collection Literals

<a href="https://crates.io/crates/collection_literals">
    <img src="https://img.shields.io/crates/v/collection_literals">
</a>

Simple library for convenient collection initialization.

## `collection!` macro

Macro for initializing collections of any type.

```rust
use collection_literals::collection;

fn main() {
    let empty_collection: HashMap<String, bool> = collection! {};
    let empty_collection = collection! { HashMap::<String, bool> };

    let hash_set: HashSet<u8> = collection! { 1, 5, 9, 12 };
    let hash_set = collection! { HashSet::<u8>; 1, 5, 9, 12 };

    let hash_map: HashMap<u64, char> = collection! {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
    };
    let hash_map = collection! { HashMap::<u64, char>;
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
    };

    let btree_set: BTreeSet<u8> = collection! { 1, 5, 9, 12 };
    let btree_set = collection! { BTreeSet::<u8>; 1, 5, 9, 12 };

    let btree_map: BTreeMap<u64, char> = collection! {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
    };
    let btree_map = collection! { BTreeMap::<u64, char>;
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
    };
}
```

## `hash!` macro

Macro for initializing both HashMaps and HashSets. It can infer both type of collection and types of entries, but you
can provide explicit type annotations.

```rust
use collection_literals::hash;

fn main() {
    let empty_hash_map: HashMap<String, bool> = hash! {};
    let empty_hash_map = hash! { map of String => bool };
    let empty_hash_map = hash! { map of String => bool {} };
    let empty_hash_map = hash! { of String => bool };
    let empty_hash_map = hash! { of String => bool {} };

    let empty_hash_set: HashSet<u8> = hash! {};
    let empty_hash_set = hash! { set of u8 };
    let empty_hash_set = hash! { set of u8 {} };
    let empty_hash_set = hash! { of u8 };
    let empty_hash_set = hash! { of u8 {} };

    let hash_map: HashMap<u64, char> = hash! {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
    };
    let hash_map = hash! { map of u64 => char {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
    }};
    let hash_map = hash! { of u64 => char {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
    }};
    let hash_map = hash! { 
        0_u64 => '0',
        1_u64 => '1',
        2_u64 => '2',
        3_u64 => '3',
    };

    let hash_set: HashSet<u8> = hash! { 1, 2, 3 };
    let hash_set = hash! { set of u8 { 1, 2, 3 } };
    let hash_set = hash! { of u8 { 1, 2, 3 } };
    let hash_set = hash! { 1_u8, 2_u8, 3_u8 };
}
```

## `btree!` macro:

Macro for initializing both BTreeMaps and BTreeSets. It can infer both type of collection and types of entries, but you
can provide explicit type annotations.

```rust
use collection_literals::btree;

fn main() {
    let empty_btree_map: BTreeMap<String, bool> = btree! {};
    let empty_btree_map = btree! { map of String => bool };
    let empty_btree_map = btree! { map of String => bool {} };
    let empty_btree_map = btree! { of String => bool };
    let empty_btree_map = btree! { of String => bool {} };

    let empty_btree_set: BTreeSet<u8> = hash! {};
    let empty_btree_set = btree! { set of u8 };
    let empty_btree_set = btree! { set of u8 {} };
    let empty_btree_set = btree! { of u8 };
    let empty_btree_set = btree! { of u8 {} };

    let btree_map: BTreeMap<u64, char> = btree! {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
    };
    let btree_map = btree! { map of u64 => char {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
    }};
    let btree_map = btree! { of u64 => char {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
    }};
    let btree_map = btree! { 
        0_u64 => '0',
        1_u64 => '1',
        2_u64 => '2',
        3_u64 => '3',
    };

    let btree_set: BTreeSet<u8> = btree! { 1, 2, 3 };
    let btree_set = btree! { set of u8 { 1, 2, 3 } };
    let btree_set = btree! { of u8 { 1, 2, 3 } };
    let btree_set = btree! { 1_u8, 2_u8, 3_u8 };
}
```
