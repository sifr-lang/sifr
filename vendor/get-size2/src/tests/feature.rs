use std::mem::size_of;

use get_size2::*;

#[test]
fn chrono() {
    use chrono::TimeZone;

    let timedelta = chrono::TimeDelta::seconds(5);
    assert_eq!(timedelta.get_heap_size(), 0);

    let datetime = chrono::Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 11).unwrap();
    assert_eq!(datetime.naive_utc().get_heap_size(), 0);
    assert_eq!(datetime.naive_utc().date().get_heap_size(), 0);
    assert_eq!(datetime.naive_utc().time().get_heap_size(), 0);
    assert_eq!(datetime.timezone().get_heap_size(), 0);
    assert_eq!(datetime.fixed_offset().timezone().get_heap_size(), 0);
    assert_eq!(datetime.get_heap_size(), 0);
}

#[test]
fn chrono_tz() {
    use chrono::TimeZone;

    let datetime = chrono_tz::UTC
        .with_ymd_and_hms(2014, 7, 8, 9, 10, 11)
        .unwrap();
    assert_eq!(datetime.offset().get_heap_size(), 0);
}

#[test]
fn url() {
    const URL_STR: &str = "https://example.com/path?a=b&c=d";

    let url = url::Url::parse(URL_STR).unwrap();
    assert_eq!(url.get_heap_size(), URL_STR.len());
}

#[test]
fn bytes() {
    const BYTES_STR: &str = "Hello world";

    let bytes = bytes::Bytes::from(BYTES_STR);
    assert_eq!(bytes.get_heap_size(), BYTES_STR.len());

    let mut bytes_mut = bytes::BytesMut::from(BYTES_STR);
    assert_eq!(bytes_mut.get_heap_size(), BYTES_STR.len());
    bytes_mut.truncate(0);
    assert_eq!(bytes_mut.get_heap_size(), 0);
}

#[test]
fn compact_str() {
    const STR: &str = "Hello world";
    const LONG_STR: &str = "A much looooonger string that exceeds 24 bytes.";

    let value = compact_str::CompactString::from(STR);
    assert_eq!(value.get_heap_size(), 0);

    let mut value = compact_str::CompactString::from(LONG_STR);
    assert_eq!(value.get_heap_size(), value.capacity());

    value.shrink_to_fit();

    assert_eq!(value.len(), value.capacity());
    assert_eq!(value.get_heap_size(), LONG_STR.len());
}

#[test]
fn hashbrown() {
    use std::hash::{BuildHasher, RandomState};

    const VALUE_STR: &str = "A very looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooonng string.";

    let hasher = RandomState::new();

    let mut map = hashbrown::HashTable::new();
    assert_eq!(map.get_heap_size(), 0);
    map.insert_unique(
        hasher.hash_one(VALUE_STR),
        String::from(VALUE_STR),
        |value| hasher.hash_one(value),
    );
    assert!(map.get_heap_size() >= size_of::<String>() + VALUE_STR.len());

    let mut map = hashbrown::HashMap::<i32, String, RandomState>::default();
    assert_eq!(map.get_heap_size(), 0);
    map.insert(0, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(i32, String)>() + VALUE_STR.len());

    let mut set = hashbrown::HashSet::<String, RandomState>::default();
    assert_eq!(set.get_heap_size(), 0);
    set.insert(String::from(VALUE_STR));
    assert!(set.get_heap_size() >= size_of::<String>() + VALUE_STR.len());
}

#[test]
fn smallvec() {
    const ITEM_STR: &str = "Hello world";
    let mut vec = smallvec::SmallVec::<[String; 2]>::from([String::new(), String::from(ITEM_STR)]);

    assert_eq!(vec.get_heap_size(), ITEM_STR.len());
    vec.push(String::new());

    assert_eq!(
        vec.get_heap_size(),
        ITEM_STR.len() + std::mem::size_of::<String>() * vec.capacity()
    );

    vec.shrink_to_fit();

    assert_eq!(
        vec.get_heap_size(),
        ITEM_STR.len() + std::mem::size_of::<String>() * 3
    );
}

#[test]
fn thin_vec() {
    const ITEM_STR: &str = "Hello world";

    assert_eq!(thin_vec::ThinVec::<String>::default().get_heap_size(), 0);

    let mut vec = thin_vec::ThinVec::<String>::from([String::new(), String::from(ITEM_STR)]);
    assert_eq!(
        vec.get_heap_size(),
        ITEM_STR.len()
            + std::mem::size_of::<String>() * vec.capacity()
            + std::mem::size_of::<usize>() * 2
    );

    vec.shrink_to_fit();

    assert_eq!(
        vec.get_heap_size(),
        ITEM_STR.len()
            + std::mem::size_of::<String>() * vec.len()
            + std::mem::size_of::<usize>() * 2
    );
}

#[test]
fn test_indexmap() {
    use std::hash::RandomState;

    const VALUE_STR: &str = "A very looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooonng string.";

    let hasher = RandomState::new();

    let mut map = indexmap::IndexMap::with_capacity_and_hasher(1, hasher);
    assert_eq!(map.get_heap_size(), 40);
    map.insert(VALUE_STR, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(&'static str, String)>() + VALUE_STR.len());

    let mut map = indexmap::IndexMap::<i32, String, RandomState>::default();
    assert_eq!(map.get_heap_size(), 0);
    map.insert(0, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(i32, String)>() + VALUE_STR.len());

    let mut set = indexmap::IndexSet::<String, RandomState>::default();
    assert_eq!(set.get_heap_size(), 0);
    set.insert(String::from(VALUE_STR));
    assert!(set.get_heap_size() >= size_of::<String>() + VALUE_STR.len());
}

#[test]
fn test_ordermap() {
    use std::hash::RandomState;

    const VALUE_STR: &str = "A very looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooonng string.";

    let hasher = RandomState::new();

    let mut map = ordermap::OrderMap::with_capacity_and_hasher(1, hasher);
    assert_eq!(map.get_heap_size(), 40);
    map.insert(VALUE_STR, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(&'static str, String)>() + VALUE_STR.len());

    let mut map = ordermap::OrderMap::<i32, String, RandomState>::default();
    assert_eq!(map.get_heap_size(), 0);
    map.insert(0, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(i32, String)>() + VALUE_STR.len());

    let mut set = ordermap::OrderSet::<String, RandomState>::default();
    assert_eq!(set.get_heap_size(), 0);
    set.insert(String::from(VALUE_STR));
    assert!(set.get_heap_size() >= size_of::<String>() + VALUE_STR.len());
}
