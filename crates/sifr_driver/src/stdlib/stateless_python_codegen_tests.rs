use super::compile_stdlib_uncached;
use sifr_sysroot::sha256_hex;

#[test]
fn python_primitive_constructors_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("_sifr.python should generate private Rust code");
    assert_eq!(private_code.module, "_sifr.python");
    assert_eq!(private_code.source_path, "stdlib/_sifr/python.sifr");
    assert_eq!(
        private_code.source_sha256,
        sha256_hex(include_bytes!("../../../../stdlib/_sifr/python.sifr"))
    );
    for name in [
        "py_from_none",
        "py_from_bool",
        "py_from_int",
        "py_from_float",
        "py_from_str",
        "py_from_bytes",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::python::{name}(")),
            "{name} should lower through _sifr.python private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains("PythonError { message: __sifr_bridge_error.message.to_string(), kind: __sifr_bridge_error.kind.to_string(), exception_type: __sifr_bridge_error.exception_type.to_string(), traceback: __sifr_bridge_error.traceback.to_string(), context: __sifr_bridge_error.context.to_string(), __sifr_python_error: Some(__sifr_bridge_error) }"));
    assert!(
        compiled
            .code
            .transitive_deps
            .get("sifr.python_core")
            .is_some_and(|deps| deps.contains("_sifr.python"))
    );
    assert!(
        compiled
            .code
            .transitive_deps
            .get("sifr.python")
            .is_some_and(|deps| deps.contains("_sifr.python"))
    );
}

#[test]
fn python_primitive_extractors_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("_sifr.python should generate private Rust code");
    for name in [
        "py_to_none",
        "py_to_bool",
        "py_to_int",
        "py_to_i8",
        "py_to_i16",
        "py_to_i32",
        "py_to_i64",
        "py_to_u8",
        "py_to_u16",
        "py_to_u32",
        "py_to_u64",
        "py_to_isize",
        "py_to_usize",
        "py_to_float",
        "py_to_str",
        "py_to_bytes",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::python::{name}(")),
            "{name} should lower through _sifr.python private Rust interop declarations"
        );
    }
    assert!(
        private_code
            .rust
            .contains("::sifr_stdlib::python::py_to_int(object)")
    );
    assert!(
        private_code
            .rust
            .contains("__sifr_bridge_ok.into_sifr_int()"),
        "{}",
        private_code.rust
    );
}

#[test]
fn python_object_core_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("_sifr.python should generate private Rust code");
    for name in [
        "py_import_module",
        "py_get_attr",
        "py_get_item_str",
        "py_close",
        "py_resource_diagnostics",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::python::{name}(")),
            "{name} should lower through _sifr.python private Rust interop declarations"
        );
    }
    assert!(
        private_code
            .rust
            .contains("::sifr_stdlib::python::py_get_attr(object, name)")
    );
    assert!(!private_code.rust.contains("SifrIntBridge::from(handle)"));
    assert!(!private_code.rust.contains("SifrIntBridge::from(token)"));
}

#[test]
fn python_collection_constructors_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("_sifr.python should generate private Rust code");
    for name in [
        "py_from_list",
        "py_from_tuple",
        "py_from_dict_str",
        "py_from_record",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::python::{name}(")),
            "{name} should lower through _sifr.python private Rust interop declarations"
        );
    }
}

#[test]
fn python_call_helpers_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("_sifr.python should generate private Rust code");
    for name in ["py_call", "py_call_attr"] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::python::{name}(")),
            "{name} should lower through _sifr.python private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains("kwargs_keys"));
    assert!(private_code.rust.contains("kwargs_values"));
}

#[test]
fn python_copy_helpers_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("_sifr.python should generate private Rust code");
    for name in [
        "py_copy_list_bool",
        "py_copy_list_int",
        "py_copy_list_i32",
        "py_copy_list_u8",
        "py_copy_list_float",
        "py_copy_list_str",
        "py_copy_list_bytes",
        "py_copy_tuple_bool",
        "py_copy_tuple_int",
        "py_copy_tuple_i32",
        "py_copy_tuple_u8",
        "py_copy_tuple_float",
        "py_copy_tuple_str",
        "py_copy_tuple_bytes",
        "py_copy_dict_str_bool",
        "py_copy_dict_str_int",
        "py_copy_dict_str_i32",
        "py_copy_dict_str_u8",
        "py_copy_dict_str_float",
        "py_copy_dict_str_str",
        "py_copy_dict_str_bytes",
        "py_copy_record_fields",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::python::{name}(")),
            "{name} should lower through _sifr.python private Rust interop declarations"
        );
    }
}

