#![expect(dead_code, clippy::unwrap_used, reason = "This is a test module")]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::io::{BufReader, BufWriter};
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak as RcWeak};
use std::sync::{Arc, Mutex, OnceLock, RwLock, Weak as ArcWeak};

use get_size2::*;

mod feature;

#[derive(GetSize)]
pub struct TestStruct {
    value1: String,
    value2: u64,
}

#[test]
fn derive_struct() {
    let test = TestStruct {
        value1: "Hello".into(),
        value2: 123,
    };

    assert_eq!(test.get_heap_size(), 5);
}

#[derive(GetSize)]
pub struct TestStructGenerics<A, B> {
    value1: A,
    value2: B,
}

#[test]
fn derive_struct_with_generics() {
    let test: TestStructGenerics<String, u64> = TestStructGenerics {
        value1: "Hello".into(),
        value2: 123,
    };

    assert_eq!(test.get_heap_size(), 5);
}

#[derive(GetSize)]
#[get_size(ignore(B, C))]
struct TestStructGenericsIgnore<A, B, C> {
    value1: A,
    #[get_size(ignore)]
    value2: B,
    #[get_size(ignore)]
    value3: C,
}

struct TestStructNoGetSize {
    value: String,
}

#[test]
fn derive_struct_with_generics_and_ignore() {
    let no_impl = TestStructNoGetSize {
        value: "World!".into(),
    };

    let test: TestStructGenericsIgnore<String, u64, TestStructNoGetSize> =
        TestStructGenericsIgnore {
            value1: "Hello".into(),
            value2: 123,
            value3: no_impl,
        };

    assert_eq!(test.get_heap_size(), 5);
}

#[derive(GetSize)]
#[get_size(ignore(B, C))]
struct TestStructHelpers<A, B, C> {
    value1: A,
    #[get_size(size = 100)]
    value2: B,
    #[get_size(size_fn = get_size_helper)]
    value3: C,
}

const fn get_size_helper<C>(_value: &C) -> usize {
    50
}

#[test]
fn derive_struct_with_generics_and_helpers() {
    let no_impl = TestStructNoGetSize {
        value: "World!".into(),
    };

    let test: TestStructHelpers<String, u64, TestStructNoGetSize> = TestStructHelpers {
        value1: "Hello".into(),
        value2: 123,
        value3: no_impl,
    };

    assert_eq!(test.get_heap_size(), 5 + 100 + 50);
}

#[derive(GetSize)]
pub struct TestStructGenericsLifetimes<'a, A, B> {
    value1: A,
    value2: &'a B,
}

#[test]
fn derive_struct_with_generics_and_lifetimes() {
    let value = 123u64;

    let test: TestStructGenericsLifetimes<'_, String, u64> = TestStructGenericsLifetimes {
        value1: "Hello".into(),
        value2: &value,
    };

    assert_eq!(test.get_heap_size(), 5);
}

