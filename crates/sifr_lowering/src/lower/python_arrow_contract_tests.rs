use crate::{
    lower_module, ExternalDefs, HirDiagnostic, HirModule, LoweringOptions,
    PythonBridgeTargetAuthority,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{PythonArrowSchemaMode, PythonInteropDecoratorKind, PythonParameterKind};
use sifr_python_parser::parse_module;
use sifr_type_system::{OwnershipKind, PythonArrowKind, Type};
use std::collections::BTreeMap;

const ERROR: &str = r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str
"#;

fn lower_ok(source: &str) -> HirModule {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite())
        .map(|result| result.module)
        .expect("source should lower")
}

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    }
}

#[test]
fn arrow_resource_annotations_are_closed_affine_kinds() {
    let module = lower_ok(
        "def keep(array: python.ArrowArray, schema: python.ArrowSchema, stream: python.ArrowStream, device_array: python.ArrowDeviceArray, device_stream: python.ArrowDeviceStream) -> None:\n    return None\n",
    );
    let kinds = module.functions[0]
        .params
        .iter()
        .map(|parameter| parameter.ty.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            PythonArrowKind::Array,
            PythonArrowKind::Schema,
            PythonArrowKind::Stream,
            PythonArrowKind::DeviceArray,
            PythonArrowKind::DeviceStream,
        ]
        .map(Type::PythonArrow)
    );
    for resource in kinds {
        assert_eq!(resource.ownership(), OwnershipKind::Move);
        assert!(resource.contains_affine_resource());
        assert!(!resource.supports_derived_clone());
        assert!(!resource.supports_structural_equality());
    }
}

#[test]
fn arrow_declaration_derives_return_kind_and_omitted_schema() {
    let module = lower_ok(&format!(
        "{ERROR}\n@python.arrow(pkg.make_array, schema=omitted)\ndef array(size: int) -> Result[python.ArrowArray, PythonError]: ...\n"
    ));
    let function = &module.functions[0];
    let declaration = &function.python_interop[0];
    assert_eq!(declaration.kind, PythonInteropDecoratorKind::Arrow);
    assert_eq!(declaration.required_import_root.as_deref(), Some("pkg"));
    let arrow = declaration.arrow.as_ref().expect("Arrow contract");
    assert_eq!(arrow.kind, PythonArrowKind::Array);
    assert_eq!(arrow.schema, PythonArrowSchemaMode::Omitted);
    assert_eq!(declaration.parameters.len(), 1);
}

