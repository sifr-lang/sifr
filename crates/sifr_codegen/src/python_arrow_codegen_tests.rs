use crate::generate_rust;
use sifr_lowering::{ExternalDefs, LoweringOptions, PythonBridgeTargetAuthority};
use std::collections::BTreeMap;

const ERROR: &str = r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str
"#;

fn generate(source: &str) -> String {
    let parsed = sifr_python_parser::parse_module(source).expect("source should parse");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("source should lower");
    generate_rust(&lowered.module)
}

#[test]
fn arrow_import_producer_acquires_derived_resource_without_copy_path() {
    let rust = generate(&format!(
        "{ERROR}\n@python.arrow(pkg.make_array, schema=omitted)\ndef acquire() -> Result[python.ArrowArray, PythonError]: ...\n"
    ));

    assert!(
        rust.contains("::sifr_runtime::python::call_object_owned"),
        "{rust}"
    );
    assert!(
        rust.contains("::sifr_stdlib::python::PythonArrowArray::acquire_foreign"),
        "{rust}"
    );
    assert!(!rust.contains("copy"), "{rust}");
    syn::parse_file(&rust).expect("generated Arrow Rust should parse");
}

#[test]
fn arrow_bridge_producer_uses_resolved_runtime_target() {
    let parsed = sifr_python_parser::parse_module(&format!(
        "{ERROR}\n@python.arrow(bridge.arrow.make, schema=omitted)\ndef acquire() -> Result[python.ArrowArray, PythonError]: ...\n"
    ))
    .expect("source should parse");
    let lowered = sifr_lowering::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &ExternalDefs::default(),
        LoweringOptions {
            python_bridge_authorities: BTreeMap::from([(
                "main".to_string(),
                PythonBridgeTargetAuthority {
                    runtime_package: "__sifr_bridge__.p_abc123".to_string(),
                    modules: ["arrow".to_string()].into_iter().collect(),
                },
            )]),
            ..LoweringOptions::default()
        },
    )
    .expect("bridge declaration should lower");
    let rust = generate_rust(&lowered.module);

    assert!(rust.contains("__sifr_bridge__"), "{rust}");
    assert!(rust.contains("p_abc123"), "{rust}");
    assert!(rust.contains("PythonArrowArray::acquire_foreign"), "{rust}");
    syn::parse_file(&rust).expect("generated bridge Arrow Rust should parse");
}

#[test]
fn arrow_requested_schema_is_consumed_and_not_forwarded_to_producer_call() {
    let rust = generate(&format!(
        "{ERROR}\n@python.arrow(pkg.make_stream, schema=parameter(requested))\ndef acquire(*, own requested: python.ArrowSchema) -> Result[python.ArrowStream, PythonError]: ...\n"
    ));

    assert!(
        rust.contains("::sifr_stdlib::python::PythonArrowStream::acquire_foreign_with_schema"),
        "{rust}"
    );
    assert!(rust.contains("requested"), "{rust}");
    assert!(!rust.contains("__sifr_python_kwargs.push"), "{rust}");
    syn::parse_file(&rust).expect("generated requested-schema Rust should parse");
}

#[test]
fn requested_schema_can_precede_forwarded_keyword_parameters() {
    let rust = generate(&format!(
        "{ERROR}\n@python.arrow(pkg.make_stream, schema=parameter(requested))\ndef acquire(*, own requested: python.ArrowSchema, limit: int) -> Result[python.ArrowStream, PythonError]: ...\n"
    ));

    assert!(rust.contains("acquire_foreign_with_schema"), "{rust}");
    assert!(rust.contains("\"limit\""), "{rust}");
    assert!(!rust.contains("\"requested\".to_string()"), "{rust}");
    syn::parse_file(&rust).expect("generated requested-schema Rust should parse");
}

