use super::task_scope_calls;
use crate::{
    lower_module, ExternalDefs, HirDiagnostic, HirModule, LoweringOptions,
    PythonBridgeTargetAuthority,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{PythonBufferAccess, PythonBufferLayout, PythonInteropDecoratorKind};
use sifr_python_parser::parse_module;
use sifr_type_system::{FixedIntType, Type};
use std::collections::BTreeMap;

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

const ERROR: &str = r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str
"#;

#[test]
fn import_root_buffer_declaration_retains_typed_protocol_contract() {
    let source = format!(
        "{ERROR}\n@python.buffer(pkg.make_view, access=write, layout=c_contiguous)\ndef view(size: int) -> Result[python.Buffer[uint16], PythonError]: ...\n"
    );
    let module = lower_ok(&source);
    let function = &module.functions[0];
    let declaration = &function.python_interop[0];
    assert_eq!(declaration.kind, PythonInteropDecoratorKind::Buffer);
    assert_eq!(declaration.required_import_root.as_deref(), Some("pkg"));
    let buffer = declaration.buffer.as_ref().expect("buffer contract");
    assert_eq!(buffer.access, PythonBufferAccess::Write);
    assert_eq!(buffer.layout, PythonBufferLayout::CContiguous);
    assert_eq!(buffer.element_type, Type::FixedInt(FixedIntType::U16));
    assert!(matches!(
        function.return_type.resolve_alias(),
        Type::Result(ok, _) if matches!(ok.resolve_alias(), Type::PythonBuffer(element) if element.resolve_alias() == &Type::FixedInt(FixedIntType::U16))
    ));
}

#[test]
fn bridge_buffer_producer_rewrites_to_package_runtime_identity() {
    let parsed = parse_module(&format!(
        "{ERROR}\n@python.buffer(bridge.views.make, access=read, layout=any)\ndef view() -> Result[python.Buffer[uint8], PythonError]: ...\n"
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
                    modules: ["views".to_string()].into_iter().collect(),
                },
            )]),
            ..LoweringOptions::default()
        },
    )
    .expect("resolved bridge buffer should lower");
    let declaration = &lowered.module.functions[0].python_interop[0];

    assert_eq!(
        declaration.target.as_ref().expect("target").dotted(),
        "__sifr_bridge__.p_abc123.views.make"
    );
    assert_eq!(declaration.required_import_root, None);
}

#[test]
fn opaque_receiver_buffer_declaration_uses_exact_self_target() {
    let source = format!(
        "{ERROR}\n@python.opaque(type=pkg.Owner, cleanup=drop)\nclass Owner(NonSend):\n    @python.buffer(Self, access=read, layout=f_contiguous)\n    def view(self) -> Result[python.Buffer[float], PythonError]: ...\n"
    );
    let module = lower_ok(&source);
    let method = &module.classes[1].methods[0];
    let declaration = &method.python_interop[0];
    assert_eq!(
        declaration.target.as_ref().expect("target").segments,
        ["Self"]
    );
    let buffer = declaration.buffer.as_ref().expect("buffer contract");
    assert_eq!(buffer.access, PythonBufferAccess::Read);
    assert_eq!(buffer.layout, PythonBufferLayout::FContiguous);
    assert_eq!(buffer.element_type, Type::Float);
}

#[test]
fn opaque_receiver_buffer_requires_immutable_borrowed_self() {
    for receiver in ["own self", "mut self", "own mut self"] {
        let source = format!(
            "{ERROR}\n@python.opaque(type=pkg.Owner, cleanup=drop)\nclass Owner(NonSend):\n    @python.buffer(Self, access=read, layout=any)\n    def view({receiver}) -> Result[python.Buffer[uint8], PythonError]: ...\n"
        );
        let errors = lower_errors(&source);
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                    && error.message.contains("immutable borrowed `self`")
            }),
            "{receiver}: {errors:?}"
        );
    }
}

#[test]
fn opaque_receiver_buffer_rejects_writable_self_without_owner_freezing() {
    let source = format!(
        "{ERROR}\n@python.opaque(type=pkg.Owner, cleanup=drop)\nclass Owner(NonSend):\n    @python.buffer(Self, access=write, layout=any)\n    def view(self) -> Result[python.Buffer[uint8], PythonError]: ...\n"
    );
    let errors = lower_errors(&source);
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error
                    .message
                    .contains("cannot exclusively freeze its opaque owner")
        }),
        "{errors:?}"
    );
}

