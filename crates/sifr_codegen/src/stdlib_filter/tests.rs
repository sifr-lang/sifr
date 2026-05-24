use super::*;

#[test]
fn filter_keeps_transitive_dependencies_in_item_order() {
    let code = r#"
use std::collections::HashMap;

fn root() {
    helper();
}

fn helper() {
    leaf();
}

fn leaf() {}

fn unused() {}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported);

    assert!(filtered.contains("use std::collections::HashMap;"));
    assert!(filtered.contains("fn root()"));
    assert!(filtered.contains("fn helper()"));
    assert!(filtered.contains("fn leaf()"));
    assert!(!filtered.contains("fn unused()"));
}

#[test]
fn filter_ignores_name_mentions_in_strings_and_comments() {
    let code = r#"
fn root() {
    let _ = "helper()";
    // helper()
    /* helper() */
}

fn helper() {}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported);

    assert!(filtered.contains("fn root()"));
    assert!(!filtered.contains("fn helper()"));
}

#[test]
fn filter_tracks_type_level_dependencies_via_identifiers() {
    let code = r#"
struct Node {}

fn root() -> Node {
    Node {}
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported);

    assert!(filtered.contains("fn root()"));
    assert!(filtered.contains("struct Node {}"));
}

#[test]
fn filter_supports_enum_trait_static_and_pub_items() {
    let code = r#"
pub enum Mode {
    Fast,
}

pub trait Worker {
    fn run(&self) -> i64;
}

pub struct Job {}

impl Worker for Job {
    fn run(&self) -> i64 { JOB_COUNT }
}

pub static JOB_COUNT: i64 = 7;

pub fn root() -> Box<dyn Worker> {
    let _m = Mode::Fast;
    Box::new(Job {})
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported);

    assert!(filtered.contains("pub fn root()"));
    assert!(filtered.contains("pub enum Mode"));
    assert!(filtered.contains("pub trait Worker"));
    assert!(filtered.contains("pub struct Job"));
    assert!(filtered.contains("impl Worker for Job"));
    assert!(filtered.contains("pub static JOB_COUNT"));
}

#[test]
fn filter_supports_async_const_unsafe_fn_and_static_mut() {
    let code = r#"
pub static mut COUNTER: i64 = 0;

pub const fn seed() -> i64 {
    COUNTER
}

pub unsafe fn tick() -> i64 {
    COUNTER + seed()
}

pub async fn root() -> i64 {
    tick()
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported);

    assert!(filtered.contains("pub async fn root()"));
    assert!(filtered.contains("pub unsafe fn tick()"));
    assert!(filtered.contains("pub const fn seed()"));
    assert!(filtered.contains("pub static mut COUNTER"));
}

#[test]
fn filter_tracks_type_alias_dependencies_and_drops_unused_aliases() {
    let code = r#"
pub struct Node {}

pub type UsedAlias = Node;
pub type UnusedAlias = i64;

pub fn root() -> UsedAlias {
    Node {}
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported);

    assert!(filtered.contains("pub fn root() -> UsedAlias"));
    assert!(filtered.contains("pub type UsedAlias = Node;"));
    assert!(filtered.contains("pub struct Node {}"));
    assert!(!filtered.contains("pub type UnusedAlias = i64;"));
}

#[test]
fn filter_avoids_false_positive_from_local_variable_name() {
    let code = r#"
pub fn root() -> i64 {
    let helper = 1;
    helper + 1
}

pub fn helper() -> i64 {
    2
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported);
    assert!(filtered.contains("pub fn root()"));
    assert!(!filtered.contains("pub fn helper()"));
}

#[test]
fn filter_tracks_dependencies_used_in_macro_arguments() {
    let code = r#"
fn root() {
    helper();
}

fn helper() -> String {
    format!("v={}", leaf())
}

fn leaf() -> String {
    "ok".to_string()
}

fn unused() {}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported);

    assert!(filtered.contains("fn root()"));
    assert!(filtered.contains("fn helper()"));
    assert!(filtered.contains("fn leaf()"));
    assert!(!filtered.contains("fn unused()"));
}

