//! Native regressions for inherent calls made by trait-based slot adapters.
use crate::canonicalize_generated_rust_source;
use std::process::Command;

fn compile_and_run(source: &str) -> String {
    let canonical = canonicalize_generated_rust_source(source).expect("canonical Rust");
    compile_pair(source, &canonical);
    canonical
}

fn typed_compile_and_run(source: &str) -> String {
    let mut file = syn::parse_file(source).expect("Rust fixture");
    super::rewrite(&mut file);
    let canonical = prettyplease::unparse(&file);
    compile_pair(source, &canonical);
    canonical
}

// Preserve failed inputs (including unexpected I/O failures) while successful
// native checks release their owned scratch directory automatically.
struct NativeInputs(Option<tempfile::TempDir>);

impl Drop for NativeInputs {
    fn drop(&mut self) {
        if std::thread::panicking()
            && let Some(directory) = self.0.take()
        {
            eprintln!(
                "Retained native regression inputs: {}",
                directory.keep().display()
            );
        }
    }
}

fn compile_pair(source: &str, canonical: &str) {
    let inputs = NativeInputs(Some(
        tempfile::Builder::new()
            .prefix("sifr-method-slot-borrows-")
            .tempdir()
            .expect("owned native regression directory"),
    ));
    let directory = inputs.0.as_ref().expect("native inputs are owned").path();
    // Keep both inputs and compiler diagnostics on failure.
    for (name, text) in [("before", source), ("after", canonical)] {
        let input = directory.join(format!("{name}.rs"));
        let binary = directory.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&input, text).expect("write native input");
        let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args(["--edition=2024", "--crate-name", "method_slot_borrows"])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .expect("rustc");
        std::fs::write(directory.join(format!("{name}.stderr")), &output.stderr)
            .expect("retain diagnostics");
        assert!(
            output.status.success(),
            "{name}: {}\n{text}\n{}",
            String::from_utf8_lossy(&output.stderr),
            directory.display()
        );
        let output = Command::new(&binary)
            .output()
            .expect("execute native assertions");
        assert!(
            output.status.success(),
            "{name}: {}\n{}",
            String::from_utf8_lossy(&output.stderr),
            directory.display()
        );
    }
}

#[test]
fn method_slot_static_self_borrows_resolve_the_enclosing_owner() {
    let rust = compile_and_run(
        r#"
        mod nested {
            pub trait Slot { fn invoke(value: String) -> String; }
            pub struct Model;
            impl Model { fn normalize(value: String) -> String { value.to_uppercase() } }
            impl Slot for Model {
                fn invoke(value: String) -> String { Self::normalize(value) }
            }
            pub struct Other;
            impl Other { fn normalize(value: String) -> String { value.to_lowercase() } }
            impl Slot for Other {
                fn invoke(value: String) -> String { Self::normalize(value) }
            }
            pub fn run() {
                assert_eq!(<Model as Slot>::invoke(String::from("hello")), "HELLO");
                assert_eq!(<Other as Slot>::invoke(String::from("HELLO")), "hello");
            }
        }
        fn main() { nested::run(); }
"#,
    );
    assert_eq!(
        rust.matches("fn normalize(value: &str)").count(),
        2,
        "{rust}"
    );
    assert_eq!(
        rust.matches("Self::normalize(value.as_str())").count(),
        2,
        "{rust}"
    );
}

#[test]
fn method_slot_unit_receiver_borrows_follow_declared_constructor_types() {
    let rust = compile_and_run(
        r#"
        trait Slot { fn invoke(value: String) -> String; }
        struct Model;
        impl Model { fn normalize(&self, value: String) -> String { value.to_uppercase() } }
        impl Slot for Model {
            fn invoke(value: String) -> String {
                let receiver = Model;
                receiver.normalize(value)
            }
        }
        fn main() {
            assert_eq!(<Model as Slot>::invoke(String::from("hello")), "HELLO");
        }
    "#,
    );
    assert!(rust.contains("fn normalize(&self, value: &str)"), "{rust}");
    assert!(
        rust.contains("receiver.normalize(value.as_str())"),
        "{rust}"
    );
}

#[test]
fn method_slot_decoded_input_types_survive_opaque_runtime_boundaries() {
    let rust = compile_and_run(
        r#"
        trait Slot { fn invoke() -> String; }
        struct Model;
        impl Model {
            fn normalize(value: String) -> String { value.to_uppercase() }
            fn serialize(&self, value: String) -> String { value.to_lowercase() }
        }
        impl Slot for Model {
            fn invoke() -> String {
                let value: String = std::convert::identity::<String>(String::from("hello"));
                assert_eq!(Self::normalize(value), "HELLO");
                let (receiver, value): (Model, String) =
                    std::convert::identity::<(Model, String)>((Model, String::from("HELLO")));
                receiver.serialize(value)
            }
        }
        fn main() { assert_eq!(<Model as Slot>::invoke(), "hello"); }
    "#,
    );
    assert!(rust.contains("fn normalize(value: &str)"), "{rust}");
    assert!(rust.contains("fn serialize(&self, value: &str)"), "{rust}");
}