#[test]
fn bridge_arrow_producer_rewrites_to_package_runtime_identity() {
    let parsed = parse_module(&format!(
        "{ERROR}\n@python.arrow(bridge.arrow.make, schema=omitted)\ndef array() -> Result[python.ArrowArray, PythonError]: ...\n"
    ))
    .expect("source should parse");
    let lowered = crate::lower_module_with_externals_name_and_options(
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
    .expect("resolved bridge Arrow producer should lower");
    let declaration = &lowered.module.functions[0].python_interop[0];
    assert_eq!(
        declaration.target.as_ref().expect("target").dotted(),
        "__sifr_bridge__.p_abc123.arrow.make"
    );
    assert_eq!(declaration.required_import_root, None);
}

#[test]
fn requested_schema_is_required_keyword_only_owned_and_not_sent_to_producer() {
    let module = lower_ok(&format!(
        "{ERROR}\n@python.arrow(pkg.make_stream, schema=parameter(requested))\ndef stream(size: int, *, own requested: python.ArrowSchema) -> Result[python.ArrowStream, PythonError]: ...\n"
    ));
    let function = &module.functions[0];
    let declaration = &function.python_interop[0];
    let arrow = declaration.arrow.as_ref().expect("Arrow contract");
    assert_eq!(arrow.kind, PythonArrowKind::Stream);
    assert!(matches!(
        &arrow.schema,
        PythonArrowSchemaMode::Parameter { name, .. } if name == "requested"
    ));
    assert_eq!(declaration.parameters.len(), 1);
    assert_eq!(declaration.parameters[0].name, "size");
    assert_eq!(
        declaration.parameters[0].kind,
        PythonParameterKind::Positional
    );
    let schema = function
        .params
        .iter()
        .find(|parameter| parameter.name == "requested")
        .expect("schema parameter");
    assert!(schema.convention.is_owned());
}

#[test]
fn opaque_arrow_receiver_retains_self_and_optional_schema_only() {
    let module = lower_ok(&format!(
        "{ERROR}\n@python.opaque(type=pkg.Owner, cleanup=drop)\nclass Owner(NonSend):\n    @python.arrow(Self, schema=parameter(requested))\n    def stream(self, *, own requested: python.ArrowSchema) -> Result[python.ArrowStream, PythonError]: ...\n"
    ));
    let method = &module.classes[1].methods[0];
    let declaration = &method.python_interop[0];
    assert_eq!(
        declaration.target.as_ref().expect("target").segments,
        ["Self"]
    );
    assert!(declaration.parameters.is_empty());
}

#[test]
fn arrow_declaration_rejects_invalid_schema_and_signature_shapes() {
    for source in [
        "@python.arrow(pkg.make)\ndef bad() -> Result[python.ArrowArray, PythonError]: ...",
        "@python.arrow(pkg.make, schema=dynamic)\ndef bad() -> Result[python.ArrowArray, PythonError]: ...",
        "@python.arrow(pkg.make, schema=parameter(requested))\ndef bad(own requested: python.ArrowSchema) -> Result[python.ArrowArray, PythonError]: ...",
        "@python.arrow(pkg.make, schema=parameter(requested))\ndef bad(*, own requested: python.ArrowSchema = None) -> Result[python.ArrowArray, PythonError]: ...",
        "@python.arrow(pkg.make, schema=parameter(requested))\ndef bad(*, requested: python.ArrowSchema) -> Result[python.ArrowArray, PythonError]: ...",
        "@python.arrow(pkg.make, schema=parameter(requested))\ndef bad(*, own requested: python.ArrowArray) -> Result[python.ArrowArray, PythonError]: ...",
        "@python.arrow(pkg.make, schema=parameter(requested))\ndef bad(*, own requested: python.ArrowSchema) -> Result[python.ArrowSchema, PythonError]: ...",
        "@python.arrow(pkg.make, schema=omitted)\ndef bad() -> Result[bytes, PythonError]: ...",
        "@python.arrow(pkg.make, schema=omitted)\nasync def bad() -> Result[python.ArrowArray, PythonError]: ...",
    ] {
        let errors = lower_errors(&format!("{ERROR}\n{source}\n"));
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn arrow_resources_cannot_be_implicitly_copied() {
    let errors = lower_errors(
        "def duplicate(resource: python.ArrowArray) -> tuple[python.ArrowArray, python.ArrowArray]:\n    return (resource, resource)\n",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error.message.contains("borrowed affine Python resource")
        }),
        "{errors:?}"
    );
}

#[test]
fn arrow_consumer_arguments_require_owned_transfer() {
    let errors = lower_errors(&format!(
        "{ERROR}\n@python(pkg.consume)\ndef consume(value: python.ArrowArray) -> Result[int, PythonError]: ...\n"
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("must transfer ownership with `own`")),
        "{errors:?}"
    );

    lower_ok(&format!(
        "{ERROR}\n@python(pkg.consume)\ndef consume(own value: python.ArrowArray) -> Result[int, PythonError]: ...\n"
    ));
}

#[test]
fn arrow_consumers_reject_unlowerable_async_mutable_and_omittable_shapes() {
    for declaration in [
        "@python(pkg.consume)\ndef consume(own value: python.ArrowArray = python.omit) -> Result[int, PythonError]: ...",
        "@python.coroutine(pkg.consume)\nasync def consume(own value: python.ArrowArray) -> Result[int, PythonError]: ...",
        "@python.opaque(type=pkg.Owner, cleanup=drop)\nclass Owner(NonSend):\n    @python(Self.consume)\n    def consume(self, own mut value: python.ArrowArray) -> Result[int, PythonError]: ...",
    ] {
        let errors = lower_errors(&format!("{ERROR}\n{declaration}\n"));
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)),
            "{declaration}: {errors:?}"
        );
    }
}