#[test]
fn arrow_self_receiver_acquires_directly_and_resource_methods_map_errors() {
    let rust = generate(&format!(
        "{ERROR}\n@python.opaque(type=pkg.Owner, cleanup=drop)\nclass Owner(NonSend):\n    @python.arrow(Self, schema=omitted)\n    def array(self) -> Result[python.ArrowArray, PythonError]: ...\n\ndef inspect(own value: python.ArrowArray) -> Result[None, PythonError]:\n    try:\n        names: list[str] = value.capsule_names()\n        module: str = value.producer_module()\n        released: None = value.release()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));

    assert!(
        rust.contains("::sifr_stdlib::python::PythonArrowArray::acquire_foreign"),
        "{rust}"
    );
    assert!(rust.contains("self.__sifr_python_object"), "{rust}");
    assert!(rust.contains("\"pkg.Owner\""), "{rust}");
    assert!(rust.contains("value.capsule_names().map_err("), "{rust}");
    assert!(rust.contains("value.producer_module()"), "{rust}");
    assert!(rust.contains("value.release().map_err("), "{rust}");
    syn::parse_file(&rust).expect("generated receiver Arrow Rust should parse");
}

#[test]
fn arrow_owned_consumer_argument_commits_move_and_reconciles_after_call() {
    let rust = generate(&format!(
        "{ERROR}\n@python(pkg.consume_array)\ndef consume(own value: python.ArrowArray) -> Result[int, PythonError]: ...\n"
    ));

    assert!(
        rust.contains("::sifr_stdlib::python::PythonArrowArray::prepare_argument(value)"),
        "{rust}"
    );
    assert!(
        rust.contains("__sifr_python_arrow_argument_0.object()"),
        "{rust}"
    );
    assert!(
        rust.contains("::std::mem::drop(__sifr_python_args)"),
        "{rust}"
    );
    assert!(
        rust.contains("__sifr_python_arrow_argument_0.finish()"),
        "{rust}"
    );
    assert!(
        rust.contains("::sifr_stdlib::python::reconcile_arrow_argument"),
        "{rust}"
    );
    syn::parse_file(&rust).expect("generated Arrow consumer Rust should parse");
}

#[test]
fn arrow_owned_consumer_method_prepares_and_reconciles_argument() {
    let rust = generate(&format!(
        "{ERROR}\n@python.opaque(type=pkg.Sink, cleanup=drop)\nclass Sink(NonSend):\n    @python(Self.push)\n    def push(self, own value: python.ArrowArray) -> Result[None, PythonError]: ...\n"
    ));

    assert!(
        rust.contains("::sifr_stdlib::python::PythonArrowArray::prepare_argument(value)"),
        "{rust}"
    );
    assert!(
        rust.contains("__sifr_python_arrow_argument_0.finish()"),
        "{rust}"
    );
    assert!(
        rust.contains("::sifr_stdlib::python::reconcile_arrow_argument"),
        "{rust}"
    );
    syn::parse_file(&rust).expect("generated Arrow consumer method Rust should parse");
}

#[test]
fn every_arrow_kind_has_a_certified_acquisition_path() {
    for (kind, rust_type) in [
        ("ArrowArray", "PythonArrowArray"),
        ("ArrowSchema", "PythonArrowSchema"),
        ("ArrowStream", "PythonArrowStream"),
        ("ArrowDeviceArray", "PythonArrowDeviceArray"),
        ("ArrowDeviceStream", "PythonArrowDeviceStream"),
    ] {
        let rust = generate(&format!(
            "{ERROR}\n@python.arrow(pkg.make, schema=omitted)\ndef acquire() -> Result[python.{kind}, PythonError]: ...\n"
        ));
        assert!(
            rust.contains(&format!("{rust_type}::acquire_foreign")),
            "{rust}"
        );
        assert!(rust.contains("\"pkg.make\""), "{rust}");
        syn::parse_file(&rust).expect("generated Arrow Rust should parse");
    }
}

#[test]
fn multiple_owned_arrow_arguments_preserve_positional_and_keyword_order() {
    let rust = generate(&format!(
        "{ERROR}\n@python(pkg.consume)\ndef consume(own first: python.ArrowArray, *, own second: python.ArrowStream) -> Result[int, PythonError]: ...\n"
    ));
    let first = rust
        .find("PythonArrowArray::prepare_argument(first)")
        .expect("first preparation");
    let second = rust
        .find("PythonArrowStream::prepare_argument(second)")
        .expect("second preparation");
    assert!(first < second, "{rust}");
    assert!(rust.contains("\"second\".to_string()"), "{rust}");
    assert!(
        rust.contains("__sifr_python_arrow_argument_0.finish()"),
        "{rust}"
    );
    assert!(
        rust.contains("__sifr_python_arrow_argument_1.finish()"),
        "{rust}"
    );
    syn::parse_file(&rust).expect("generated Arrow Rust should parse");
}