#[test]
fn buffer_declarations_and_methods_reject_shadow_python_error_shapes() {
    let shadow = r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str
    code: int
"#;
    let declaration = lower_errors(&format!(
        "{shadow}\n@python.buffer(pkg.make, access=read, layout=any)\ndef bad() -> Result[python.Buffer[uint8], PythonError]: ...\n"
    ));
    assert!(
        declaration.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error
                    .message
                    .contains("canonical `PythonError` field contract")
        }),
        "{declaration:?}"
    );

    let method = lower_errors(&format!(
        "{shadow}\ndef bad(view: python.Buffer[uint8]) -> None:\n    value: Result[uint8, PythonError] = view.read(0)\n"
    ));
    assert!(
        method.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error
                    .message
                    .contains("python.Buffer methods require the canonical")
        }),
        "{method:?}"
    );
}

#[test]
fn infallible_buffer_metadata_does_not_require_python_error_in_scope() {
    lower_ok("def length(view: python.Buffer[uint8]) -> int:\n    return view.length()\n");
}

#[test]
fn buffer_policy_and_return_contract_fail_with_pyzc_0001() {
    for declaration in [
        "@python.buffer(pkg.make, access=copy, layout=any)\ndef bad() -> Result[python.Buffer[uint8], PythonError]: ...",
        "@python.buffer(pkg.make, access=read, layout=strided)\ndef bad() -> Result[python.Buffer[uint8], PythonError]: ...",
        "@python.buffer(pkg.make, access=read, layout=any)\ndef bad() -> Result[bytes, PythonError]: ...",
    ] {
        let errors = lower_errors(&format!("{ERROR}\n{declaration}\n"));
        assert!(errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)));
    }
}

#[test]
fn buffer_declaration_rejects_incomplete_async_and_non_opaque_forms() {
    for declaration in [
        "@python.buffer(pkg.make, access=read)\ndef bad() -> Result[python.Buffer[uint8], PythonError]: ...",
        "@python.buffer(pkg.make, layout=any)\ndef bad() -> Result[python.Buffer[uint8], PythonError]: ...",
        "@python.buffer(pkg.make, access=read, layout=any, cache=read)\ndef bad() -> Result[python.Buffer[uint8], PythonError]: ...",
        "@python.buffer(pkg.make, access=read, layout=any)\nasync def bad() -> Result[python.Buffer[uint8], PythonError]: ...",
        "class Owner:\n    @python.buffer(Self, access=read, layout=any)\n    def bad(self) -> Result[python.Buffer[uint8], PythonError]: ...",
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
fn python_buffer_rejects_non_closed_element_types() {
    for element in ["int", "str", "bytes", "list[uint8]"] {
        let errors = lower_errors(&format!(
            "{ERROR}\ndef bad(view: python.Buffer[{element}]) -> None:\n    return None\n"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)),
            "{element}: {errors:?}"
        );
    }
}

#[test]
fn release_consumes_buffer_and_borrowed_release_is_rejected() {
    let moved = lower_errors(&format!(
        "{ERROR}\ndef consume(own view: python.Buffer[uint8]) -> Result[None, PythonError]:\n    try:\n        released: None = view.release()\n        value: uint8 = view.read(0)\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(moved
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)));

    let borrowed = lower_errors(&format!(
        "{ERROR}\ndef consume(view: python.Buffer[uint8]) -> Result[None, PythonError]:\n    try:\n        released: None = view.release()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(borrowed
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)));

    let field = lower_errors(&format!(
        "{ERROR}\nclass Holder:\n    view: python.Buffer[uint8]\n\ndef release_field(holder: Holder) -> Result[None, PythonError]:\n    try:\n        released: None = holder.view.release()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(
        field
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)),
        "{field:?}"
    );
}