#[test]
fn class_methods_preserve_owned_arrow_parameters() {
    let module = lower_ok(&format!(
        "{ERROR}\n@python.opaque(type=pkg.Owner, cleanup=drop)\nclass Owner(NonSend):\n    @python(Self.consume)\n    def consume(self, own value: python.ArrowArray) -> Result[int, PythonError]: ...\n"
    ));
    let parameter = &module.classes[1].methods[0].params[0];
    assert!(parameter.convention.is_owned());
    assert_eq!(parameter.ty, Type::PythonArrow(PythonArrowKind::Array));
}

#[test]
fn class_method_keyword_only_defaults_are_available_at_calls() {
    lower_ok(
        "class Config:\n    def value(self, *, extra: int = 5) -> int:\n        return extra\n\ndef read(config: Config) -> int:\n    return config.value()\n",
    );
}

#[test]
fn plain_method_move_parameters_follow_borrow_by_default() {
    let errors =
        lower_errors("class Echo:\n    def echo(self, value: str) -> str:\n        return value\n");
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES)
                && error.message.contains("borrowed parameter")
        }),
        "{errors:?}"
    );
    lower_ok("class Echo:\n    def echo(self, own value: str) -> str:\n        return value\n");
    let module = lower_ok(
        "class Sink[T]:\n    def send(self, value: T) -> None:\n        return None\n\ndef send_int(sink: Sink[int]) -> None:\n    sink.send(1)\n",
    );
    assert!(module.classes[0].methods[0].params[0]
        .convention
        .is_borrowed());
    let module = lower_ok(
        "class Sink[T]:\n    def send(self, own value: T) -> None:\n        return None\n",
    );
    assert!(module.classes[0].methods[0].params[0].convention.is_owned());
}

#[test]
fn class_method_borrowed_arrow_parameters_cannot_escape() {
    let errors = lower_errors(&format!(
        "{ERROR}\n@python(pkg.consume)\ndef consume(own value: python.ArrowArray) -> Result[int, PythonError]: ...\n\nclass Owner:\n    def misuse(self, value: python.ArrowArray) -> None:\n        result: int = consume(value)\n        return None\n"
    ));
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error.message.contains("borrowed affine Python resource")
        }),
        "{errors:?}"
    );
}

#[test]
fn a_single_call_cannot_consume_the_same_arrow_resource_twice() {
    for call in ["consume2(value, value)", "consume2(value, take(value))"] {
        let errors = lower_errors(&format!(
            "{ERROR}\ndef take(own value: python.ArrowArray) -> python.ArrowArray:\n    return value\n\n@python(pkg.consume2)\ndef consume2(own left: python.ArrowArray, own right: python.ArrowArray) -> Result[int, PythonError]: ...\n\ndef misuse(own value: python.ArrowArray) -> None:\n    result: int = {call}\n    return None\n"
        ));
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
                    && error.message.contains("moved")
            }),
            "{call}: {errors:?}"
        );
    }
}

#[test]
fn method_calls_consume_owned_arrow_arguments_once() {
    for invocation in [
        "sink.consume(value)\n    second: None = sink.consume(value)",
        "sink.consume2(value, right=value)",
    ] {
        let errors = lower_errors(&format!(
            "{ERROR}\nclass Sink:\n    def consume(self, own value: python.ArrowArray) -> None:\n        return None\n\n    def consume2(self, own left: python.ArrowArray, *, own right: python.ArrowArray) -> None:\n        return None\n\ndef misuse(sink: Sink, own value: python.ArrowArray) -> None:\n    first: None = {invocation}\n    return None\n"
        ));
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
                    && error.message.contains("moved")
            }),
            "{invocation}: {errors:?}"
        );
    }
}