#[test]
fn method_slot_constructor_facts_keep_value_and_type_namespaces_separate() {
    let rust = typed_compile_and_run(
        r#"
        struct Model;
        impl Model { fn normalize(&self, value: &str) -> usize { value.len() } }
        struct Other;
        impl Other {
            fn normalize(&self, value: String) -> usize {
                let length = value.len();
                std::mem::drop(value);
                length
            }
        }
        fn main() {
            {
                const Model: Other = Other;
                let receiver = Model;
                let value = String::from("hello");
                assert_eq!(receiver.normalize(value), 5);
                let typed: Model = crate::Model;
                assert_eq!(typed.normalize("hello"), 5);
            }
            {
                static Model: Other = Other;
                let receiver = &Model;
                let value = String::from("hello");
                assert_eq!(receiver.normalize(value), 5);
            }
            let receiver = { const Model: Other = Other; Model };
            let value = String::from("hello");
            assert_eq!(receiver.normalize(value), 5);
            let marker = Model;
            {
                const marker: Other = Other;
                let receiver = marker;
                let value = String::from("hello");
                assert_eq!(receiver.normalize(value), 5);
            }
            assert_eq!(marker.normalize("hello"), 5);
            // The value shadow must not leak out of its lexical block.
            let receiver = Model;
            assert_eq!(receiver.normalize("hello"), 5);
        }
    "#,
    );
    assert!(!rust.contains(".as_str()"), "{rust}");
}

#[test]
fn method_slot_self_does_not_resolve_a_shadowed_block_local_owner() {
    let rust = typed_compile_and_run(
        r#"
        struct Model;
        impl Model { fn normalize(value: &str) -> usize { value.len() } }
        fn main() {
            struct Model;
            impl Model {
                fn normalize(value: String) -> usize { value.len() }
                fn invoke(value: String) -> usize { Self::normalize(value) }
            }
            assert_eq!(Model::invoke(String::from("hello")), 5);
        }
    "#,
    );
    assert!(!rust.contains(".as_str()"), "{rust}");
}

#[test]
fn method_slot_self_does_not_escape_into_a_nested_trait() {
    let rust = typed_compile_and_run(
        r#"
        struct Model;
        impl Model {
            fn normalize(value: &str) -> usize { value.len() }
            fn run() {
                trait Local {
                    fn normalize(value: String) -> usize;
                    fn invoke(value: String) -> usize {
                        let text: String = value;
                        Self::normalize(text)
                    }
                }
                struct Other;
                impl Local for Other {
                    fn normalize(value: String) -> usize { value.len() }
                }
                assert_eq!(<Other as Local>::invoke(String::from("hello")), 5);
            }
        }
        fn main() { Model::run(); }
    "#,
    );
    assert!(!rust.contains(".as_str()"), "{rust}");
}

#[test]
fn method_slot_self_and_constructor_keep_opaque_owner_paths() {
    let rust = typed_compile_and_run(
        r#"
        struct Model;
        impl Model { fn normalize(value: &str) -> usize { value.len() } }
        mod other {
            pub struct Other;
            impl Other { pub fn normalize(value: String) -> usize { value.len() } }
        }
        mod owner {
            pub struct Model;
            impl Model {
                pub fn normalize(value: &str) -> usize { value.len() }
                pub fn serialize(&self, value: &str) -> usize { value.len() }
            }
        }
        fn through_alias() {
            use other::Other as Model;
            impl Model {
                fn invoke(value: String) -> usize { Self::normalize(value) }
            }
            assert_eq!(Model::invoke(String::from("hello")), 5);
        }
        fn through_local_module() {
            mod owner {
                pub struct Model;
                impl Model {
                    pub fn normalize(value: String) -> usize { value.len() }
                    pub fn serialize(&self, value: String) -> usize { value.len() }
                }
            }
            impl owner::Model {
                fn invoke(value: String) -> usize { Self::normalize(value) }
            }
            assert_eq!(owner::Model::invoke(String::from("hello")), 5);
            let receiver = owner::Model;
            let value = String::from("hello");
            assert_eq!(receiver.serialize(value), 5);
        }
        fn main() { through_alias(); through_local_module(); }
    "#,
    );
    assert!(!rust.contains(".as_str()"), "{rust}");
    // Isolate constructor lookup from the opaque import in the first program.
    let rust = typed_compile_and_run(
        r#"
        mod owner {
            pub struct Model;
            impl Model { pub fn serialize(&self, value: &str) -> usize { value.len() } }
        }
        fn main() {
            mod owner {
                pub struct Model;
                impl Model { pub fn serialize(&self, value: String) -> usize { value.len() } }
            }
            let receiver = owner::Model;
            let value = String::from("hello");
            assert_eq!(receiver.serialize(value), 5);
        }
    "#,
    );
    assert!(!rust.contains(".as_str()"), "{rust}");
}