#[test]
fn writable_access_requires_exclusive_parameter_borrow() {
    let direct = lower_errors(&format!(
        "{ERROR}\ndef overwrite(view: python.Buffer[uint8], value: uint8) -> Result[None, PythonError]:\n    try:\n        written: None = view.write(0, value)\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(direct
        .iter()
        .any(|error| { error.code == Some(DiagnosticCode::OWN_IMMUTABLE_PARAMETER_MUTATION) }));

    for function in [
        "class Holder:\n    view: python.Buffer[uint8]\n\ndef overwrite(holder: Holder, value: uint8) -> Result[None, PythonError]:\n    try:\n        written: None = holder.view.write(0, value)\n        return None\n    except PythonError as error:\n        raise error",
        "def overwrite(views: list[python.Buffer[uint8]], value: uint8) -> Result[None, PythonError]:\n    try:\n        written: None = views[0].write(0, value)\n        return None\n    except PythonError as error:\n        raise error",
    ] {
        let errors = lower_errors(&format!("{ERROR}\n{function}\n"));
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                    && error.message.contains("affine Python buffer")
            }),
            "{function}: {errors:?}"
        );
    }

    lower_ok(&format!(
        "{ERROR}\ndef overwrite(mut view: python.Buffer[uint8], value: uint8) -> Result[None, PythonError]:\n    try:\n        written: None = view.write(0, value)\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
}

#[test]
fn python_buffer_is_rejected_at_sendability_boundaries() {
    let ty = Type::PythonBuffer(Box::new(Type::FixedInt(FixedIntType::U8)));
    assert_eq!(
        task_scope_calls::non_send_reason(&ty).as_deref(),
        Some("Python buffer resources are non-send")
    );
}

#[test]
fn same_basename_class_identity_does_not_hide_nested_buffer_sendability() {
    let buffer = Type::PythonBuffer(Box::new(Type::FixedInt(FixedIntType::U8)));
    let inner = Type::Class {
        identity: Some("inner.Root".to_string()),
        type_args: Vec::new(),
        name: "Root".to_string(),
        fields: vec![("view".to_string(), buffer)],
        methods: Vec::new(),
        parent_class: None,
    };
    let outer = Type::Class {
        identity: Some("outer.Root".to_string()),
        type_args: Vec::new(),
        name: "Root".to_string(),
        fields: vec![("inner".to_string(), inner)],
        methods: Vec::new(),
        parent_class: None,
    };

    let reason = task_scope_calls::non_send_reason(&outer)
        .expect("nested Python buffer should make the outer class non-send");
    assert!(reason.contains("Python buffer resources are non-send"));
}

#[test]
fn python_buffer_equality_is_rejected_before_codegen() {
    let errors = lower_errors(
        "def same(left: python.Buffer[uint8], right: python.Buffer[uint8]) -> bool:\n    return left == right\n",
    );
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message.contains("cannot compare affine values")
    }));
}

#[test]
fn affine_buffer_aggregate_projections_are_rejected_before_codegen() {
    for source in [
        "def first(values: list[python.Buffer[uint8]]) -> python.Buffer[uint8] | None:\n    return values[0]\n",
        "def first(values: tuple[python.Buffer[uint8]]) -> python.Buffer[uint8]:\n    return values[0]\n",
        "class Holder:\n    value: python.Buffer[uint8]\n\ndef take(holder: Holder) -> python.Buffer[uint8]:\n    return holder.value\n",
        "def visit(values: list[python.Buffer[uint8]]) -> None:\n    for value in values:\n        print(value.length())\n",
    ] {
        let errors = lower_errors(source);
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                || error.message.contains("affine Python buffer")
        }), "{source}: {errors:?}");
    }
}

