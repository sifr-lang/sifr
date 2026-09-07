use super::canonicalize_fields;
use std::collections::BTreeMap;

fn project(sources: &[(&str, &str)]) -> BTreeMap<String, String> {
    sources
        .iter()
        .map(|(name, source)| (name.to_string(), source.to_string()))
        .collect()
}

fn compile_modules(sources: &BTreeMap<String, String>) {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "sifr-field-identity-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory).expect("unique compiler output directory");
    let source = sources
        .iter()
        .map(|(module, source)| {
            if module.is_empty() {
                source.clone()
            } else {
                format!("mod {module} {{ {source} }}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut child = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-name=field_identity",
            "--crate-type=lib",
            "--emit=metadata",
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
        .expect("rustc stdin")
        .write_all(source.as_bytes())
        .expect("write Rust");
    let output = child.wait_with_output().expect("rustc completion");
    std::fs::remove_dir_all(&directory).expect("remove owned test artifacts");
    assert!(
        output.status.success(),
        "{}\n{source}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn project_field_identity_imports_collisions_patterns_and_members() {
    let sources = project(&[
        (
            "left",
            "pub struct Value { pub _payload: i64, pub payload: i64 }",
        ),
        ("right", "pub struct Value { pub _payload: i64 }"),
        ("exports", "pub use crate::left::Value as Exported;"),
        (
            "consumer",
            r#"
            use crate::exports::Exported as Alias;
            use crate::right::Value;
            pub fn read(left: Alias, right: &Value) -> i64 {
                let Alias { _payload, payload } = left;
                let value = Value { _payload };
                assert_eq!(value._payload, right._payload);
                _payload + payload + value._payload
            }
        "#,
        ),
    ]);
    let output = canonicalize_fields(&sources).expect("qualified field identities");
    assert!(output["left"].contains("pub payload_field: i64"));
    assert!(output["right"].contains("pub payload: i64"));
    assert!(output["consumer"].contains("payload_field: _payload"));
    assert!(output["consumer"].contains("payload: _payload"));
    assert!(output["consumer"].contains("value.payload, right.payload"));
    assert_eq!(canonicalize_fields(&output).expect("idempotent"), output);
    compile_modules(&output);
}

#[test]
fn project_field_identity_nested_modules_aliases_self_and_return_types() {
    let sources = project(&[(
        "",
        r#"
        mod records {
            pub struct Value { pub __sifr_payload: i64 }
            impl Value {
                pub fn new(value: i64) -> Self { Self { __sifr_payload: value } }
                pub fn get(&self) -> i64 { self.__sifr_payload }
            }
        }
        mod consumer {
            use super::records::*;
            type Alias = Value;
            fn read(value: &Alias) -> i64 { Value::new(5).__sifr_payload + value.__sifr_payload }
        }
    "#,
    )]);
    let output = canonicalize_fields(&sources).expect("nested module and return identities");
    assert!(output[""].contains("pub payload: i64"));
    assert!(output[""].contains("self.payload"));
    assert!(output[""].contains("Value::new(5).payload + value.payload"));
    compile_modules(&output);
}

#[test]
fn project_field_identity_preserves_external_fields_and_local_shadowing() {
    let sources = project(&[(
        "",
        r#"
        struct Local { _payload: i64 }
        fn read(value: Local, external: &::foreign::Local) -> i64 {
            let result = value._payload;
            { let value = external; assert_eq!(value._payload, 2); }
            result + value._payload + external._payload
        }
    "#,
    )]);
    let output = canonicalize_fields(&sources).expect("external fields stay external");
    assert!(output[""].contains("result + value.payload + external._payload"));
    assert!(output[""].contains("value._payload, 2"));
}

#[test]
fn project_field_identity_unresolved_receiver_fails_closed() {
    let sources = project(&[(
        "",
        "struct Local { _payload: i64 } fn read() { unknown()._payload; }",
    )]);
    assert!(
        canonicalize_fields(&sources)
            .expect_err("no name-only substitution")
            .contains("cannot resolve generated field owner")
    );
}

#[test]
fn project_field_identity_same_owner_collision_and_keyword_are_injective() {
    let sources = project(&[(
        "",
        r#"
        struct Value { __payload: i64, _payload: i64, payload: i64, payload_field: i64, _type: i64 }
        fn read(value: Value) -> i64 { value.__payload + value._payload + value.payload + value.payload_field + value._type }
    "#,
    )]);
    let output = canonicalize_fields(&sources).expect("collision disambiguation");
    assert!(output[""].contains("payload_field_field: i64"));
    assert!(output[""].contains("payload_field_field_field: i64"));
    assert!(output[""].contains("r#type: i64"));
    assert!(output[""].contains("value.r#type"));
    compile_modules(&output);
}

#[test]
fn project_field_identity_full_pipeline_keeps_shorthand_value_namespace() {
    let sources = project(&[
        (
            "declaration",
            "pub struct Value { pub _payload: i64, pub payload: i64 }",
        ),
        (
            "",
            r#"
            mod declaration;
            use crate::declaration::Value;
            pub fn read(_payload: i64) -> i64 {
                let value = Value { _payload, payload: 7 };
                let Value { _payload, payload } = value;
                _payload + payload
            }
        "#,
        ),
    ]);
    let output = crate::canonicalize_generated_rust_project(&sources).expect("project pipeline");
    assert!(output["declaration"].contains("pub payload_field: i64"));
    assert!(output[""].contains("payload_field: sifr_generated_payload"));
    for source in output.values() {
        syn::parse_file(source).expect("valid Rust");
    }
}

#[test]
fn project_field_identity_loop_bindings_and_declared_method_returns() {
    let sources = project(&[(
        "",
        r#"
        struct Left { _payload: i64, payload: i64 }
        #[derive(Clone)]
        struct Right { _payload: i64 }
        impl Left { fn clone(&self) -> Right { Right { _payload: 3 } } }
        fn read(value: Left, values: Vec<Right>) -> i64 {
            let mut total = value.clone()._payload;
            for value in values.iter() { total += value._payload; }
            let mut values = values.into_iter();
            while let Some(value) = values.next() { total += value._payload; }
            let repeated = vec![Right { _payload: 2 }; 3];
            total + value._payload + repeated[0]._payload
        }
    "#,
    )]);
    let output = canonicalize_fields(&sources).expect("loop scope and declared return identity");
    assert!(output[""].contains("value.clone().payload"));
    assert!(output[""].contains("total += value.payload;"));
    assert!(output[""].contains("total + value.payload_field"));
    compile_modules(&output);
}

#[test]
fn project_field_identity_result_closure_and_error_pattern_payloads() {
    let sources = project(&[
        (
            "errors",
            "pub struct Failure { pub __sifr_payload: i64, pub payload: i64 }",
        ),
        (
            "",
            r#"
            use crate::errors::Failure;
            fn load() -> Result<i64, Failure> { Err(Failure { __sifr_payload: 3, payload: 2 }) }
            fn convert() -> Result<i64, Failure> {
                let result = (|| -> Result<i64, Failure> {
                    let value = load().map_err(|error: _| Failure {
                        __sifr_payload: error.__sifr_payload,
                        payload: error.payload,
                    })?;
                    Ok(value)
                })();
                match result { Ok(value) => Ok(value), Err(error) => Err(Failure {
                    __sifr_payload: error.__sifr_payload, payload: error.payload,
                }) }
            }
        "#,
        ),
    ]);
    let output = canonicalize_fields(&sources).expect("typed error flow");
    assert!(!output[""].contains(".__sifr_payload"));
    compile_modules(&output);
}

#[test]
fn project_field_identity_generic_member_chain_and_variant_payload() {
    let sources = project(&[(
        "",
        r#"
        mod records {
            pub struct Value { pub _payload: i64 }
            pub struct Holder<T> { pub _inner: T }
            pub enum Event { Ready(Value) }
        }
        use records::{Value, Holder, Event};
        type Alias = Holder<Value>;
        fn read(holder: Alias, event: Event) -> i64 {
            let result = holder._inner._payload;
            match event { Event::Ready(value) => result + value._payload }
        }
    "#,
    )]);
    let output = canonicalize_fields(&sources).expect("instantiated receiver type");
    assert!(output[""].contains("holder.inner.payload"));
    assert!(output[""].contains("result + value.payload"));
    compile_modules(&output);
}