#[test]
fn python_zero_copy_helpers_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("_sifr.python should generate private Rust code");
    for name in [
        "py_buffer_u8",
        "py_buffer_shape",
        "py_buffer_strides",
        "py_buffer_suboffsets",
        "py_copy_buffer_u8",
        "py_release_buffer",
        "py_arrow_array",
        "py_arrow_capsule_names",
        "py_arrow_stream",
        "py_arrow_schema",
        "py_release_arrow",
        "py_dlpack_tensor",
        "py_dlpack_shape",
        "py_dlpack_strides",
        "py_release_dlpack",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::python::{name}(")),
            "{name} should lower through _sifr.python private Rust interop declarations"
        );
    }
    let public_code = compiled
        .code
        .module_rust_code
        .get("sifr.python")
        .expect("sifr.python should generate public Rust code");
    // Check propagation and borrowing independently of expression parentheses.
    fn ungroup(mut expr: &syn::Expr) -> &syn::Expr {
        loop {
            expr = match expr {
                syn::Expr::Paren(paren) => &paren.expr,
                syn::Expr::Group(group) => &group.expr,
                _ => return expr,
            };
        }
    }
    #[derive(Default)]
    struct PropagatedMetadataCalls(std::collections::HashSet<String>);
    impl<'ast> syn::visit::Visit<'ast> for PropagatedMetadataCalls {
        fn visit_expr_try(&mut self, expr: &'ast syn::ExprTry) {
            if let syn::Expr::Call(call) = ungroup(&expr.expr)
                && let syn::Expr::Path(function) = ungroup(&call.func)
                && let Some(name) = function.path.get_ident()
                && call.args.len() == 1
                && let Some(syn::Expr::Reference(reference)) = call.args.first().map(ungroup)
                && reference.mutability.is_none()
                && let syn::Expr::Field(field) = ungroup(&reference.expr)
                && matches!(&field.member, syn::Member::Unnamed(index) if index.index == 0)
                && let syn::Expr::Path(base) = ungroup(&field.base)
                && base.path.is_ident("raw")
            {
                self.0.insert(name.to_string());
            }
            syn::visit::visit_expr_try(self, expr);
        }
    }
    let parsed = syn::parse_file(&public_code.rust).expect("public Python helpers must parse");
    let mut propagated = PropagatedMetadataCalls::default();
    syn::visit::Visit::visit_file(&mut propagated, &parsed);
    for call in [
        "py_buffer_shape",
        "py_buffer_strides",
        "py_buffer_suboffsets",
        "py_arrow_capsule_names",
        "py_dlpack_shape",
        "py_dlpack_strides",
    ] {
        assert!(
            propagated.0.contains(call),
            "{call} should borrow raw.0 and propagate metadata accessor errors; emitted public source:\n{}",
            public_code.rust
        );
    }
}

#[test]
fn python_context_coroutine_helpers_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("_sifr.python should generate private Rust code");
    for name in [
        "py_enter_context",
        "py_exit_context",
        "py_exit_context_with_error",
        "py_run_coroutine_blocking",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::python::{name}(")),
            "{name} should lower through _sifr.python private Rust interop declarations"
        );
    }
}

#[test]
fn python_callback_helpers_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("_sifr.python should generate private Rust code");
    for name in [
        "py_local_callback",
        "py_threadsafe_callback",
        "py_local_callback_echo",
        "py_threadsafe_callback_echo",
        "py_close_callback",
    ] {
        assert!(
            private_code
                .rust
                .contains(&format!("::sifr_stdlib::python::{name}(")),
            "{name} should lower through _sifr.python private Rust interop declarations"
        );
    }
    assert!(private_code.rust.contains("handler(&__sifr_callback_arg)"));
    assert!(
        private_code
            .rust
            .contains("::sifr_stdlib::python::PythonError")
    );
    let compact_private: String = private_code
        .rust
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    let python_error = sifr_type_system::stdlib_class_rust_name("_sifr.python", "PythonError");
    assert!(compact_private.contains(&format!(
        "handler:implFn(&::sifr_runtime::interop::Handle<::sifr_runtime::python::ForeignObject>)->Result<::sifr_runtime::interop::Handle<::sifr_runtime::python::ForeignObject>,{python_error}>+Send+Sync+'static"
    )));
    assert!(!compact_private.contains("(i64,i64)"));
    let public_core_code = compiled
        .code
        .module_rust_code
        .get("sifr.python_core")
        .expect("sifr.python_core should generate Rust code");
    let compact_public: String = public_core_code
        .rust
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(compact_public.contains("py_local_callback(handler)?"));
    assert!(!compact_public.contains("_call_object_callback"));
}
