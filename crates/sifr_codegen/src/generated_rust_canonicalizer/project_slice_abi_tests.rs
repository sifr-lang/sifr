use crate::{canonicalize_generated_rust_project, finalize_formatted_generated_rust_source};
use std::collections::BTreeMap;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

fn native_project(sources: &BTreeMap<String, String>, stage: &str) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "sifr-project-slice-abi-{}-{}-{stage}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).expect("owned native regression directory");
    for (module, source) in sources {
        let path = root.join(if module.is_empty() {
            "main.rs".to_owned()
        } else {
            format!("{}.rs", module.replace("::", "/"))
        });
        std::fs::create_dir_all(path.parent().expect("source parent"))
            .expect("owned module directories");
        std::fs::write(path, source).expect("native regression source");
    }
    let binary = root.join(format!("program{}", std::env::consts::EXE_SUFFIX));
    let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg("--edition=2024")
        .arg(root.join("main.rs"))
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("native compiler");
    assert!(
        output.status.success(),
        "{stage}: {}\nsources={sources:#?}\nretained={}",
        String::from_utf8_lossy(&output.stderr),
        root.display()
    );
    let output = Command::new(binary).output().expect("native assertions");
    assert!(
        output.status.success(),
        "{stage}: {}\nretained={}",
        String::from_utf8_lossy(&output.stderr),
        root.display()
    );
    std::fs::remove_dir_all(root).expect("remove successful owned regression output");
}

fn qualify(sources: BTreeMap<String, String>) -> BTreeMap<String, String> {
    native_project(&sources, "original");
    let canonical =
        canonicalize_generated_rust_project(&sources).expect("project canonicalization");
    native_project(&canonical, "canonical");
    assert_eq!(
        canonicalize_generated_rust_project(&canonical).expect("stable project canonicalization"),
        canonical,
        "complete project ABI must be stable after one invocation"
    );
    let finalized = canonical
        .iter()
        .map(|(name, source)| {
            (
                name.clone(),
                finalize_formatted_generated_rust_source(source).expect("API finalization"),
            )
        })
        .collect();
    native_project(&finalized, "finalized");
    canonical
}

fn project(sources: &[(&str, &str)]) -> BTreeMap<String, String> {
    sources
        .iter()
        .map(|(name, source)| ((*name).to_owned(), (*source).to_owned()))
        .collect()
}

#[test]
fn project_slice_abi_direct_and_macro_calls() {
    qualify(project(&[
        (
            "",
            r#"
            mod metrics;
            use crate::metrics::score;
            fn main() {
                let values = vec![3_i32, 5, 7];
                let count = score(values);
                assert_eq!(count, 3);
                assert_eq!(score(vec![2_i32, 4]), 2);
                assert_eq!(format!("{}", score(vec![9_i32])), "1");
            }
        "#,
        ),
        (
            "metrics",
            "pub fn score(values: Vec<i32>) -> usize { values.len() }",
        ),
    ]));
}

#[test]
fn project_slice_abi_scoped_aliases_and_reexports_keep_distinct_owners() {
    qualify(project(&[
        (
            "",
            r#"
            mod nested { pub mod read; pub mod consume; }
            mod facade;
            use crate::facade::read_count as imported;
            fn main() {
                assert_eq!(imported(vec![1_i32, 2, 3]), 3);
                assert_eq!(nested::read::score(vec![1_i32]), 1);
                assert_eq!(nested::consume::score(vec![1_i32]), vec![1_i32, 9]);
            }
        "#,
        ),
        (
            "facade",
            "pub use crate::nested::read::score as read_count;",
        ),
        (
            "nested::read",
            "pub fn score(values: Vec<i32>) -> usize { values.len() }",
        ),
        (
            "nested::consume",
            "pub fn score(mut values: Vec<i32>) -> Vec<i32> { values.push(9); values }",
        ),
    ]));
}

#[test]
fn project_slice_abi_transitive_calls_are_stable_in_both_module_orders() {
    for [leaf, middle, outer] in [["a", "b", "c"], ["z", "y", "x"]] {
        let sources = BTreeMap::from([
            (
                String::new(),
                format!(
                    "mod {leaf}; mod {middle}; mod {outer}; fn main() {{ assert_eq!({outer}::run(vec![1_i32, 2]), 2); }}"
                ),
            ),
            (
                leaf.to_owned(),
                "pub fn score(values: Vec<i32>) -> usize { values.len() }".to_owned(),
            ),
            (
                middle.to_owned(),
                format!(
                    "use crate::{leaf}::score as count; pub fn relay(values: Vec<i32>) -> usize {{ count(values) }}"
                ),
            ),
            (
                outer.to_owned(),
                format!(
                    "pub fn run(values: Vec<i32>) -> usize {{ crate::{middle}::relay(values) }}"
                ),
            ),
        ]);
        qualify(sources);
    }
}

#[test]
fn project_slice_abi_preserves_consuming_and_mutating_vector_parameters() {
    qualify(project(&[
        (
            "",
            r#"
            mod owned;
            fn main() {
                assert_eq!(owned::append(vec![1_i32]), vec![1_i32, 9]);
                assert_eq!(owned::consume(vec![2_i32, 3]), 5);
            }
        "#,
        ),
        (
            "owned",
            r#"
            pub fn append(mut values: Vec<i32>) -> Vec<i32> { values.push(9); values }
            pub fn consume(values: Vec<i32>) -> i32 { values.into_iter().sum() }
        "#,
        ),
    ]));
}
