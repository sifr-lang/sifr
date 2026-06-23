/// Macro for initializing collections of any type.
///
/// The collection must implement [From] array of values or key-value pairs.
/// [From] [IntoIterator] will also work.
/// You must specify type of collection.
/// ```rust
/// use std::collections::LinkedList;
/// use collection_literals::collection;
///
/// let linked_list: LinkedList<String> = collection! { "Hello".to_string(), "Hallo".to_string() };
/// assert_eq!(linked_list, LinkedList::from(["Hello".to_string(), "Hallo".to_string()]));
///
/// let linked_list = collection! { LinkedList::<&str>; "Bonjour", "Здравствуй" };
/// assert_eq!(linked_list, LinkedList::from(["Bonjour", "Здравствуй"]));
/// ```
#[macro_export]
macro_rules! collection {
    ($collection_type:ty) => {{
        <$collection_type>::new()
    }};

    // map-like
    ($collection_type:ty; $($key:expr => $value:expr),* $(,)?) => {{
        let temp: $collection_type = core::convert::From::from([$(($key, $value),)*]);
        temp
    }};
    ($($key:expr => $value:expr),* $(,)?) => {{
        core::convert::From::from([$(($key, $value),)*])
    }};

    // set-like
    ($collection_type:ty; $($value:expr),* $(,)?) => {{
        let temp: $collection_type = core::convert::From::from([$($value,)*]);
        temp
    }};
    ($($value:expr),* $(,)?) => {{
        core::convert::From::from([$($value,)*])
    }};
}

/// Macro for initializing both HashMaps and HashSets.
/// It can infer both type of collection and types of entries but you can provide explicit type annotations.
/// ```rust
/// use std::collections::{HashMap, HashSet};
/// use collection_literals::hash;
///
/// let set = hash! { set of &str { "Hi", "Hoi" } };
/// let map = hash! { map of u8 => char {
///     0 => '0',
///     1 => '1',
///     2 => '2',
/// }};
///
/// assert_eq!(set, HashSet::from(["Hi", "Hoi"]));
/// assert_eq!(map, HashMap::from([(0, '0'), (1, '1'), (2, '2')]));
///
///
/// assert_eq!(hash! { 88, 99 }, hash! { set of i32 { 88, 99 } });
/// assert_eq!(hash! { 88 => 99 }, hash! { map of i32 => i32 { 88 => 99 } });
///```
#[macro_export]
macro_rules! hash {
    () => {{ collection!{} }};

    // map-like
    ($(map)? of $key_type:ty => $value_type:ty) => {{
        std::collections::HashMap::<$key_type, $value_type>::new()
    }};
    ($($key:expr => $value:expr),* $(,)?) => {{
        std::collections::HashMap::from([$(($key, $value),)*])
    }};
    ($(map)? of $key_type:ty => $value_type:ty { $($key:expr => $value:expr),* $(,)? }) => {{
        std::collections::HashMap::<$key_type, $value_type>::from([$(($key, $value),)*])
    }};

    // set-like
    ($(set)? of $value_type:ty) => {{
        std::collections::HashSet::<$value_type>::new()
    }};
    ($($value:expr),* $(,)?) => {{
        std::collections::HashSet::from([$($value,)*])
    }};
    ($(set)? of $value_type:ty { $($value:expr),* $(,)? }) => {{
        std::collections::HashSet::<$value_type>::from([$($value,)*])
    }};
}

/// Macro for initializing both BTreeMaps and BTreeSets.
/// It can infer both type of collection and types of entries but you can provide explicit type annotations.
/// ```rust
/// use std::collections::{BTreeMap, BTreeSet};
/// use collection_literals::btree;
///
/// let set = btree! { set of &str { "Hi", "Hoi" } };
/// let map = btree! { map of u8 => char {
///     0 => '0',
///     1 => '1',
///     2 => '2',
/// }};
///
/// assert_eq!(set, BTreeSet::from(["Hi", "Hoi"]));
/// assert_eq!(map, BTreeMap::from([(0, '0'), (1, '1'), (2, '2')]));
///
///
/// assert_eq!(btree! { 88, 99 }, btree! { set of i32 { 88, 99 } });
/// assert_eq!(btree! { 88 => 99 }, btree! { map of i32 => i32 { 88 => 99 } });
///```
#[macro_export]
macro_rules! btree {
    () => {{ collection!{} }};

    // map-like
    ($(map)? of $key_type:ty => $value_type:ty) => {{
        std::collections::BTreeMap::<$key_type, $value_type>::new()
    }};
    ($($key:expr => $value:expr),* $(,)?) => {{
        std::collections::BTreeMap::from([$(($key, $value),)*])
    }};
    ($(map)? of $key_type:ty => $value_type:ty { $($key:expr => $value:expr),* $(,)? }) => {{
        std::collections::BTreeMap::<$key_type, $value_type>::from([$(($key, $value),)*])
    }};

    // set-like
    ($(set)? of $value_type:ty) => {{
        std::collections::BTreeSet::<$value_type>::new()
    }};
    ($($value:expr),* $(,)?) => {{
        std::collections::BTreeSet::from([$($value,)*])
    }};
    ($(set)? of $value_type:ty { $($value:expr),* $(,)? }) => {{
        std::collections::BTreeSet::<$value_type>::from([$($value,)*])
    }};
}
