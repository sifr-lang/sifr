use crate::python_interop_direct::{python_interop_function_body, python_interop_method_body};
use crate::{generate_rust, render_stmts};
use ruff_text_size::TextRange;
use sifr_ir::{
    HirFunction, MethodKind, PythonBufferAccess, PythonBufferDeclaration, PythonBufferLayout,
    PythonInteropDeclaration, PythonInteropDecoratorKind, PythonInteropEffect, PythonTargetPath,
};
use sifr_type_system::{FixedIntType, Type};

const ERROR: &str = r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str
"#;

#[test]
fn source_buffer_declaration_and_methods_generate_parseable_rust() {
    let source = format!(
        "{ERROR}\n@python.buffer(builtins.bytearray, access=write, layout=c_contiguous)\ndef writable_view(value: bytes) -> Result[python.Buffer[uint8], PythonError]: ...\n\ndef exercise(mut view: python.Buffer[uint8]) -> Result[None, PythonError]:\n    try:\n        value: uint8 = view.read(0)\n        written: None = view.write(0, value)\n        copied: list[uint8] = view.copy_slice(0, view.length())\n        return None\n    except PythonError as error:\n        raise error\n"
    );
    let parsed = sifr_python_parser::parse_module(&source).expect("source should parse");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("source should lower");
    let rust = generate_rust(&lowered.module);

    assert!(
        rust.contains("sifr_stdlib::python::PythonBuffer::<u8>::acquire"),
        "{rust}"
    );
    assert!(
        rust.contains("sifr_runtime::interop::Handle::new(__sifr_python_result)"),
        "{rust}"
    );
    assert!(rust.contains("view.read(0_i64).map_err("), "{rust}");
    assert!(rust.contains("view.write(0_i64, value).map_err("), "{rust}");
    assert!(
        rust.contains("view.copy_slice(0_i64, view.length())"),
        "{rust}"
    );
    syn::parse_file(&rust).expect("generated buffer Rust should parse");
}

#[test]
fn affine_list_append_moves_buffer_without_cloning() {
    let source = "def pack(own view: python.Buffer[uint8]) -> None:\n    values: list[python.Buffer[uint8]] = []\n    values.append(view)\n";
    let parsed = sifr_python_parser::parse_module(source).expect("source should parse");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("source should lower");
    let rust = generate_rust(&lowered.module);

    assert!(rust.contains("values.push(view)"), "{rust}");
    assert!(!rust.contains("values.push(view.clone())"), "{rust}");
    syn::parse_file(&rust).expect("generated affine append Rust should parse");
}

#[test]
fn top_level_buffer_wrapper_acquires_typed_writable_export() {
    let function = buffer_function(
        vec!["builtins", "bytearray"],
        PythonBufferAccess::Write,
        PythonBufferLayout::CContiguous,
    );

    let body = python_interop_function_body(&function, &Default::default())
        .expect("buffer wrapper should lower");
    let rendered = render_stmts(&body);

    assert!(rendered.contains("sifr_runtime::python::resolve_target"));
    assert!(rendered.contains("sifr_runtime::python::call_object_owned"));
    assert!(rendered.contains("sifr_stdlib::python::PythonBuffer::<u16>::acquire"));
    assert!(rendered.contains("sifr_runtime::python::PythonBufferAccess::Write"));
    assert!(rendered.contains("sifr_runtime::python::PythonBufferLayout::CContiguous"));
    assert!(rendered.contains("__sifr_python_result"));
    assert!(rendered.contains("map_err"));
}

#[test]
fn bridge_buffer_wrapper_calls_resolved_package_producer_before_acquire() {
    let function = buffer_function(
        vec!["__sifr_bridge__", "p_abc123", "views", "make"],
        PythonBufferAccess::Read,
        PythonBufferLayout::Any,
    );

    let body = python_interop_function_body(&function, &Default::default())
        .expect("bridge buffer wrapper should lower");
    let rendered = render_stmts(&body);

    assert!(rendered.contains("\"__sifr_bridge__\""));
    assert!(rendered.contains("\"p_abc123\""));
    assert!(rendered.contains("sifr_runtime::python::call_object_owned"));
    assert!(rendered.contains("sifr_stdlib::python::PythonBuffer::<u16>::acquire"));
}

#[test]
fn self_buffer_wrapper_acquires_opaque_receiver_without_python_call() {
    let function = buffer_function(
        vec!["Self"],
        PythonBufferAccess::Read,
        PythonBufferLayout::FContiguous,
    );

    let body = python_interop_method_body(&function, &Default::default(), None)
        .expect("receiver buffer wrapper should lower");
    let rendered = render_stmts(&body);

    assert!(rendered.contains("self.__sifr_python_object"));
    assert!(rendered.contains("sifr_stdlib::python::PythonBuffer::<u16>::acquire_foreign"));
    assert!(rendered.contains("sifr_runtime::python::PythonBufferAccess::Read"));
    assert!(rendered.contains("sifr_runtime::python::PythonBufferLayout::FContiguous"));
    assert!(!rendered.contains("resolve_target"));
    assert!(!rendered.contains("call_object_owned"));
}

#[test]
fn self_buffer_source_emits_shared_receiver_signature() {
    let source = format!(
        "{ERROR}\n@python.opaque(type=pkg.Owner, cleanup=drop)\nclass Owner(NonSend):\n    @python.buffer(Self, access=read, layout=any)\n    def view(self) -> Result[python.Buffer[uint8], PythonError]: ...\n"
    );
    let parsed = sifr_python_parser::parse_module(&source).expect("source should parse");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("source should lower");
    let rust = generate_rust(&lowered.module);

    assert!(
        rust.contains(
            "fn view(&self) -> Result<sifr_stdlib::python::PythonBuffer<u8>, PythonError>"
        ),
        "{rust}"
    );
    syn::parse_file(&rust).expect("generated receiver buffer Rust should parse");
}

fn buffer_function(
    target: Vec<&str>,
    access: PythonBufferAccess,
    layout: PythonBufferLayout,
) -> HirFunction {
    HirFunction {
        name: "view".to_string(),
        params: Vec::new(),
        return_type: Type::Result(
            Box::new(Type::PythonBuffer(Box::new(Type::FixedInt(
                FixedIntType::U16,
            )))),
            Box::new(python_error_type()),
        ),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![PythonInteropDeclaration {
            kind: PythonInteropDecoratorKind::Buffer,
            target: Some(PythonTargetPath {
                segments: target.into_iter().map(str::to_string).collect(),
                span: TextRange::default(),
            }),
            span: TextRange::default(),
            effect: PythonInteropEffect::BlockingIo,
            cleanup: None,
            consumes_receiver: false,
            parameters: Vec::new(),
            required_import_root: None,
            callbacks: Vec::new(),
            buffer: Some(PythonBufferDeclaration {
                element_type: Type::FixedInt(FixedIntType::U16),
                access,
                layout,
            }),
        }],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    }
}

fn python_error_type() -> Type {
    Type::Class {
        name: "PythonError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}
