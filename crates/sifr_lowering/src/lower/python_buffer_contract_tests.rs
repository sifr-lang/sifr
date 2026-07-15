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
    let errors = lower_errors(&format!(
        "{ERROR}\ndef overwrite(view: python.Buffer[uint8], value: uint8) -> Result[None, PythonError]:\n    try:\n        written: None = view.write(0, value)\n        return None\n    except PythonError as error:\n        raise error\n"
    ));
    assert!(
        errors
            .iter()
            .any(|error| { error.code == Some(DiagnosticCode::OWN_IMMUTABLE_PARAMETER_MUTATION) }),
        "{errors:?}"
    );

    let source = format!(
        "{ERROR}\ndef overwrite(mut view: python.Buffer[uint8], value: uint8) -> Result[None, PythonError]:\n    try:\n        written: None = view.write(0, value)\n        return None\n    except PythonError as error:\n        raise error\n"
    );
    lower_ok(&source);
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
fn python_buffer_equality_is_rejected_before_codegen() {
    let errors = lower_errors(
        "def same(left: python.Buffer[uint8], right: python.Buffer[uint8]) -> bool:\n    return left == right\n",
    );
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message.contains("cannot compare affine values")
    }));
}