#[test]
fn affine_buffer_literal_insertion_consumes_the_source() {
    for insertion in [
        "values: list[python.Buffer[uint8]] = [view]",
        "values: tuple[python.Buffer[uint8]] = (view,)",
    ] {
        let errors = lower_errors(&format!(
            "{ERROR}\ndef pack(own view: python.Buffer[uint8]) -> None:\n    {insertion}\n    print(view.length())\n"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{insertion}: {errors:?}"
        );
    }
}

#[test]
fn affine_buffer_collection_capabilities_are_rejected_before_codegen() {
    for source in [
        "def duplicate(values: list[python.Buffer[uint8]]) -> list[python.Buffer[uint8]]:\n    return values.copy()\n",
        "def concatenate(left: list[python.Buffer[uint8]], right: list[python.Buffer[uint8]]) -> list[python.Buffer[uint8]]:\n    return left + right\n",
        "def search(values: list[python.Buffer[uint8]], value: python.Buffer[uint8]) -> bool:\n    return values.contains(value)\n",
        "def duplicate_values(values: dict[str, python.Buffer[uint8]]) -> dict[str, python.Buffer[uint8]]:\n    return values.copy()\n",
        "def make_set(own value: python.Buffer[uint8]) -> set[python.Buffer[uint8]]:\n    return {value}\n",
        "def make_dict(own key: python.Buffer[uint8]) -> dict[python.Buffer[uint8], int]:\n    return {key: 1}\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn affine_buffer_collection_insertion_consumes_the_source() {
    for source in [
        "def pack(own view: python.Buffer[uint8]) -> None:\n    values: list[python.Buffer[uint8]] = []\n    values.append(view)\n    print(view.length())\n",
        "def pack(own left: python.Buffer[uint8], own right: python.Buffer[uint8], flag: bool) -> None:\n    values: list[python.Buffer[uint8]] = []\n    values.append(left if flag else right)\n    print(left.length())\n    print(right.length())\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{source}: {errors:?}"
        );
    }
    let borrowed = lower_errors(
        "def pack(view: python.Buffer[uint8]) -> None:\n    values: list[python.Buffer[uint8]] = []\n    values.append(view)\n",
    );
    assert!(borrowed.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
            && error.message.contains("borrowed affine Python buffer")
    }));
}

#[test]
fn affine_buffer_storage_assignments_transfer_owned_values() {
    for source in [
        "def replace(mut values: list[python.Buffer[uint8]], own view: python.Buffer[uint8]) -> None:\n    values[0] = view\n    print(view.length())\n",
        "def replace(mut values: dict[str, python.Buffer[uint8]], own view: python.Buffer[uint8]) -> None:\n    values[\"x\"] = view\n    print(view.length())\n",
        "def replace(mut values: list[list[python.Buffer[uint8]]], own view: python.Buffer[uint8]) -> None:\n    values[0][0] = view\n    print(view.length())\n",
        "class Holder:\n    view: python.Buffer[uint8]\n\ndef replace(mut holder: Holder, own view: python.Buffer[uint8]) -> None:\n    holder.view = view\n    print(view.length())\n",
        "class Inner:\n    view: python.Buffer[uint8]\n\nclass Outer:\n    inner: Inner\n\ndef replace(mut holder: Outer, own view: python.Buffer[uint8]) -> None:\n    holder.inner.view = view\n    print(view.length())\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{source}: {errors:?}"
        );
    }
    let borrowed = lower_errors(
        "def replace(mut values: list[python.Buffer[uint8]], view: python.Buffer[uint8]) -> None:\n    values[0] = view\n",
    );
    assert!(borrowed.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
            && error.message.contains("borrowed affine Python buffer")
    }));
}

#[test]
fn affine_buffer_constructor_moves_are_tracked_and_walrus_is_rejected() {
    let constructor = lower_errors(
        "class Holder:\n    view: python.Buffer[uint8]\n\ndef pack(own view: python.Buffer[uint8]) -> None:\n    holder: Holder = Holder(view)\n    print(view.length())\n",
    );
    assert!(
        constructor
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
        "{constructor:?}"
    );

    let walrus = lower_errors(
        "def alias(own view: python.Buffer[uint8]) -> None:\n    retained: python.Buffer[uint8] = (moved := view)\n    print(view.length())\n",
    );
    assert!(
        walrus.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error.message.contains("walrus target 'moved'")
        }),
        "{walrus:?}"
    );
}

#[test]
fn affine_buffer_reusable_callable_captures_are_rejected() {
    for source in [
        "def keep(own view: python.Buffer[uint8]) -> Callable[[], python.Buffer[uint8]]:\n    return lambda: view\n",
        "def keep(own view: python.Buffer[uint8]) -> Callable[[], python.Buffer[uint8]]:\n    def inner() -> python.Buffer[uint8]:\n        return view\n    return inner\n",
    ] {
        let errors = lower_errors(source);
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error.message.contains("reusable callables")
        }), "{source}: {errors:?}");
    }
}

