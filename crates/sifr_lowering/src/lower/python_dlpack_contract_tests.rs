use crate::{lower_module, HirDiagnostic, HirModule};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    PythonDlpackDevice, PythonDlpackStreamMode, PythonInteropDecoratorKind, PythonParameterKind,
};
use sifr_python_parser::parse_module;
use sifr_type_system::{OwnershipKind, Type};

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
fn dlpack_annotations_are_closed_affine_resources() {
    let module = lower_ok(
        "def keep(tensor: python.DlpackTensor[float], stream: python.DlpackStream) -> None:\n    return None\n",
    );
    let tensor = &module.functions[0].params[0].ty;
    let stream = &module.functions[0].params[1].ty;
    assert_eq!(tensor, &Type::PythonDlpackTensor(Box::new(Type::Float)));
    assert_eq!(stream, &Type::PythonDlpackStream);
    for resource in [tensor, stream] {
        assert_eq!(resource.ownership(), OwnershipKind::Move);
        assert!(resource.contains_affine_resource());
        assert!(!resource.supports_derived_clone());
        assert!(!resource.supports_structural_equality());
    }
}

#[test]
fn cpu_tensor_declaration_activates_without_stream() {
    let module = lower_ok(&format!(
        "{ERROR}\n@python.dlpack(pkg.make, device=cpu, stream=none)\ndef make(size: int) -> Result[python.DlpackTensor[float], PythonError]: ...\n"
    ));
    let declaration = &module.functions[0].python_interop[0];
    assert_eq!(declaration.kind, PythonInteropDecoratorKind::Dlpack);
    assert_eq!(declaration.required_import_root.as_deref(), Some("pkg"));
    let dlpack = declaration.dlpack.as_ref().expect("DLPack contract");
    assert_eq!(dlpack.device, PythonDlpackDevice::Cpu);
    assert_eq!(dlpack.stream, PythonDlpackStreamMode::None);
    assert_eq!(dlpack.element_type, Some(Type::Float));
}

#[test]
fn cuda_tensor_uses_required_keyword_only_borrowed_stream() {
    let module = lower_ok(&format!(
        "{ERROR}\n@python.dlpack(pkg.make, device=cuda, stream=parameter(consumer_stream))\ndef make(size: int, *, consumer_stream: python.DlpackStream) -> Result[python.DlpackTensor[float], PythonError]: ...\n"
    ));
    let function = &module.functions[0];
    let declaration = &function.python_interop[0];
    let dlpack = declaration.dlpack.as_ref().expect("DLPack contract");
    assert_eq!(dlpack.device, PythonDlpackDevice::Cuda);
    assert!(matches!(
        &dlpack.stream,
        PythonDlpackStreamMode::Parameter { name, .. } if name == "consumer_stream"
    ));
    assert_eq!(declaration.parameters.len(), 1);
    assert_eq!(declaration.parameters[0].name, "size");
    assert_eq!(
        declaration.parameters[0].kind,
        PythonParameterKind::Positional
    );
    assert!(function.params[1].convention.is_shared_borrow());
}

#[test]
fn stream_declaration_requires_concrete_device() {
    let module = lower_ok(&format!(
        "{ERROR}\n@python.dlpack.stream(pkg.stream, device=cuda)\ndef stream(device_id: int) -> Result[python.DlpackStream, PythonError]: ...\n"
    ));
    let declaration = &module.functions[0].python_interop[0];
    assert_eq!(declaration.kind, PythonInteropDecoratorKind::DlpackStream);
    assert_eq!(
        declaration.dlpack.as_ref().expect("DLPack contract").device,
        PythonDlpackDevice::Cuda
    );
}

#[test]
fn dlpack_surface_diagnostics_identify_invalid_stream_and_missing_element_type() {
    let stream_errors = lower_errors(&format!(
        "{ERROR}\n@python.dlpack.stream(pkg.stream, device=cuda, stream=none)\ndef stream() -> Result[python.DlpackStream, PythonError]: ...\n"
    ));
    assert!(stream_errors.iter().any(|error| error
        .message
        .contains("does not accept a `stream` argument")));

    let annotation_errors =
        lower_errors("def bad(value: python.DlpackTensor) -> None:\n    return None\n");
    assert!(annotation_errors
        .iter()
        .any(|error| error.message.contains("requires exactly 1 element type")));
}

