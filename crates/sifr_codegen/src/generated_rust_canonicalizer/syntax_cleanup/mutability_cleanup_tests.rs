use crate::canonicalize_generated_rust_source;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

fn canonical_and_compile(source: &str) -> String {
    let canonical = canonicalize_generated_rust_source(source).expect("canonical Rust");
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "sifr-option-mutability-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory).expect("unique retained native evidence");
    let mut child = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--emit=metadata",
            "-Dunused_mut",
            "--out-dir",
        ])
        .arg(&directory)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rustc");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(canonical.as_bytes())
        .expect("source");
    let output = child.wait_with_output().expect("compiler output");
    assert!(
        output.status.success(),
        "{}\n{canonical}",
        String::from_utf8_lossy(&output.stderr)
    );
    canonical
}

#[test]
fn option_mutable_receiver_family_preserves_indexed_and_direct_places() {
    let canonical = canonical_and_compile(
        r#"
        pub fn update(mut slots: [Option<i32>; 2], mut value: Option<i32>) -> i32 {
            assert_eq!(slots[0].replace(7), None);
            let mut locals: [Option<i32>; 2] = [None, None];
            assert!(locals[1].replace(9).is_none());
            *value.get_or_insert(1) += 1;
            *value.get_or_insert_default() += 1;
            *value.get_or_insert_with(|| 2) += 1;
            assert!(value.take_if(|value| *value == 4).is_some());
            let _ = value.insert(3);
            if let Some(inner) = value.as_mut() { *inner += 1; }
            for inner in value.iter_mut() { *inner += 1; }
            for inner in value.as_mut_slice() { *inner += 1; }
            let mut text: Option<String> = Some(String::from("a"));
            if let Some(inner) = text.as_deref_mut() { inner.make_ascii_uppercase(); }
            value.clone_from(&Some(11));
            value.take().unwrap_or(0) + slots[0].unwrap_or(0) + locals[1].unwrap_or(0)
        }
    "#,
    );
    assert!(canonical.contains("mut slots"), "{canonical}");
    assert!(canonical.contains("mut locals"), "{canonical}");
    assert!(canonical.contains("mut value"), "{canonical}");
    assert!(canonical.contains("mut text"), "{canonical}");
}

#[test]
fn text_replace_stays_shared_without_confusing_same_name_mutable_methods() {
    let canonical = canonical_and_compile(
        r#"
        pub fn replace_text(mut text: String) -> String {
            let mut local: String = String::from("aa");
            let mut literal = "aa";
            format!("{}{}{}", text.replace("a", "b"), local.replace("a", "c"), literal.replace("a", "d"))
        }
    "#,
    );
    assert!(!canonical.contains("mut text"), "{canonical}");
    assert!(!canonical.contains("mut local"), "{canonical}");
    assert!(!canonical.contains("mut literal"), "{canonical}");
    let shadow = canonical_and_compile(
        r#"
        pub struct String { value: i32 }
        impl String { pub fn replace(&mut self, value: i32) { self.value = value; } }
        pub fn mutate(mut value: String) -> i32 { value.replace(7); value.value }
    "#,
    );
    assert!(shadow.contains("mut value"), "{shadow}");
}