#[test]
fn filter_keeps_all_items_for_needed_type_name() {
    let code = r#"
pub struct Builder {}

impl Builder {
    pub fn new() -> Builder {
        Builder {}
    }
}

pub fn root() -> Builder {
    Builder::new()
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported);

    assert!(filtered.contains("pub fn root() -> Builder"));
    assert!(filtered.contains("pub struct Builder {}"));
    assert!(filtered.contains("impl Builder"));
}

#[test]
fn dedup_uses_impl_signature_keys() {
    let code = r#"
struct Item {}

impl Item {
    fn a(&self) {}
}

impl Item {
    fn b(&self) {}
}

impl std::fmt::Display for Item {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }
}
"#;
    let mut emitted = HashSet::new();
    let skip_types = HashSet::new();
    let once = dedup_rust_items(code, &mut emitted, &skip_types);
    let twice = dedup_rust_items(code, &mut emitted, &skip_types);

    assert!(once.contains("struct Item {}"));
    assert!(once.contains("impl Item"));
    assert!(once.contains("impl std::fmt::Display for Item"));
    assert!(twice.trim().is_empty());
}

#[test]
fn collects_and_strips_shared_prelude_bits() {
    let input = r#"
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Mutex;

enum SifrFileHandle {
    Reader(std::fs::File),
}

static __SIFR_FILE_HANDLES: std::sync::OnceLock<
    Mutex<HashMap<i64, SifrFileHandle>>
> = std::sync::OnceLock::new();
static __SIFR_NEXT_FILE_HANDLE_ID: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);
fn __sifr_next_file_handle_id() -> i64 {
    __SIFR_NEXT_FILE_HANDLE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

struct FileHandle {
    _handle: i64,
}

fn keep_me() {
    let _ = __SIFR_FILE_HANDLES.get();
}
"#;
    let prepared = collect_and_strip_shared_prelude(input);
    assert!(prepared.shared_needs.collections.needs_hashmap);
    assert!(prepared.shared_needs.collections.needs_hashset);
    assert!(prepared.shared_needs.collections.needs_vecdeque);
    assert!(prepared.shared_needs.file_handles.needs_file_handles);
    assert!(
        prepared
            .shared_needs
            .file_handles
            .provides_file_handle_struct
    );
    assert!(!prepared
        .stripped_code
        .contains("use std::collections::HashMap;"));
    assert!(!prepared.stripped_code.contains("enum SifrFileHandle"));
    assert!(!prepared
        .stripped_code
        .contains("static __SIFR_FILE_HANDLES"));
    assert!(!prepared
        .stripped_code
        .contains("static __SIFR_NEXT_FILE_HANDLE_ID"));
    assert!(!prepared
        .stripped_code
        .contains("fn __sifr_next_file_handle_id"));
    assert!(prepared.stripped_code.contains("fn keep_me()"));
}

#[test]
fn shared_prelude_needs_ignore_comment_mentions() {
    let input = r#"
// HashMap HashSet VecDeque __SIFR_FILE_HANDLES
fn keep_me() {}
"#;
    let prepared = collect_and_strip_shared_prelude(input);
    assert!(!prepared.shared_needs.collections.needs_hashmap);
    assert!(!prepared.shared_needs.collections.needs_hashset);
    assert!(!prepared.shared_needs.collections.needs_vecdeque);
    assert!(!prepared.shared_needs.file_handles.needs_file_handles);
    assert!(prepared.stripped_code.contains("fn keep_me()"));
}

#[test]
fn strips_combined_shared_use_groups() {
    let input = r#"
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Mutex;
fn keep_me() {}
"#;
    let prepared = collect_and_strip_shared_prelude(input);
    assert!(!prepared
        .stripped_code
        .contains("std::collections::{HashMap, HashSet, VecDeque}"));
    assert!(!prepared.stripped_code.contains("use std::sync::Mutex;"));
    assert!(prepared.stripped_code.contains("fn keep_me()"));
}