#[test]
fn affine_buffer_comprehension_moves_are_rejected() {
    for source in [
        "def capture(own view: python.Buffer[uint8]) -> list[python.Buffer[uint8]]:\n    return [view for number in [1, 2]]\n",
        "def capture_set(own view: python.Buffer[uint8]) -> set[python.Buffer[uint8]]:\n    return {view for number in [1, 2]}\n",
        "def capture_dict(own view: python.Buffer[uint8]) -> dict[int, python.Buffer[uint8]]:\n    return {number: view for number in [1, 2]}\n",
        "def iterate(values: list[python.Buffer[uint8]]) -> list[int]:\n    return [value.length() for value in values]\n",
        "def generate(own view: python.Buffer[uint8]) -> Iterator[python.Buffer[uint8]]:\n    return (view for number in [1, 2])\n",
    ] {
        let errors = lower_errors(source);
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                || error.code == Some(DiagnosticCode::FLOW_INVALID_ITERATION)
        }), "{source}: {errors:?}");
    }
}

#[test]
fn affine_buffer_iterator_builtins_are_rejected_before_codegen() {
    for source in [
        "def project(values: list[python.Buffer[uint8]]) -> Iterator[python.Buffer[uint8]]:\n    return iter(values)\n",
        "def project(values: Iterator[python.Buffer[uint8]]) -> python.Buffer[uint8] | None:\n    return next(values)\n",
        "def project(values: list[python.Buffer[uint8]]) -> Iterator[tuple[python.Buffer[uint8], int]]:\n    return zip(values, [1])\n",
        "def project(values: list[python.Buffer[uint8]]) -> Iterator[python.Buffer[uint8]]:\n    return reversed(values)\n",
        "def project(values: list[python.Buffer[uint8]]) -> Iterator[tuple[int, python.Buffer[uint8]]]:\n    return enumerate(values)\n",
        "def keep(value: python.Buffer[uint8]) -> bool:\n    return True\n\ndef project(values: list[python.Buffer[uint8]]) -> Iterator[python.Buffer[uint8]]:\n    return filter(keep, values)\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                    && error.message.contains("cannot project")
            }),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn affine_buffer_yield_is_consuming_and_borrowed_generator_inputs_are_rejected() {
    let reused = lower_errors(
        "def generate(own view: python.Buffer[uint8]) -> Iterator[python.Buffer[uint8]]:\n    yield view\n    print(view.length())\n",
    );
    assert!(
        reused
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
        "{reused:?}"
    );

    let borrowed = lower_errors(
        "def generate(view: python.Buffer[uint8]) -> Iterator[python.Buffer[uint8]]:\n    yield view\n",
    );
    assert!(
        borrowed.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error.message.contains("cannot borrow affine Python buffer")
        }),
        "{borrowed:?}"
    );
}

#[test]
fn borrowed_affine_buffer_parameter_reassignment_is_rejected_before_clone_codegen() {
    let errors = lower_errors(
        "def replace(mut view: python.Buffer[uint8], own replacement: python.Buffer[uint8]) -> None:\n    view = replacement\n",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error.message.contains("mutable parameter shadowing")
        }),
        "{errors:?}"
    );
}