#[test]
fn static_super_and_buffer_method_calls_consume_owned_arguments() {
    for source in [
        format!(
            "{ERROR}\nclass Tools:\n    @staticmethod\n    def consume(own value: python.ArrowArray) -> None:\n        return None\n\ndef misuse(own value: python.ArrowArray) -> None:\n    first: None = Tools.consume(value)\n    second: None = Tools.consume(value)\n    return None\n"
        ),
        format!(
            "{ERROR}\nclass Parent:\n    def consume(self, own value: python.ArrowArray) -> None:\n        return None\n\nclass Child(Parent):\n    def misuse(self, own value: python.ArrowArray) -> None:\n        first: None = super().consume(value)\n        second: None = super().consume(value)\n        return None\n"
        ),
        format!(
            "{ERROR}\nclass Sink:\n    def consume(self, own value: python.Buffer[uint8]) -> None:\n        return None\n\ndef misuse(sink: Sink, own value: python.Buffer[uint8]) -> None:\n    first: None = sink.consume(value)\n    second: None = sink.consume(value)\n    return None\n"
        ),
    ] {
        let errors = lower_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn explicit_constructors_consume_owned_affine_arguments() {
    for resource in ["python.ArrowArray", "python.Buffer[uint8]"] {
        let source = format!(
            "class Holder:\n    value: {resource}\n\n    def __init__(self, own value: {resource}):\n        self.value = value\n\ndef misuse(own value: {resource}) -> None:\n    first: Holder = Holder(value)\n    second: Holder = Holder(value)\n    return None\n"
        );
        let errors = lower_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn super_constructor_consumes_owned_affine_arguments() {
    for resource in ["python.ArrowArray", "python.Buffer[uint8]"] {
        let source = format!(
            "class Parent:\n    value: {resource}\n\n    def __init__(self, own value: {resource}):\n        self.value = value\n\nclass Child(Parent):\n    def __init__(self, own value: {resource}):\n        super().__init__(value)\n        second: Parent = Parent(value)\n\ndef keep(child: Child) -> None:\n    return None\n"
        );
        let errors = lower_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn callable_objects_consume_owned_affine_arguments() {
    for resource in ["python.ArrowArray", "python.Buffer[uint8]"] {
        let source = format!(
            "class Consumer:\n    def __call__(self, own value: {resource}) -> None:\n        return None\n\ndef misuse(consumer: Consumer, own value: {resource}) -> None:\n    first: None = consumer(value)\n    second: None = consumer(value)\n    return None\n"
        );
        let errors = lower_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn class_style_calls_reject_protocol_and_instance_methods() {
    for source in [
        "class Duck(Protocol):\n    def quack(self) -> None: ...\n\ndef misuse() -> None:\n    Duck.quack()\n",
        "class Duck:\n    def quack(self) -> None:\n        return None\n\ndef misuse() -> None:\n    Duck.quack()\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn super_calls_apply_defaults_and_reject_missing_methods() {
    lower_ok(
        "class Parent:\n    def value(self, extra: int = 5) -> int:\n        return extra\n\nclass Child(Parent):\n    def value(self, extra: int = 7) -> int:\n        return super().value()\n",
    );

    let errors = lower_errors(
        "class Parent:\n    pass\n\nclass Child(Parent):\n    def misuse(self) -> None:\n        super().missing()\n        return None\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)),
        "{errors:?}"
    );
}

#[test]
fn owned_affine_method_overrides_replace_inherited_conventions() {
    for resource in ["python.ArrowArray", "python.Buffer[uint8]"] {
        let source = format!(
            "class Base:\n    def consume(self, value: {resource}) -> None:\n        return None\n\nclass Child(Base):\n    def consume(self, own value: {resource}) -> None:\n        return None\n\ndef misuse(child: Child, own value: {resource}) -> None:\n    child.consume(value)\n    child.consume(value)\n"
        );
        let errors = lower_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn comprehension_and_generator_bodies_cannot_repeat_affine_moves() {
    for resource in ["python.ArrowArray", "python.Buffer[uint8]"] {
        for expression in [
            "[consume(value) for i in range(3)]",
            "{consume(value) for i in range(3)}",
            "{i: consume(value) for i in range(3)}",
            "(consume(value) for i in range(3))",
        ] {
            let source = format!(
                "def consume(own value: {resource}) -> int:\n    return 1\n\ndef misuse(own value: {resource}) -> None:\n    result = {expression}\n    return None\n"
            );
            let errors = lower_errors(&source);
            assert!(
                errors
                    .iter()
                    .any(|error| error.code == Some(DiagnosticCode::OWN_MOVED_ACROSS_LOOP)),
                "{source}: {errors:?}"
            );
        }
    }
}

#[test]
fn async_loops_and_comprehensions_cannot_repeat_affine_moves() {
    lower_ok(
        "async def readings(own value: str) -> AsyncGenerator[int, GeneratorCloseError]:\n    yield len(value)\n\nasync def valid(own value: str) -> Result[None, GeneratorCloseError]:\n    result = [i async for i in readings(value)]\n    return None\n",
    );

    for resource in ["python.ArrowArray", "python.Buffer[uint8]"] {
        let statement = format!(
            "def consume(own value: {resource}) -> int:\n    return 1\n\nasync def readings() -> AsyncGenerator[int, GeneratorCloseError]:\n    yield 2\n\nasync def misuse(own value: {resource}) -> Result[None, GeneratorCloseError]:\n    async for i in readings():\n        result: int = consume(value)\n    return None\n"
        );
        let errors = lower_errors(&statement);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_MOVED_ACROSS_LOOP)),
            "{statement}: {errors:?}"
        );

        for expression in [
            "[consume(value) async for i in readings()]",
            "[i async for i in readings() if consume(value) > 0]",
            "{consume(value) async for i in readings()}",
            "{i: consume(value) async for i in readings()}",
            "{consume(value): i async for i in readings()}",
        ] {
            let source = format!(
                "def consume(own value: {resource}) -> int:\n    return 1\n\nasync def readings() -> AsyncGenerator[int, GeneratorCloseError]:\n    yield 2\n\nasync def misuse(own value: {resource}) -> Result[None, GeneratorCloseError]:\n    result = {expression}\n    return None\n"
            );
            let errors = lower_errors(&source);
            assert!(
                errors
                    .iter()
                    .any(|error| error.code == Some(DiagnosticCode::OWN_MOVED_ACROSS_LOOP)),
                "{source}: {errors:?}"
            );
        }
    }
}

#[test]
fn affine_consuming_operator_parameters_fail_closed() {
    for resource in ["python.ArrowArray", "python.Buffer[uint8]"] {
        for method in ["__add__", "__pow__", "__getitem__"] {
            let source = format!(
                "class Accumulator:\n    def {method}(self, own value: {resource}) -> int:\n        return 1\n"
            );
            let errors = lower_errors(&source);
            assert!(
                errors
                    .iter()
                    .any(|error| error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)),
                "{source}: {errors:?}"
            );
        }
    }

    lower_ok(
        "class Accumulator:\n    def __pow__(self, own value: list[int]) -> int:\n        return len(value)\n\n    def __getitem__(self, value: python.ArrowArray) -> int:\n        return 1\n",
    );
}

#[test]
fn borrowed_non_affine_operator_parameters_preserve_return_semantics() {
    lower_ok(
        "class Value:\n    number: int\n\nclass Identity:\n    def __add__(self, other: Value) -> Value:\n        return other\n",
    );
}

#[test]
fn inherited_instance_methods_remain_rejected_as_class_style_calls() {
    let errors = lower_errors(
        "class Parent:\n    def consume(self, value: int) -> None:\n        return None\n\nclass Child(Parent):\n    pass\n\ndef misuse() -> None:\n    Child.consume(1)\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)),
        "{errors:?}"
    );
}

#[test]
fn arrow_release_and_owned_transfer_leave_the_resource_moved() {
    for body in [
        "released: None = value.release()\n    names: list[str] = value.capsule_names()",
        "result: int = consume(value)\n    names: list[str] = value.capsule_names()",
    ] {
        let source = format!(
            "{ERROR}\n@python(pkg.consume)\ndef consume(own value: python.ArrowArray) -> Result[int, PythonError]: ...\n\ndef misuse(own value: python.ArrowArray) -> None:\n    {body}\n    return None\n"
        );
        let errors = lower_errors(&source);
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
                    && error.message.contains("moved")
            }),
            "{source}: {errors:?}"
        );
    }
}
