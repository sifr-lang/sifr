use crate::generate_rust;

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
    generate_rust(&lowered.module).expect("code generation should succeed")
}

#[test]
fn cpu_producer_acquires_typed_tensor_with_explicit_no_stream_policy() {
    let rust = generate(&format!(
        "{ERROR}\n@python.dlpack(pkg.make, device=cpu, stream=none)\ndef acquire(size: int) -> Result[python.DlpackTensor[float], PythonError]: ...\n"
    ));

    assert!(
        rust.contains("::sifr_runtime::python::call_object_owned"),
        "{rust}"
    );
    assert!(
        rust.contains("::sifr_stdlib::python::PythonDlpackTensor::<f64>::acquire_foreign"),
        "{rust}"
    );
    assert!(rust.contains("\"cpu\""), "{rust}");
    assert!(rust.contains("None"), "{rust}");
    syn::parse_file(&rust).expect("generated DLPack Rust should parse");
}

#[test]
fn cuda_stream_parameter_is_used_for_acquisition_and_not_forwarded() {
    let rust = generate(&format!(
        "{ERROR}\n@python.dlpack(pkg.make, device=cuda, stream=parameter(consumer_stream))\ndef acquire(size: int, *, consumer_stream: python.DlpackStream) -> Result[python.DlpackTensor[float], PythonError]: ...\n"
    ));

    assert!(
        rust.contains("PythonDlpackTensor::<f64>::acquire_foreign"),
        "{rust}"
    );
    assert!(rust.contains("Some(&consumer_stream)"), "{rust}");
    assert!(rust.contains("\"cuda\""), "{rust}");
    assert!(!rust.contains("\"consumer_stream\".to_string()"), "{rust}");
    syn::parse_file(&rust).expect("generated CUDA DLPack Rust should parse");
}

#[test]
fn stream_producer_acquires_closed_stream_value() {
    let rust = generate(&format!(
        "{ERROR}\n@python.dlpack.stream(pkg.current_stream, device=cuda)\ndef current_stream(device_id: int) -> Result[python.DlpackStream, PythonError]: ...\n"
    ));

    assert!(
        rust.contains("::sifr_stdlib::python::PythonDlpackStream::acquire_foreign"),
        "{rust}"
    );
    assert!(rust.contains("\"cuda\""), "{rust}");
    syn::parse_file(&rust).expect("generated DLPack stream Rust should parse");
}

#[test]
fn self_receiver_acquires_tensor_and_methods_map_runtime_errors() {
    let rust = generate(&format!(
        "{ERROR}\n@python.opaque(type=pkg.Owner, cleanup=drop)\nclass Owner(NonSend):\n    @python.dlpack(Self, device=cpu, stream=none)\n    def tensor(self) -> Result[python.DlpackTensor[float], PythonError]: ...\n\ndef inspect(own value: python.DlpackTensor[float]) -> Result[None, PythonError]:\n    try:\n        shape: list[int] = value.shape()\n        strides: list[int] = value.strides()\n        released: None = value.release()\n        return None\n    except PythonError as error:\n        raise error\n"
    ));

    assert!(
        rust.contains("PythonDlpackTensor::<f64>::acquire_foreign"),
        "{rust}"
    );
    assert!(rust.contains("self.__sifr_python_object"), "{rust}");
    assert!(rust.contains("value.shape()"), "{rust}");
    assert!(rust.contains("value.strides()"), "{rust}");
    assert!(rust.contains("value.release().map_err("), "{rust}");
    syn::parse_file(&rust).expect("generated receiver DLPack Rust should parse");
}

#[test]
fn owned_consumer_commits_move_and_reconciles_after_python_call() {
    let rust = generate(&format!(
        "{ERROR}\n@python(pkg.consume)\ndef consume(own value: python.DlpackTensor[float]) -> Result[int, PythonError]: ...\n"
    ));

    assert!(rust.contains("value.prepare_argument()"), "{rust}");
    assert!(
        rust.contains("__sifr_python_dlpack_argument_0.object()"),
        "{rust}"
    );
    assert!(
        rust.contains("::std::mem::drop(__sifr_python_args)"),
        "{rust}"
    );
    assert!(
        rust.contains("__sifr_python_dlpack_argument_0.finish()"),
        "{rust}"
    );
    assert!(
        rust.contains("::sifr_stdlib::python::reconcile_dlpack_argument"),
        "{rust}"
    );
    syn::parse_file(&rust).expect("generated consumer DLPack Rust should parse");
}

#[test]
fn owned_consumer_method_uses_the_same_one_shot_transfer_path() {
    let rust = generate(&format!(
        "{ERROR}\n@python.opaque(type=pkg.Sink, cleanup=drop)\nclass Sink(NonSend):\n    @python(Self.push)\n    def push(self, own value: python.DlpackTensor[float]) -> Result[None, PythonError]: ...\n"
    ));

    assert!(rust.contains("value.prepare_argument()"), "{rust}");
    assert!(
        rust.contains("__sifr_python_dlpack_argument_0.finish()"),
        "{rust}"
    );
    assert!(rust.contains("reconcile_dlpack_argument"), "{rust}");
    syn::parse_file(&rust).expect("generated consumer method DLPack Rust should parse");
}

#[test]
fn mixed_arrow_and_dlpack_arguments_drop_call_handles_once_before_reconciliation() {
    let rust = generate(&format!(
        "{ERROR}\n@python(pkg.consume)\ndef consume(own array: python.ArrowArray, own tensor: python.DlpackTensor[int64]) -> Result[int, PythonError]: ...\n"
    ));

    assert_eq!(
        rust.matches("::std::mem::drop(__sifr_python_args)").count(),
        1
    );
    assert_eq!(
        rust.matches("::std::mem::drop(__sifr_python_kwargs)")
            .count(),
        1
    );
    assert!(rust.contains("reconcile_arrow_argument"), "{rust}");
    assert!(rust.contains("reconcile_dlpack_argument"), "{rust}");
    let call = rust.find("call_object_owned").expect("owned Python call");
    let drop_args = rust
        .find("::std::mem::drop(__sifr_python_args)")
        .expect("argument-vector drop");
    let arrow_finish = rust
        .find("reconcile_arrow_argument")
        .expect("Arrow cleanup");
    let dlpack_finish = rust
        .find("reconcile_dlpack_argument")
        .expect("DLPack cleanup");
    assert!(call < drop_args && drop_args < arrow_finish && arrow_finish < dlpack_finish);
    syn::parse_file(&rust).expect("mixed zero-copy consumer Rust should parse");
}

#[test]
fn multiple_dlpack_arguments_share_one_call_handle_cleanup() {
    let rust = generate(&format!(
        "{ERROR}\n@python(pkg.consume)\ndef consume(own left: python.DlpackTensor[int64], own right: python.DlpackTensor[int64]) -> Result[int, PythonError]: ...\n"
    ));

    assert_eq!(
        rust.matches("::std::mem::drop(__sifr_python_args)").count(),
        1
    );
    assert_eq!(rust.matches("reconcile_dlpack_argument").count(), 2);
    syn::parse_file(&rust).expect("multi-tensor consumer Rust should parse");
}