#[test]
fn dlpack_declarations_reject_invalid_policy_and_signature_shapes() {
    for source in [
        "@python.dlpack(pkg.make, device=cpu)\ndef bad() -> Result[python.DlpackTensor[float], PythonError]: ...",
        "@python.dlpack(pkg.make, device=cuda, stream=none)\ndef bad() -> Result[python.DlpackTensor[float], PythonError]: ...",
        "@python.dlpack(pkg.make, device=any, stream=none)\ndef bad() -> Result[python.DlpackTensor[float], PythonError]: ...",
        "@python.dlpack(pkg.make, device=cuda, stream=parameter(stream))\ndef bad(stream: python.DlpackStream) -> Result[python.DlpackTensor[float], PythonError]: ...",
        "@python.dlpack(pkg.make, device=cuda, stream=parameter(stream))\ndef bad(*, own stream: python.DlpackStream) -> Result[python.DlpackTensor[float], PythonError]: ...",
        "@python.dlpack(pkg.make, device=cpu, stream=none)\ndef bad() -> Result[bytes, PythonError]: ...",
        "@python.dlpack.stream(pkg.stream, device=any)\ndef bad() -> Result[python.DlpackStream, PythonError]: ...",
        "@python.dlpack.stream(pkg.stream, device=cuda)\ndef bad() -> Result[int, PythonError]: ...",
        "@python.dlpack(pkg.make, device=cpu, stream=none)\nasync def bad() -> Result[python.DlpackTensor[float], PythonError]: ...",
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
fn dlpack_consumer_requires_owned_one_shot_transfer() {
    let errors = lower_errors(&format!(
        "{ERROR}\n@python(pkg.consume)\ndef consume(value: python.DlpackTensor[float]) -> Result[int, PythonError]: ...\n"
    ));
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("must transfer ownership with plain `own`")),
        "{errors:?}"
    );
    lower_ok(&format!(
        "{ERROR}\n@python(pkg.consume)\ndef consume(own value: python.DlpackTensor[float]) -> Result[int, PythonError]: ...\n"
    ));
}

#[test]
fn dlpack_resources_cannot_be_implicitly_copied_or_consumed_twice() {
    let duplicate = lower_errors(
        "def duplicate(value: python.DlpackTensor[int64]) -> tuple[python.DlpackTensor[int64], python.DlpackTensor[int64]]:\n    return (value, value)\n",
    );
    assert!(duplicate.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYZC_INVALID_DECLARATION)
            && error.message.contains("borrowed affine Python resource")
    }));

    for call in ["consume2(value, value)", "consume2(value, take(value))"] {
        let errors = lower_errors(&format!(
            "{ERROR}\ndef take(own value: python.DlpackTensor[int64]) -> python.DlpackTensor[int64]:\n    return value\n\n@python(pkg.consume2)\ndef consume2(own left: python.DlpackTensor[int64], own right: python.DlpackTensor[int64]) -> Result[int, PythonError]: ...\n\ndef misuse(own value: python.DlpackTensor[int64]) -> None:\n    result: int = {call}\n    return None\n"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{call}: {errors:?}"
        );
    }
}

#[test]
fn dlpack_release_and_owned_calls_leave_the_tensor_moved() {
    for body in [
        "released: None = value.release()\n    shape: list[int] = value.shape()",
        "first: int = consume(value)\n    second: int = consume(value)",
    ] {
        let errors = lower_errors(&format!(
            "{ERROR}\n@python(pkg.consume)\ndef consume(own value: python.DlpackTensor[int64]) -> Result[int, PythonError]: ...\n\ndef misuse(own value: python.DlpackTensor[int64]) -> None:\n    {body}\n    return None\n"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{body}: {errors:?}"
        );
    }
}

#[test]
fn dlpack_constructor_callable_and_loop_moves_are_checked() {
    for source in [
        "class Holder:\n    value: python.DlpackTensor[int64]\n\n    def __init__(self, own value: python.DlpackTensor[int64]):\n        self.value = value\n\ndef misuse(own value: python.DlpackTensor[int64]) -> None:\n    first: Holder = Holder(value)\n    second: Holder = Holder(value)\n",
        "class Consumer:\n    def __call__(self, own value: python.DlpackTensor[int64]) -> None:\n        return None\n\ndef misuse(consumer: Consumer, own value: python.DlpackTensor[int64]) -> None:\n    consumer(value)\n    consumer(value)\n",
    ] {
        let errors = lower_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)),
            "{source}: {errors:?}"
        );
    }

    let loop_errors = lower_errors(
        "def consume(own value: python.DlpackTensor[int64]) -> int:\n    return 1\n\ndef misuse(own value: python.DlpackTensor[int64]) -> None:\n    results = [consume(value) for index in range(3)]\n    return None\n",
    );
    assert!(loop_errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::OWN_MOVED_ACROSS_LOOP)));
}