#[test]
fn borrowed_affine_buffers_cannot_enter_owned_calls_or_aggregate_returns() {
    for source in [
        "def sink(own value: python.Buffer[uint8]) -> None:\n    return None\n\ndef escape(view: python.Buffer[uint8]) -> None:\n    sink(view)\n",
        "class Holder:\n    view: python.Buffer[uint8]\n\ndef escape(view: python.Buffer[uint8]) -> Holder:\n    return Holder(view)\n",
        "def escape(view: python.Buffer[uint8]) -> list[python.Buffer[uint8]]:\n    return [view]\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                    && error.message.contains("borrowed affine Python buffer")
            }),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn affine_membership_chained_assignment_star_unpack_and_match_are_rejected() {
    for source in [
        "def membership(view: python.Buffer[uint8], values: list[python.Buffer[uint8]]) -> bool:\n    return view in values\n",
        "def non_membership(view: python.Buffer[uint8], values: list[python.Buffer[uint8]]) -> bool:\n    return view not in values\n",
        "def chain(own view: python.Buffer[uint8]) -> None:\n    left = right = view\n",
        "def unpack(own values: list[python.Buffer[uint8]]) -> None:\n    first, *rest = values\n",
        "def inspect(own view: python.Buffer[uint8]) -> None:\n    match view:\n        case _:\n            print(1)\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn equality_and_unpack_capabilities_reject_non_rust_traits() {
    for source in [
        "def member(value: Any, values: list[Any]) -> bool:\n    return value in values\n",
        "class Holder:\n    callback: Callable[[int], int]\n\ndef member(value: Holder, values: list[Holder]) -> bool:\n    return value in values\n",
        "def same(left: python.Buffer[uint8], right: python.Buffer[uint8]) -> bool:\n    return left is right\n",
        "def unpack(values: list[Any]) -> None:\n    first, *rest = values\n",
        "class Holder:\n    callback: Callable[[int], int]\n\ndef unpack(values: list[Holder]) -> None:\n    first, *rest = values\n",
        "class Holder:\n    callback: Callable[[int], int]\n\ndef unpack(values: tuple[Holder, int]) -> Holder:\n    first, number = values\n    return first\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors.iter().any(|error| {
                matches!(
                    error.code,
                    Some(
                        DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR
                            | DiagnosticCode::PYZC_INVALID_DECLARATION
                    )
                )
            }),
            "{source}: {errors:?}"
        );
    }

    lower_ok(
        "class Holder:\n    callback: Callable[[int], int]\n\ndef unpack(own values: tuple[Holder, int]) -> Holder:\n    first, number = values\n    return first\n",
    );
    lower_ok(
        "def unpack(values: tuple[str, int]) -> str:\n    first, number = values\n    return first\n",
    );
}

#[test]
fn nested_async_generator_affine_capture_is_rejected() {
    let errors = lower_errors(
        "def make(own view: python.Buffer[uint8]) -> AsyncGenerator[int, GeneratorCloseError]:\n    async def generate() -> AsyncGenerator[int, GeneratorCloseError]:\n        yield view.length()\n    return generate()\n",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error.message.contains("capture 'view'")
        }),
        "{errors:?}"
    );
}

#[test]
fn tuple_unpack_consumes_owned_affine_source_and_rejects_borrowed_source() {
    let moved = lower_errors(
        "def unpack(own pair: tuple[python.Buffer[uint8], int]) -> None:\n    view, number = pair\n    print(pair)\n",
    );
    assert!(
        moved
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
        "{moved:?}"
    );

    let borrowed = lower_errors(
        "def unpack(pair: tuple[python.Buffer[uint8], int]) -> None:\n    view, number = pair\n",
    );
    assert!(borrowed.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
            && error.message.contains("borrowed affine Python buffer")
    }));
}

#[test]
fn affine_buffer_conditional_assignment_consumes_every_candidate() {
    let errors = lower_errors(
        "def choose(own left: python.Buffer[uint8], own right: python.Buffer[uint8], flag: bool) -> None:\n    selected: python.Buffer[uint8] = left if flag else right\n    print(left.length())\n    print(right.length())\n",
    );
    assert!(
        errors
            .iter()
            .filter(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE))
            .count()
            >= 2,
        "{errors:?}"
    );
}

