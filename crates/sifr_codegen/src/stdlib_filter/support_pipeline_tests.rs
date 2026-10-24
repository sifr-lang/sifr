use super::{
    RustItemDeduper, absolutize_external_crate_paths, dedup_rust_items,
    seal_and_dedup_stdlib_support, seal_canonical_stdlib_names,
};
use std::collections::HashSet;

fn previous(
    source: &str,
    names: &HashSet<String>,
    deduper: &mut RustItemDeduper,
    skip: &HashSet<String>,
) -> String {
    let sealed = seal_canonical_stdlib_names(source, "sifr.pipeline", names);
    let absolute = absolutize_external_crate_paths(&sealed);
    dedup_rust_items(&absolute, deduper, skip)
}

#[test]
fn structured_support_preserves_names_macros_paths_and_cross_module_dedup() {
    let names = HashSet::from(["Widget".to_string()]);
    let skip = HashSet::from(["Skipped".to_string()]);
    let cases = [
        r#"pub struct Widget { pub value: i64 }
impl Widget { pub fn new() -> Self { Self { value: 3 } } }
fn make() -> Widget { Widget::new() }
fn runtime() -> sifr_runtime::Value { sifr_runtime::value() }
use sifr_stdlib::value as external_value;
struct Skipped;"#,
        r#"#[allow(dead_code)]
macro_rules! build { () => { Widget { value: 4 } }; }
fn qualified(value: foreign::Widget) -> foreign::Widget { value }
fn literal() -> &'static str { "Widget sifr_runtime::value" }
use sifr_runtime::{Value, value as runtime_value};"#,
        r#"struct Wrapper<T>(T);
impl<T> Wrapper<T> { fn take(self) -> T { self.0 } }
fn raw(r#type: Widget) -> Option<Widget> { Some(r#type) }
fn path() { let _ = ::sifr_runtime::value(); }"#,
    ];
    let mut old = RustItemDeduper::default();
    let mut new = RustItemDeduper::default();
    for source in cases.iter().chain(cases.iter()) {
        assert_eq!(
            seal_and_dedup_stdlib_support(source, "sifr.pipeline", &names, &mut new, &skip),
            previous(source, &names, &mut old, &skip),
        );
    }
}

#[test]
fn structured_support_preserves_conflicting_body_rejection() {
    let names = HashSet::new();
    let skip = HashSet::new();
    let mut old = RustItemDeduper::default();
    let mut new = RustItemDeduper::default();
    let first = "fn same() -> i64 { 1 }";
    assert_eq!(
        previous(first, &names, &mut old, &skip),
        seal_and_dedup_stdlib_support(first, "sifr.pipeline", &names, &mut new, &skip)
    );
    let changed = "fn same() -> i64 { 2 }";
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            previous(changed, &names, &mut old, &skip)
        }))
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            seal_and_dedup_stdlib_support(changed, "sifr.pipeline", &names, &mut new, &skip)
        }))
        .is_err()
    );
}

#[test]
fn structured_support_rejects_malformed_owned_source() {
    for source in [
        "fn missing( {",
        "struct Broken { field: }",
        "/* unterminated",
    ] {
        assert!(
            std::panic::catch_unwind(|| {
                previous(
                    source,
                    &HashSet::new(),
                    &mut RustItemDeduper::default(),
                    &HashSet::new(),
                )
            })
            .is_err()
        );
        assert!(
            std::panic::catch_unwind(|| {
                seal_and_dedup_stdlib_support(
                    source,
                    "sifr.pipeline",
                    &HashSet::new(),
                    &mut RustItemDeduper::default(),
                    &HashSet::new(),
                )
            })
            .is_err()
        );
    }
}