#[derive(GetSize)]
pub enum TestEnum {
    Variant1(u8, u16, u32),
    Variant2(String),
    Variant3(i64, Vec<u16>),
    Variant4(String, i32, Vec<u32>, bool, &'static str),
    Variant5(f64, TestStruct),
    Variant6,
    Variant7 { x: String, y: String },
}

#[test]
fn derive_enum() {
    let test = TestEnum::Variant1(1, 2, 3);
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum::Variant2("Hello".into());
    assert_eq!(test.get_heap_size(), 5);

    let test = TestEnum::Variant3(-12, vec![1, 2, 3]);
    assert_eq!(test.get_heap_size(), 6);

    let s: String = "Test".into();
    assert_eq!(s.get_heap_size(), 4);
    let v = vec![1, 2, 3, 4];
    assert_eq!(v.get_heap_size(), 16);
    let test = TestEnum::Variant4(s, -123, v, false, "Hello world!");
    assert_eq!(test.get_heap_size(), 4 + 16);

    let test_struct = TestStruct {
        value1: "Hello world".into(),
        value2: 123,
    };

    let test = TestEnum::Variant5(12.34, test_struct);
    assert_eq!(test.get_heap_size(), 11);

    let test = TestEnum::Variant6;
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum::Variant7 {
        x: "Hello".into(),
        y: "world".into(),
    };
    assert_eq!(test.get_heap_size(), 5 + 5);
}

#[derive(GetSize)]
pub enum TestEnumGenerics<'a, A, B, C> {
    Variant1(A),
    Variant2(B),
    Variant3(&'a C),
}

#[test]
fn derive_enum_generics() {
    let test: TestEnumGenerics<'_, u64, String, TestStruct> = TestEnumGenerics::Variant1(123);
    assert_eq!(test.get_heap_size(), 0);

    let test: TestEnumGenerics<'_, u64, String, TestStruct> =
        TestEnumGenerics::Variant2("Hello".into());
    assert_eq!(test.get_heap_size(), 5);

    let test_struct = TestStruct {
        value1: "Hello world".into(),
        value2: 123,
    };

    let test: TestEnumGenerics<'_, u64, String, TestStruct> =
        TestEnumGenerics::Variant3(&test_struct);
    assert_eq!(test.get_heap_size(), 0);
}

const MINIMAL_NODE_SIZE: usize = 3;

#[derive(Clone, GetSize)]
enum Node<T>
where
    T: Default,
{
    Block(T),
    Blocks(Box<[T; MINIMAL_NODE_SIZE * MINIMAL_NODE_SIZE * MINIMAL_NODE_SIZE]>),
    Nodes(Box<[Self; 8]>),
}

#[test]
fn derive_enum_generics_issue1() {
    let test: Node<String> = Node::Block("test".into());
    assert_eq!(test.get_heap_size(), 4);

    let test: Node<u64> = Node::Blocks(Box::new([123; 27]));
    assert_eq!(test.get_heap_size(), 8 * 27);

    let t1: Node<u64> = Node::Block(123);
    let t2 = t1.clone();
    let t3 = t1.clone();
    let t4 = t1.clone();
    let t5 = t1.clone();
    let t6 = t1.clone();
    let t7 = t1.clone();
    let t8 = t1.clone();
    let test: Node<u64> = Node::Nodes(Box::new([t1, t2, t3, t4, t5, t6, t7, t8]));
    assert_eq!(test.get_heap_size(), 8 * std::mem::size_of::<Node<u64>>());
}

#[derive(GetSize)]
pub enum TestEnum2 {
    Zero = 0,
    One = 1,
    Two = 2,
}

#[test]
fn derive_enum_c_style() {
    assert_eq!(TestEnum2::Zero.get_heap_size(), 0);
    assert_eq!(TestEnum2::One.get_heap_size(), 0);
    assert_eq!(TestEnum2::Two.get_heap_size(), 0);
}

#[derive(GetSize)]
pub struct TestNewType(u64);

#[test]
fn derive_newtype() {
    let test = TestNewType(0);
    assert_eq!(u64::get_stack_size(), test.get_size());
}

#[test]
fn tracker() {
    #[derive(GetSize, Clone)]
    struct RcWrapper(Rc<i32>);

    let shared = RcWrapper(Rc::new(5));

    let (size, _) =
        (shared.clone(), shared.clone()).get_heap_size_with_tracker(StandardTracker::new());
    assert_eq!(size, shared.get_heap_size());

    let vec = vec![shared.clone(); 100];
    let (size, _) = vec.get_heap_size_with_tracker(StandardTracker::new());
    assert_eq!(
        size,
        (std::mem::size_of::<Rc<i32>>() * 100) + shared.get_heap_size()
    );
}

#[test]
fn boxed_slice() {
    let boxed = vec![1u8; 10].into_boxed_slice();
    assert_eq!(boxed.get_heap_size(), size_of::<u8>() * boxed.len());

    let boxed = vec![1u32; 10].into_boxed_slice();
    assert_eq!(boxed.get_heap_size(), size_of::<u32>() * boxed.len());

    let boxed = vec![&1u8; 10].into_boxed_slice();
    assert_eq!(boxed.get_heap_size(), size_of::<&u8>() * boxed.len());

    let rc = Rc::<[u8]>::from([1u8; 10]);
    assert_eq!(rc.get_heap_size(), size_of::<u8>() * rc.len());

    let arc = Arc::<[u8]>::from([1u8; 10]);
    assert_eq!(arc.get_heap_size(), size_of::<u8>() * arc.len());
}

#[test]
fn boxed_str() {
    let boxed: Box<str> = "a".to_owned().into();
    assert_eq!(boxed.get_heap_size(), size_of::<u8>() * boxed.len());

    let rc: Rc<str> = "a".to_owned().into();
    assert_eq!(rc.get_heap_size(), size_of::<u8>() * boxed.len());

    let arc: Arc<str> = "a".to_owned().into();
    assert_eq!(arc.get_heap_size(), size_of::<u8>() * boxed.len());
}

#[test]
fn cow() {
    use std::borrow::Cow;

    let cow: Cow<'_, str> = Cow::Borrowed("Hello world");
    assert_eq!(cow.get_heap_size(), 0);

    let cow: Cow<'_, str> = Cow::Owned("Hello world".into());
    assert_eq!(cow.get_heap_size(), 11);
}

#[test]
fn once_lock() {
    let lock: OnceLock<String> = OnceLock::new();
    assert_eq!(lock.get_heap_size(), 0);

    let lock_filled: OnceLock<String> = {
        let l = OnceLock::new();
        l.set(String::from("HalloTest")).unwrap();
        l
    };
    assert_eq!(
        lock_filled.get_heap_size(),
        lock_filled.get().unwrap().capacity()
    );
}

#[test]
fn test_enum() {
    #[derive(GetSize)]
    enum Enum {
        A {
            #[get_size(ignore)]
            b: B,
        },
    }

    struct B;
}

#[test]
fn test_ignore_attribute_on_enum_field() {
    #[derive(GetSize)]
    enum WithIgnore {
        A {
            #[get_size(ignore)]
            data: Vec<u8>,
        },
    }

    #[derive(GetSize)]
    enum WithoutIgnore {
        A { data: Vec<u8> },
    }

    let heap_vec = vec![0u8; 100];
    let with = WithIgnore::A {
        data: heap_vec.clone(),
    };
    let without = WithoutIgnore::A { data: heap_vec };

    let size_with_ignore = with.get_heap_size();
    let size_without_ignore = without.get_heap_size();

    assert!(size_with_ignore < size_without_ignore);
    let expected_size = size_without_ignore - size_with_ignore;
    assert!(
        expected_size >= 100,
        "Expected heap size contribution from Vec<u8> to be at least 100"
    );
}

#[test]
fn refcell() {
    let cell = RefCell::new(42u32);
    assert_eq!(cell.get_heap_size(), 0);
    assert_eq!(cell.get_size(), size_of::<RefCell<u32>>());

    let cell = RefCell::new(String::from("Hello, World!"));
    assert_eq!(cell.get_heap_size(), 13);
    assert_eq!(cell.get_size(), size_of::<RefCell<String>>() + 13);

    let cell = RefCell::new(String::new());
    assert_eq!(cell.get_heap_size(), 0);

    let vec_data = vec![1u32, 2, 3, 4, 5];
    let expected_heap_size = vec_data.capacity() * size_of::<u32>();
    let cell = RefCell::new(vec_data);
    assert_eq!(cell.get_heap_size(), expected_heap_size);
    assert_eq!(
        cell.get_size(),
        size_of::<RefCell<Vec<u32>>>() + expected_heap_size
    );

    let inner = RefCell::new(String::from("nested"));
    let outer = RefCell::new(inner);
    assert_eq!(outer.get_heap_size(), 6);

    let cell = RefCell::new(String::from("borrowed"));
    {
        let _borrowed = cell.borrow();
        assert_eq!(cell.get_heap_size(), 8);
    }
    assert_eq!(cell.get_heap_size(), 8);

    let cell = RefCell::new(String::from("mutable"));
    {
        let _borrowed = cell.borrow_mut();
    }
    assert_eq!(cell.get_heap_size(), 7);

    let boxed = Box::new(vec![1u32, 2, 3]);
    let cell = RefCell::new(boxed);
    let expected_heap_size = size_of::<Vec<u32>>() + 3 * size_of::<u32>();
    assert_eq!(cell.get_heap_size(), expected_heap_size);

    let cell = RefCell::new(String::from("tracker"));
    let (heap_size, _tracker) = cell.get_heap_size_with_tracker(StandardTracker::new());
    assert_eq!(heap_size, 7);
    assert_eq!(heap_size, cell.get_heap_size());

    let cell = RefCell::new(());
    assert_eq!(cell.get_heap_size(), 0);
    assert_eq!(cell.get_size(), size_of::<RefCell<()>>());
}

#[test]
fn covers_heap_size_function() {
    let value = String::from("heap");
    assert_eq!(heap_size(&value), value.get_heap_size());
}

#[test]
fn covers_ranges() {
    let range = String::from("ab")..String::from("cde");
    let range_expected = range.start.get_heap_size() + range.end.get_heap_size();
    assert_eq!(range.get_heap_size(), range_expected);

    let from = (String::from("abcdef"))..;
    assert_eq!(from.get_heap_size(), from.start.get_heap_size());

    let to = ..String::from("xyz");
    assert_eq!(to.get_heap_size(), to.end.get_heap_size());

    let to_inclusive = ..=String::from("xyz");
    assert_eq!(
        to_inclusive.get_heap_size(),
        to_inclusive.end.get_heap_size()
    );

    let full = ..;
    assert_eq!(full.get_heap_size(), 0);

    let inclusive = String::from("left")..=String::from("right");
    let inclusive_expected = inclusive.start().get_heap_size() + inclusive.end().get_heap_size();
    assert_eq!(inclusive.get_heap_size(), inclusive_expected);
}

#[test]
fn covers_collections_remaining_branches() {
    let mut btree_set = BTreeSet::new();
    btree_set.insert(String::from("set"));
    assert!(btree_set.get_heap_size() >= btree_set.iter().map(GetSize::get_size).sum::<usize>());

    let mut linked = LinkedList::new();
    linked.push_back(String::from("linked"));
    assert!(linked.get_heap_size() >= linked.iter().map(GetSize::get_size).sum::<usize>());

    let mut btree_map = BTreeMap::new();
    btree_map.insert(String::from("k"), String::from("value"));
    assert!(btree_map.get_heap_size() >= 6);

    let mut hash_map = HashMap::new();
    hash_map.insert(String::from("k"), String::from("value"));
    assert!(hash_map.get_heap_size() >= std::mem::size_of::<(String, String)>());

    let mut hash_set = HashSet::new();
    hash_set.insert(String::from("value"));
    assert!(hash_set.get_heap_size() >= std::mem::size_of::<String>());

    let mut heap = BinaryHeap::new();
    heap.push(String::from("value"));
    assert!(heap.get_heap_size() >= std::mem::size_of::<String>());

    let mut deque = VecDeque::new();
    deque.push_back(String::from("value"));
    assert!(deque.get_heap_size() >= std::mem::size_of::<String>());
}

#[test]
fn covers_ownership_remaining_branches() {
    let arc = Arc::new(String::from("arc"));
    let pair = (Arc::clone(&arc), Arc::clone(&arc));
    let (tracked_size, _) = pair.get_heap_size_with_tracker(StandardTracker::new());
    assert_eq!(tracked_size, arc.as_ref().get_size());

    let none_value: Option<String> = None;
    assert_eq!(none_value.get_heap_size(), 0);

    let err_value: Result<u32, String> = Err(String::from("err"));
    assert!(err_value.get_heap_size() >= 3);

    let rc = Rc::new(String::from("weak"));
    let weak: RcWeak<String> = Rc::downgrade(&rc);
    assert_eq!(weak.get_heap_size(), 0);

    let arc2 = Arc::new(String::from("weak"));
    let weak2: ArcWeak<String> = Arc::downgrade(&arc2);
    assert_eq!(weak2.get_heap_size(), 0);
}

#[test]
fn covers_std_types_remaining_branches() {
    let c_string = match CString::new("abc") {
        Ok(v) => v,
        Err(err) => panic!("CString::new failed: {err}"),
    };
    assert_eq!(c_string.get_heap_size(), 4);

    let c_str: &CStr = c_string.as_c_str();
    assert_eq!(c_str.get_heap_size(), 4);

    let os_string = OsString::from("hello");
    assert_eq!(os_string.get_heap_size(), os_string.len());

    let os_str: &OsStr = os_string.as_os_str();
    assert_eq!(os_str.get_heap_size(), os_str.len());

    let path_buf = PathBuf::from("folder/file.txt");
    assert!(path_buf.get_heap_size() >= path_buf.as_os_str().len());

    let path: &Path = path_buf.as_path();
    assert_eq!(path.get_heap_size(), 0);

    let reader = BufReader::with_capacity(32, &b"input"[..]);
    assert!(reader.get_heap_size() >= 32);

    let writer = BufWriter::with_capacity(16, Vec::<u8>::new());
    assert!(writer.get_heap_size() >= 16);
}

#[test]
fn covers_sync_impls_and_tracker_paths() {
    let cell = RefCell::new(String::from("mutably-borrowed"));
    let borrowed = cell.borrow_mut();
    assert_eq!(cell.get_heap_size(), 0);
    drop(borrowed);
    assert!(cell.get_heap_size() >= "mutably-borrowed".len());

    let empty_lock: OnceLock<String> = OnceLock::new();
    assert_eq!(empty_lock.get_heap_size(), 0);
    let full_lock = OnceLock::from(String::from("set"));
    assert!(full_lock.get_heap_size() >= 3);

    let mutex = Mutex::new(String::from("poisoned"));
    std::thread::scope(|scope| {
        let handle = scope.spawn(|| {
            let _guard = match mutex.lock() {
                Ok(guard) => guard,
                Err(err) => err.into_inner(),
            };
            panic!("intentional poison");
        });
        assert!(handle.join().is_err());
    });
    assert!(mutex.get_heap_size() >= 8);

    let rwlock = RwLock::new(String::from("poisoned"));
    std::thread::scope(|scope| {
        let handle = scope.spawn(|| {
            let _guard = match rwlock.write() {
                Ok(guard) => guard,
                Err(err) => err.into_inner(),
            };
            panic!("intentional poison");
        });
        assert!(handle.join().is_err());
    });
    assert!(rwlock.get_heap_size() >= 8);
}

#[test]
fn covers_tracker_trait_forwarders() {
    let value = 123_u64;
    let addr = &raw const value;

    let mut base = StandardTracker::new();
    let mut by_ref = &mut base;
    assert!(GetSizeTracker::track(&mut by_ref, addr));
    assert!(!GetSizeTracker::track(&mut by_ref, addr));

    let mut boxed = Box::new(StandardTracker::new());
    assert!(GetSizeTracker::track(&mut boxed, addr));
    assert!(!GetSizeTracker::track(&mut boxed, addr));

    let mut locked = Mutex::new(StandardTracker::new());
    assert!(GetSizeTracker::track(&mut locked, addr));
    assert!(!GetSizeTracker::track(&mut locked, addr));

    let mut rw_locked = RwLock::new(StandardTracker::new());
    assert!(GetSizeTracker::track(&mut rw_locked, addr));
    assert!(!GetSizeTracker::track(&mut rw_locked, addr));

    let mut arc_mutex = Arc::new(Mutex::new(StandardTracker::new()));
    assert!(GetSizeTracker::track(&mut arc_mutex, addr));
    assert!(!GetSizeTracker::track(&mut arc_mutex, addr));

    let mut arc_rwlock = Arc::new(RwLock::new(StandardTracker::new()));
    assert!(GetSizeTracker::track(&mut arc_rwlock, addr));
    assert!(!GetSizeTracker::track(&mut arc_rwlock, addr));

    let mut tracker = StandardTracker::new();
    assert!(tracker.track(addr));
    tracker.clear();
    assert!(tracker.track(addr));

    let mut no_tracker = NoTracker::new(false);
    assert!(!no_tracker.answer());
    assert!(!no_tracker.track(addr));
    no_tracker.set_answer(true);
    assert!(no_tracker.answer());
    assert!(no_tracker.track(addr));
}
