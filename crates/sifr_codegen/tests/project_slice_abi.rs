use sifr_codegen::{canonicalize_generated_rust_project, finalize_formatted_generated_rust_source};
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

#[test]
fn project_slice_abi_long_transitive_chain_has_no_fixed_depth_limit() {
    // Alternate physical modules so a module-count bound cannot hide a
    // remaining limit on the number of caller/callee transitions.
    for modules in [["a", "z"], ["z", "a"]] {
        let mut sources = BTreeMap::from([
            (
                String::new(),
                format!(
                    "mod a; mod z; fn main() {{ assert_eq!({}::step23(vec![1_i32, 2]), 2); }}",
                    modules[1],
                ),
            ),
            ("a".to_owned(), String::new()),
            ("z".to_owned(), String::new()),
        ]);
        for index in 0..24 {
            let body = if index == 0 {
                "values.len()".to_owned()
            } else {
                format!(
                    "crate::{}::step{}(values)",
                    modules[(index - 1) % 2],
                    index - 1
                )
            };
            sources
                .get_mut(modules[index % 2])
                .expect("physical module")
                .push_str(&format!(
                    "pub fn step{index}(values: Vec<i32>) -> usize {{ {body} }}\n"
                ));
        }
        qualify(sources);
    }
}

#[test]
fn project_slice_abi_local_shadow() {
    qualify(project(&[
        (
            "",
            "mod metrics; use crate::metrics::score; fn main() { let score = |v: Vec<i32>| v; assert_eq!(score(vec![1_i32,2]), vec![1_i32,2]); assert_eq!(metrics::score(vec![1_i32]), 1); }",
        ),
        (
            "metrics",
            "pub fn score(values: Vec<i32>) -> usize { values.len() }",
        ),
    ]));
}
#[test]
fn project_slice_abi_parameter_shadow() {
    qualify(project(&[
        (
            "",
            "mod metrics; use crate::metrics::score; fn identity(v: Vec<i32>) -> Vec<i32> { v } fn relay(score: fn(Vec<i32>)->Vec<i32>) -> Vec<i32> { score(vec![1_i32,2]) } fn main() { assert_eq!(relay(identity), vec![1_i32,2]); }",
        ),
        (
            "metrics",
            "pub fn score(values: Vec<i32>) -> usize { values.len() }",
        ),
    ]));
}
#[test]
fn project_slice_abi_block_item_shadow() {
    qualify(project(&[
        (
            "",
            "mod metrics; use crate::metrics::score; fn main() { fn score(v: Vec<i32>) -> Vec<i32> { v } assert_eq!(score(vec![1_i32,2]), vec![1_i32,2]); }",
        ),
        (
            "metrics",
            "pub fn score(values: Vec<i32>) -> usize { values.len() }",
        ),
    ]));
}
#[test]
fn project_slice_abi_module_alias() {
    qualify(project(&[
        (
            "",
            "mod metrics; use crate::metrics as alias; fn main() { assert_eq!(alias::score(vec![1_i32,2]), 2); }",
        ),
        (
            "metrics",
            "pub fn score(values: Vec<i32>) -> usize { values.len() }",
        ),
    ]));
}

#[test]
fn project_slice_abi_slice_binding_shadow() {
    qualify(project(&[
        (
            "",
            "mod metrics; use crate::metrics::score; fn forward(values: &[i32]) -> usize { let values = values.to_vec(); score(&values) } fn main() { assert_eq!(forward(&[1_i32,2]), 2); }",
        ),
        (
            "metrics",
            "pub fn score(values: &[i32]) -> usize { values.len() }",
        ),
    ]));
}
#[test]
fn project_slice_abi_owned_function_value() {
    qualify(project(&[
        (
            "",
            "mod metrics; fn main() { let callback: fn(Vec<i32>)->usize = metrics::score; assert_eq!(callback(vec![1_i32,2]), 2); }",
        ),
        (
            "metrics",
            "pub fn score(values: Vec<i32>) -> usize { values.len() }",
        ),
    ]));
}
#[test]
fn project_slice_abi_borrowed_function_value() {
    qualify(project(&[
        (
            "",
            "mod metrics; fn main() { let callback: fn(&Vec<i32>)->usize = metrics::score; assert_eq!(callback(&vec![1_i32,2]), 2); }",
        ),
        (
            "metrics",
            "pub fn score(values: &Vec<i32>) -> usize { values.len() }",
        ),
    ]));
}
#[test]
fn project_slice_abi_private_parent_consumer() {
    qualify(project(&[
        (
            "",
            "mod parent; fn main() { assert_eq!(parent::child::run(), 2); }",
        ),
        (
            "parent",
            "pub mod child; fn score(values: Vec<i32>) -> usize { values.len() }",
        ),
        (
            "parent::child",
            "pub fn run() -> usize { super::score(vec![1_i32,2]) }",
        ),
    ]));
}

#[test]
fn project_slice_literals_require_the_receiving_signature() {
    let output = qualify(project(&[
        (
            "",
            r#"
            mod api;
            use api::Writer;
            fn main() {
                let writer: Writer = Writer;
                assert_eq!(writer.write_all(&vec![3_u8, 5_u8]), 8);
                assert_eq!(api::sum(&vec![2_u8, 7_u8]), 9);
                let callback: fn(&[u8]) -> u8 = api::sum;
                assert_eq!(callback(&vec![1_u8, 4_u8]), 5);
            }
        "#,
        ),
        (
            "api",
            r#"
            pub struct Writer;
            impl Writer {
                pub fn write_all(&self, values: &[u8]) -> u8 { values.iter().sum() }
            }
            pub fn sum(values: &[u8]) -> u8 { values.iter().sum() }
        "#,
        ),
    ]));
    let compact = output[""].split_whitespace().collect::<String>();
    assert!(!compact.contains("vec!"), "{compact}");
}

#[test]
fn project_slice_literals_preserve_owned_elements_and_evaluation_order() {
    let output = qualify(project(&[
        (
            "",
            r#"
            mod api;
            fn main() {
                let mut next = 0;
                let command: api::Command = api::Command;
                assert_eq!(command.args(&vec![
                    { next += 1; next.to_string() },
                    { next += 1; next.to_string() },
                ]), "12");
                assert_eq!(next, 2);
            }
        "#,
        ),
        (
            "api",
            r#"
            pub struct Command;
            impl Command {
                pub fn args(&self, values: &[String]) -> String { values.join("") }
            }
        "#,
        ),
    ]));
    assert!(!output[""].contains("vec!"), "{}", output[""]);
}

#[test]
fn project_slice_literals_preserve_runtime_repeat_and_clone_semantics() {
    qualify(project(&[
        (
            "",
            r#"
            mod api;
            fn main() {
                let count = std::env::args().count() + 1;
                assert_eq!(api::joined(&vec![String::from("x"); count]), "xx");
            }
        "#,
        ),
        (
            "api",
            r#"pub fn joined(values: &[String]) -> String { values.join("") }"#,
        ),
    ]));
}

#[test]
fn project_slice_literals_preserve_shadowed_macro_behavior() {
    qualify(project(&[
        (
            "",
            r#"
            mod api;
            macro_rules! vec { ($value:expr) => { std::vec![$value + 1] }; }
            fn main() { assert_eq!(api::sum(&vec![3_u8]), 4); }
        "#,
        ),
        (
            "api",
            "pub fn sum(values: &[u8]) -> u8 { values.iter().sum() }",
        ),
    ]));
}
