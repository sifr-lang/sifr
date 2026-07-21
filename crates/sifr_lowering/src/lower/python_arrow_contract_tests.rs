use crate::{lower_module, HirDiagnostic, HirModule};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{PythonArrowSchemaMode, PythonInteropDecoratorKind, PythonParameterKind};
use sifr_python_parser::parse_module;
use sifr_type_system::{OwnershipKind, PythonArrowKind, Type};

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
fn requested_schema_is_required_keyword_only_borrowed_and_not_sent_to_producer() {
    let module = lower_ok(&format!(
        "{ERROR}\n@python.arrow(pkg.make_stream, schema=parameter(requested))\ndef stream(size: int, *, requested: python.ArrowSchema) -> Result[python.ArrowStream, PythonError]: ...\n"
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
    assert!(schema.convention.is_shared_borrow());
}

#[test]
fn opaque_arrow_receiver_retains_self_and_optional_schema_only() {
    let module = lower_ok(&format!(
        "{ERROR}\n@python.opaque(type=pkg.Owner, cleanup=drop)\nclass Owner(NonSend):\n    @python.arrow(Self, schema=parameter(requested))\n    def stream(self, *, requested: python.ArrowSchema) -> Result[python.ArrowStream, PythonError]: ...\n"
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
        "@python.arrow(pkg.make, schema=parameter(requested))\ndef bad(requested: python.ArrowSchema) -> Result[python.ArrowArray, PythonError]: ...",
        "@python.arrow(pkg.make, schema=parameter(requested))\ndef bad(*, requested: python.ArrowSchema = None) -> Result[python.ArrowArray, PythonError]: ...",
        "@python.arrow(pkg.make, schema=parameter(requested))\ndef bad(*, own requested: python.ArrowSchema) -> Result[python.ArrowArray, PythonError]: ...",
        "@python.arrow(pkg.make, schema=parameter(requested))\ndef bad(*, requested: python.ArrowArray) -> Result[python.ArrowArray, PythonError]: ...",
        "@python.arrow(pkg.make, schema=parameter(requested))\ndef bad(*, requested: python.ArrowSchema) -> Result[python.ArrowSchema, PythonError]: ...",
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
