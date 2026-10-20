//! Native regressions for inherent calls made by trait-based slot adapters.
use crate::canonicalize_generated_rust_source;
use std::process::Command;

fn compile_and_run(source: &str) -> String {
    let canonical = canonicalize_generated_rust_source(source).expect("canonical Rust");
    let directory = tempfile::Builder::new()
        .prefix("sifr-method-slot-borrows-")
        .tempdir()
        .expect("owned native regression directory")
        .keep();
    // Keep both inputs and compiler diagnostics, including on failure.
    for (name, text) in [("before", source), ("after", canonical.as_str())] {
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
    canonical
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
        }
        fn main() {
            assert_eq!(<nested::Model as nested::Slot>::invoke(String::from("hello")), "HELLO");
            assert_eq!(<nested::Other as nested::Slot>::invoke(String::from("HELLO")), "hello");
        }
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
fn method_slot_constructor_facts_do_not_cross_shadowing_bindings_or_items() {
    let mut file = syn::parse_file(
        r#"
        struct Model;
        impl Model { fn normalize(&self, value: &str) {} }
        fn run(opaque: Other, value: String) {
            let Model = opaque;
            Model.normalize(value);
            { struct Model { field: i32 }
              let receiver = Model { field: 1 };
              receiver.normalize(value); }
        }
    "#,
    )
    .expect("Rust fixture");
    super::rewrite(&mut file);
    let rust = prettyplease::unparse(&file);
    assert!(rust.contains("Model.normalize(value)"), "{rust}");
    assert!(rust.contains("receiver.normalize(value)"), "{rust}");
    assert!(!rust.contains(".as_str()"), "{rust}");
}