#[test]
fn affine_and_dynamic_generic_collection_capabilities_fail_during_lowering() {
    let affine = lower_errors(
        "def duplicate[T](values: list[T]) -> list[T]:\n    return values.copy()\n\ndef use(values: list[python.Buffer[uint8]]) -> list[python.Buffer[uint8]]:\n    return duplicate(values)\n",
    );
    assert!(
        affine.iter().any(|error| {
            error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
                && error.message.contains("generic function 'duplicate'")
        }),
        "{affine:?}"
    );

    for source in [
        "def duplicate(values: list[Any]) -> list[Any]:\n    return values.copy()\n",
        "def duplicate(values: set[Any]) -> set[Any]:\n    return values.copy()\n",
        "def duplicate(values: dict[Any, int]) -> dict[Any, int]:\n    return values.copy()\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors.iter().any(|error| {
                matches!(
                    error.code,
                    Some(DiagnosticCode::TYPE_MISMATCH)
                        | Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
                )
            }),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn dynamic_collection_clone_and_projection_capabilities_fail_during_lowering() {
    for source in [
        "def concatenate(left: list[Any], right: list[Any]) -> list[Any]:\n    return left + right\n",
        "def repeat(values: list[Any], count: int) -> list[Any]:\n    return values * count\n",
        "def repeat(values: list[list[Any]], count: int) -> list[list[Any]]:\n    return values * count\n",
        "def repeat(count: int) -> None:\n    values: list[Any] = []\n    values *= count\n",
        "def order(values: list[Any]) -> list[Any]:\n    return sorted(values)\n",
        "def total(values: list[Any]) -> Any:\n    return sum(values)\n",
        "def minimum(left: Any, right: Any) -> Any:\n    return min(left, right)\n",
        "def maximum(left: Any, right: Any) -> Any:\n    return max(left, right)\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                    && (error.message.contains("statically known")
                        || error.message.contains("Clone-capable"))
            }),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn generic_variadic_min_max_requires_concrete_total_order() {
    for builtin in ["min", "max"] {
        let errors = lower_errors(&format!(
            "def choose[T](left: T, right: T) -> T:\n    return {builtin}(left, right)\n"
        ));
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message.contains("concrete total-order capability")
        }));
    }
}

#[test]
fn affine_buffer_copying_constructors_and_sorting_are_rejected() {
    for source in [
        "def duplicate(values: list[python.Buffer[uint8]]) -> list[python.Buffer[uint8]]:\n    return list(values)\n",
        "def duplicate(values: dict[str, python.Buffer[uint8]]) -> dict[str, python.Buffer[uint8]]:\n    return dict(values)\n",
        "def duplicate(values: dict[str, python.Buffer[uint8]], own extra: python.Buffer[uint8]) -> dict[str, python.Buffer[uint8]]:\n    return dict(values, extra=extra)\n",
        "def repeat(values: list[python.Buffer[uint8]], count: int) -> list[python.Buffer[uint8]]:\n    return values * count\n",
        "def repeat(count: int) -> None:\n    values: list[python.Buffer[uint8]] = []\n    values *= count\n",
        "def duplicate() -> None:\n    values: list[python.Buffer[uint8]] = []\n    values += values\n",
        "def duplicate(own left: list[python.Buffer[uint8]], own other: list[python.Buffer[uint8]], flag: bool) -> None:\n    values: list[python.Buffer[uint8]] = left\n    values += values if flag else other\n",
        "def duplicate(own left: list[python.Buffer[uint8]]) -> None:\n    values: list[python.Buffer[uint8]] = left\n    values += (alias := values)\n",
        "def order(values: list[python.Buffer[uint8]]) -> list[python.Buffer[uint8]]:\n    return sorted(values)\n",
        "def minimum(values: list[python.Buffer[uint8]]) -> python.Buffer[uint8] | None:\n    return min(values)\n",
        "def maximum(values: list[python.Buffer[uint8]]) -> python.Buffer[uint8] | None:\n    return max(values)\n",
        "def minimum(own left: python.Buffer[uint8], own right: python.Buffer[uint8]) -> python.Buffer[uint8]:\n    return min(left, right)\n",
        "def maximum(own left: python.Buffer[uint8], own right: python.Buffer[uint8]) -> python.Buffer[uint8]:\n    return max(left, right)\n",
        "async def emit_one(own view: python.Buffer[uint8]) -> AsyncGenerator[python.Buffer[uint8], str]:\n    yield view\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn affine_constructor_and_list_augassign_moves_are_tracked() {
    for source in [
        "def pack(own view: python.Buffer[uint8]) -> None:\n    retained: tuple[python.Buffer[uint8]] = tuple((view,))\n    print(view.length())\n",
        "def pack(own view: python.Buffer[uint8]) -> None:\n    retained: dict[str, python.Buffer[uint8]] = dict(view=view)\n    print(view.length())\n",
        "def merge(own right: list[python.Buffer[uint8]]) -> None:\n    left: list[python.Buffer[uint8]] = []\n    left += right\n    print(len(right))\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{source}: {errors:?}"
        );
    }
}
