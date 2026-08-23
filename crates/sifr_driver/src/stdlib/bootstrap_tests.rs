use super::*;
use sifr_lowering::CompilerIntrinsicId;
use std::path::PathBuf;

fn sample_param(name: &str, ty: Type, convention: ParamConvention) -> HirParam {
    HirParam {
        name: name.to_string(),
        ty,
        default: None,
        keyword_only: false,
        convention,
    }
}

#[test]
fn function_type_from_params_preserves_named_conventions() {
    let params = vec![
        sample_param("value", Type::Int, ParamConvention::borrow()),
        sample_param("count", Type::Int, ParamConvention::own()),
    ];

    let function_type = function_type_from_params(&params, &Type::Bool);

    assert_eq!(
        function_type,
        FunctionType {
            receiver: None,
            params: vec![
                ("value".to_string(), Type::Int, ParamConvention::borrow()),
                ("count".to_string(), Type::Int, ParamConvention::own()),
            ],
            return_type: Box::new(Type::Bool),
        }
    );
}

#[test]
fn function_type_from_hir_exports_async_functions_as_coroutines() {
    let function = HirFunction {
        name: "async_status".to_string(),
        params: vec![sample_param("value", Type::Int, ParamConvention::borrow())],
        return_type: Type::Result(Box::new(Type::Bool), Box::new(Type::Str)),
        body: Vec::new(),
        is_async: true,
        method_kind: sifr_ir::MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    assert_eq!(
        function_type_from_hir(&function),
        FunctionType {
            receiver: None,
            params: vec![("value".to_string(), Type::Int, ParamConvention::borrow())],
            return_type: Box::new(Type::Coroutine(Box::new(Type::Bool), Box::new(Type::Str))),
        }
    );
}

#[test]
fn signature_params_can_override_constructor_conventions() {
    let params = vec![sample_param("self", Type::Str, ParamConvention::borrow())];

    assert_eq!(
        signature_params(&params, Some(ParamConvention::own())),
        vec![(Type::Str, ParamConvention::own())]
    );
}

#[test]
fn public_constant_integer_value_exports_filter_to_public_recorded_values() {
    let mut values = HashMap::new();
    values.insert("ANSWER".to_string(), 42);
    values.insert("_PRIVATE".to_string(), 99);
    values.insert("STALE".to_string(), 100);

    let exports =
        collect_public_constant_integer_value_exports(["ANSWER", "MISSING"].into_iter(), &values);

    assert_eq!(exports.len(), 1);
    assert_eq!(exports.get("ANSWER"), Some(&42));
    assert!(!exports.contains_key("_PRIVATE"));
    assert!(!exports.contains_key("STALE"));
}

#[test]
fn stdlib_class_exports_preserve_parent_markers_and_generic_templates() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let channel_sender = compiled
        .defs
        .classes
        .get("sifr.sync")
        .and_then(|classes| classes.get("ChannelSender"))
        .expect("sifr.sync.ChannelSender should be exported");
    let Type::Class { methods, .. } = channel_sender else {
        panic!("ChannelSender should be a class");
    };
    let send = methods
        .iter()
        .find(|(name, _)| name == "send")
        .map(|(_, method)| method)
        .expect("ChannelSender.send should be exported");
    assert!(send.params[0].2.is_owned(), "{send:?}");
    let object_ty = compiled
        .defs
        .classes
        .get("sifr.python")
        .and_then(|classes| classes.get("Object"))
        .expect("sifr.python.Object should be exported");

    assert!(matches!(
        object_ty,
        Type::Class {
            identity: Some(identity),
            parent_class: Some(parent),
            ..
        } if identity == "_sifr.python.Object"
            && parent.split('|').any(|name| name == "NonSend")
    ));

    let shared_ty = compiled
        .defs
        .classes
        .get("sifr.sync")
        .and_then(|classes| classes.get("Shared"))
        .expect("sifr.sync.Shared should be exported");
    assert!(matches!(
        shared_ty,
        Type::Class { type_args, .. }
            if type_args == &vec![Type::TypeVar("T".to_string())]
    ));

    let (_, return_type) = compiled
        .code
        .func_signatures
        .get("_sifr.python")
        .and_then(|functions| functions.get("py_from_none"))
        .expect("private Python bridge signature should be recorded");
    let Type::Result(object, _) = return_type.resolve_alias() else {
        panic!("py_from_none should return Result");
    };
    assert!(object.is_python_object_contract(), "{object:?}");
    let private_python_rust = &compiled
        .code
        .module_rust_code
        .get("_sifr.python")
        .expect("private Python module should have generated Rust")
        .rust;
    let python_error = sifr_type_system::stdlib_class_rust_name("_sifr.python", "PythonError");
    assert!(
        private_python_rust.contains(&format!(
            "Result<::sifr_runtime::interop::Handle<::sifr_runtime::python::ForeignObject>, {python_error}>"
        )),
        "{private_python_rust}"
    );
}

#[test]
fn python_core_re_exports_preserve_callable_metadata() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let workloads = compiled
        .defs
        .function_workloads
        .get("sifr.python")
        .expect("sifr.python should export workload metadata");

    assert_eq!(
        workloads.get("threadsafe_callback_echo"),
        Some(&"blocking_io".to_string())
    );
    assert_eq!(
        workloads.get("close_local_callback"),
        Some(&"blocking_io".to_string())
    );

    let defaults = compiled
        .defs
        .function_defaults
        .get("sifr.python")
        .and_then(|module_defaults| module_defaults.get("PythonError"))
        .expect("PythonError constructor defaults should be re-exported");
    assert_eq!(defaults.len(), 4);
}

#[test]
fn python_call_helpers_borrow_argument_collections() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let functions = compiled
        .defs
        .functions
        .get("sifr.python")
        .expect("sifr.python functions should be exported");

    for function_name in ["call", "call_attr"] {
        let function = functions
            .get(function_name)
            .unwrap_or_else(|| panic!("sifr.python.{function_name} should be exported"));
        for parameter_name in ["args", "kwargs"] {
            let convention = function
                .params
                .iter()
                .find_map(|(name, _, convention)| (name == parameter_name).then_some(*convention))
                .unwrap_or_else(|| {
                    panic!("sifr.python.{function_name} should export {parameter_name}")
                });
            assert!(
                convention.is_shared_borrow(),
                "sifr.python.{function_name} must borrow {parameter_name} so temporary argument lists clone their handles"
            );
        }
    }
}

#[test]
fn retained_public_declarations_export_typed_compiler_identity() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    assert_eq!(
        compiled
            .defs
            .compiler_intrinsics
            .get("sifr.test")
            .and_then(|ids| ids.get("assert_eq")),
        Some(&CompilerIntrinsicId::TestAssertEqual)
    );
    assert_eq!(
        compiled
            .defs
            .compiler_intrinsics
            .get("sifr.task")
            .and_then(|ids| ids.get("current_context")),
        Some(&CompilerIntrinsicId::TaskCurrentContext)
    );
}

#[test]
fn numeric_and_random_modules_export_only_canonical_operation_names() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");

    let math = compiled
        .defs
        .functions
        .get("sifr.math")
        .expect("sifr.math functions should be exported");
    for name in ["fabs", "pow"] {
        assert!(
            math.contains_key(name),
            "sifr.math.{name} should be exported"
        );
    }
    for name in [
        "abs_val",
        "pow_val",
        "min_val",
        "max_val",
        "round_val",
        "dist_impl",
        "fsum_impl",
        "sumprod_impl",
    ] {
        assert!(
            !math.contains_key(name),
            "sifr.math.{name} must remain private"
        );
    }

    let random = compiled
        .defs
        .functions
        .get("sifr.random")
        .expect("sifr.random functions should be exported");
    for name in ["randint", "random", "uniform", "choice"] {
        assert!(
            random.contains_key(name),
            "sifr.random.{name} should be exported"
        );
    }
    for name in [
        "random_int",
        "random_float",
        "random_uniform",
        "random_randrange",
        "random_gauss",
        "random_module_state_words",
        "random_module_state_index",
        "random_module_state_gauss_next",
        "random_module_set_state",
    ] {
        assert!(
            !random.contains_key(name),
            "sifr.random.{name} must remain private"
        );
    }

    for module_name in ["sifr.secrets", "sifr.tempfile"] {
        let functions = compiled
            .defs
            .functions
            .get(module_name)
            .unwrap_or_else(|| panic!("{module_name} functions should be exported"));
        assert!(
            !functions.contains_key("random_int"),
            "{module_name}.random_int must remain private"
        );
    }
}

#[test]
fn runtime_information_modules_export_only_canonical_operation_names() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");

    for (module_name, canonical, private) in [
        (
            "sifr.platform",
            &[
                "system",
                "machine",
                "node",
                "release",
                "version",
                "processor",
            ][..],
            &[
                "platform_system",
                "platform_arch",
                "platform_node",
                "platform_release",
                "platform_version",
                "platform_processor",
            ][..],
        ),
        (
            "sifr.time",
            &["time", "strftime", "sleep", "perf_counter", "monotonic"][..],
            &["time_now", "time_format"][..],
        ),
        (
            "sifr.sys",
            &["argv", "exit", "version", "platform", "maxsize"][..],
            &[
                "get_args",
                "sys_exit",
                "sys_version",
                "sys_platform",
                "sys_maxsize",
            ][..],
        ),
        (
            "sifr.env",
            &[
                "getenv_opt",
                "getenv",
                "setenv",
                "unsetenv",
                "keys",
                "values",
                "items",
            ][..],
            &[
                "env_get",
                "env_set",
                "env_unset",
                "env_keys",
                "env_values",
                "env_items",
            ][..],
        ),
    ] {
        let functions = compiled
            .defs
            .functions
            .get(module_name)
            .unwrap_or_else(|| panic!("{module_name} functions should be exported"));
        for name in canonical {
            assert!(
                functions.contains_key(*name),
                "{module_name}.{name} should be exported"
            );
        }
        for name in private {
            assert!(
                !functions.contains_key(*name),
                "{module_name}.{name} must remain private"
            );
        }
    }

    let datetime = compiled
        .defs
        .functions
        .get("sifr.datetime")
        .expect("sifr.datetime functions should be exported");
    assert!(datetime.contains_key("UTC"));
    assert!(!datetime.contains_key("utc"));
    assert!(!datetime.contains_key("time_now"));

    for module_name in ["sifr.random", "sifr.os"] {
        let functions = compiled
            .defs
            .functions
            .get(module_name)
            .unwrap_or_else(|| panic!("{module_name} functions should be exported"));
        assert!(
            !functions.contains_key("time_now") && !functions.contains_key("get_args"),
            "{module_name} must not leak runtime-information intrinsics"
        );
    }
}

#[test]
fn text_and_data_modules_export_only_canonical_operation_names() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");

    for (module_name, canonical, removed) in [
        (
            "sifr.re",
            &[
                "search",
                "search_flags",
                "search_match",
                "sub",
                "sub_flags",
                "findall",
                "findall_flags",
                "split",
                "split_flags",
                "finditer",
                "compile",
                "compile_flags",
                "fullmatch",
                "fullmatch_flags",
            ][..],
            &[
                "re_match",
                "re_find",
                "re_replace",
                "re_findall",
                "re_split",
                "re_find_start",
                "re_find_end",
                "re_match_flags",
                "re_find_flags",
                "re_replace_flags",
                "re_findall_flags",
                "re_split_flags",
                "compile_pattern",
                "compile_pattern_flags",
            ][..],
        ),
        (
            "sifr.json",
            &[
                "loads",
                "dumps",
                "dumps_exact",
                "dumps_web",
                "dumps_string_ints",
            ][..],
            &[
                "json_loads",
                "json_dumps",
                "json_dumps_value",
                "json_dumps_value_exact",
                "json_dumps_value_web",
                "json_dumps_value_string_ints",
                "json_load_tokens",
                "json_dump_tokens",
            ][..],
        ),
        (
            "sifr.tomllib",
            &["loads"][..],
            &["toml_loads", "toml_parse_tokens"][..],
        ),
        (
            "sifr.base64",
            &[
                "b64encode",
                "b64decode",
                "b64encode_bytes",
                "b64decode_bytes",
                "b64encode_opts",
                "b64decode_opts",
            ][..],
            &[
                "base64_encode",
                "base64_decode",
                "base64_encode_bytes",
                "base64_decode_bytes",
                "base64_encode_opts",
                "base64_decode_opts",
                "standard_b64encode",
                "standard_b64decode",
                "standard_b64encode_bytes",
                "standard_b64decode_bytes",
                "encodebytes",
                "decodebytes",
                "encodebytes_bytes",
                "decodebytes_bytes",
            ][..],
        ),
        (
            "sifr.fnmatch",
            &["fnmatch", "filter", "filterfalse"][..],
            &["fnmatch_filter", "fnmatchcase"][..],
        ),
        (
            "sifr.html",
            &["escape", "unescape"][..],
            &["html_escape", "html_unescape"][..],
        ),
        (
            "sifr.calendar",
            &["isleap", "weekday", "monthrange", "leapdays"][..],
            &["calendar_isleap", "calendar_weekday", "calendar_monthrange"][..],
        ),
        (
            "sifr.url",
            &[
                "parse",
                "build",
                "percent_encode",
                "percent_decode",
                "percent_encode_bytes",
                "percent_decode_bytes",
                "normalize_path",
                "parse_query",
                "build_query",
            ][..],
            &[
                "parse_url",
                "build_url",
                "url_parse",
                "url_build",
                "url_percent_encode",
                "url_percent_decode",
                "url_query_parse",
                "url_query_build",
            ][..],
        ),
    ] {
        let functions = compiled
            .defs
            .functions
            .get(module_name)
            .unwrap_or_else(|| panic!("{module_name} functions should be exported"));
        for name in canonical {
            assert!(
                functions.contains_key(*name),
                "{module_name}.{name} should be exported"
            );
        }
        for name in removed {
            assert!(
                !functions.contains_key(*name),
                "{module_name}.{name} must remain private"
            );
        }
    }

    let base64 = compiled
        .defs
        .functions
        .get("sifr.base64")
        .expect("sifr.base64 functions should be exported");
    let text_encode = base64.get("b64encode").expect("text base64 encoder");
    let bytes_encode = base64.get("b64encode_bytes").expect("bytes base64 encoder");
    assert_eq!(text_encode.params[0].1, Type::Str);
    assert_eq!(*text_encode.return_type, Type::Str);
    assert_eq!(bytes_encode.params[0].1, Type::Bytes);
    assert_eq!(*bytes_encode.return_type, Type::Bytes);

    let regex_classes = compiled
        .defs
        .classes
        .get("sifr.re")
        .expect("sifr.re classes should be exported");
    assert!(regex_classes.contains_key("Pattern"));
    assert!(!regex_classes.contains_key("CompiledPattern"));
}

fn fixture_source(module: &str, source: &str, kind: LoadedStdlibSourceKind) -> LoadedStdlibSource {
    let stdlib_root =
        std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"))
            .expect("development stdlib root should resolve");
    LoadedStdlibSource {
        module: module.to_string(),
        source: source.to_string(),
        path: stdlib_root.join(format!("{}.sifr", module.replace('.', "/"))),
        kind,
    }
}

fn compile_fixture_sources(
    sources: &[LoadedStdlibSource],
) -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    let sysroot = sifr_sysroot::resolve_sysroot(None).expect("development sysroot should resolve");
    compile_stdlib_sources_with_sysroot(sources, sysroot)
}

fn fixture_diagnostics(sources: &[LoadedStdlibSource]) -> Vec<RenderedDiagnostic> {
    match compile_fixture_sources(sources) {
        Ok(_) => panic!("fixture should fail stdlib bootstrap"),
        Err(diagnostics) => diagnostics,
    }
}

#[test]
fn binary_and_hashing_exports_use_only_first_class_bytes_contracts() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");

    assert!(
        !compiled.defs.functions.contains_key("sifr.bytes")
            && !compiled.defs.classes.contains_key("sifr.bytes"),
        "the transitional sifr.bytes module must not be exported"
    );

    let functions = compiled
        .defs
        .functions
        .get("sifr.hashlib")
        .expect("sifr.hashlib functions should be exported");
    for name in [
        "new", "md5", "sha1", "sha224", "sha256", "sha384", "sha512", "blake2b", "blake2s",
    ] {
        assert!(
            functions.contains_key(name),
            "sifr.hashlib.{name} should exist"
        );
    }
    for removed in [
        "new_bytes",
        "md5_obj",
        "sha1_obj",
        "sha224_obj",
        "sha256_obj",
        "sha384_obj",
        "sha512_obj",
        "blake2b_obj",
        "blake2s_obj",
        "md5_bytes",
        "sha1_bytes",
        "sha224_bytes",
        "sha256_bytes",
        "sha384_bytes",
        "sha512_bytes",
        "blake2b_bytes",
        "blake2s_bytes",
    ] {
        assert!(
            !functions.contains_key(removed),
            "sifr.hashlib.{removed} must not be exported"
        );
    }

    let new_signature = functions.get("new").expect("hashlib.new signature");
    assert_eq!(new_signature.params[1].1, Type::Bytes);
    for name in [
        "md5", "sha1", "sha224", "sha256", "sha384", "sha512", "blake2b", "blake2s",
    ] {
        let signature = functions
            .get(name)
            .unwrap_or_else(|| panic!("hashlib.{name} signature"));
        assert_eq!(signature.params[0].1, Type::Bytes);
    }

    let hash_object = compiled
        .defs
        .classes
        .get("sifr.hashlib")
        .and_then(|classes| classes.get("HashObject"))
        .expect("sifr.hashlib.HashObject should be exported");
    let Type::Class { methods, .. } = hash_object else {
        panic!("HashObject should be a class");
    };
    for name in ["update", "digest", "hexdigest"] {
        assert!(
            methods.iter().any(|(method, _)| method == name),
            "HashObject.{name} should exist"
        );
    }
    for removed in ["update_bytes", "digest_bytes"] {
        assert!(
            methods.iter().all(|(method, _)| method != removed),
            "HashObject.{removed} must not be exported"
        );
    }
    let update = methods
        .iter()
        .find(|(method, _)| method == "update")
        .map(|(_, signature)| signature)
        .expect("HashObject.update signature");
    assert_eq!(update.params[0].1, Type::Bytes);
    let digest = methods
        .iter()
        .find(|(method, _)| method == "digest")
        .map(|(_, signature)| signature)
        .expect("HashObject.digest signature");
    assert_eq!(*digest.return_type, Type::Bytes);
    let hexdigest = methods
        .iter()
        .find(|(method, _)| method == "hexdigest")
        .map(|(_, signature)| signature)
        .expect("HashObject.hexdigest signature");
    assert_eq!(*hexdigest.return_type, Type::Str);
}

#[test]
fn collections_and_sorted_insert_modules_export_only_canonical_operations() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");

    assert!(
        !compiled.defs.functions.contains_key("_sifr.collections"),
        "the retired list-backed private adapter module must not be compiled"
    );

    let collections = compiled
        .defs
        .functions
        .get("sifr.collections")
        .expect("sifr.collections functions should be exported");
    for name in ["from_list"] {
        assert!(
            collections.contains_key(name),
            "sifr.collections.{name} should exist"
        );
    }
    let collection_classes = compiled
        .defs
        .classes
        .get("sifr.collections")
        .expect("sifr.collections classes should be exported");
    for name in ["Counter", "deque", "frozenset"] {
        assert!(
            collection_classes.contains_key(name),
            "sifr.collections.{name} should exist"
        );
    }
    for removed in [
        "new_set",
        "set_from_list",
        "set_add",
        "set_contains",
        "set_remove",
        "set_len",
        "set_union",
        "set_intersection",
    ] {
        assert!(
            !collections.contains_key(removed),
            "sifr.collections.{removed} must not be exported"
        );
    }

    let heapq = compiled
        .defs
        .functions
        .get("sifr.heapq")
        .expect("sifr.heapq functions should be exported");
    for name in [
        "heapify",
        "heappush",
        "heappop",
        "heapreplace",
        "heappushpop",
    ] {
        assert!(heapq.contains_key(name), "sifr.heapq.{name} should exist");
    }
    for removed in [
        "heapify_copy",
        "heappush_copy",
        "heappop_val",
        "heappop_rest",
    ] {
        assert!(
            !heapq.contains_key(removed),
            "sifr.heapq.{removed} must not be exported"
        );
    }

    let bisect = compiled
        .defs
        .functions
        .get("sifr.bisect")
        .expect("sifr.bisect functions should be exported");
    for name in ["bisect_left", "bisect_right", "insort_left", "insort_right"] {
        assert!(bisect.contains_key(name), "sifr.bisect.{name} should exist");
    }
    for removed in ["bisect", "insort", "insort_left_copy", "insort_right_copy"] {
        assert!(
            !bisect.contains_key(removed),
            "sifr.bisect.{removed} must not be exported"
        );
    }

    let statistics = compiled
        .defs
        .functions
        .get("sifr.statistics")
        .expect("sifr.statistics functions should be exported");
    assert!(statistics.contains_key("mean"));
    assert!(!statistics.contains_key("fmean"));
}

#[test]
fn private_stdlib_imports_resolve_only_from_compiled_source_exports() {
    let sources = [
        fixture_source(
            "_sifr.fixture",
            "def existing(value: int) -> int:\n    return value\n",
            LoadedStdlibSourceKind::PrivateDeclaration,
        ),
        fixture_source(
            "sifr.fixture",
            "from _sifr.fixture import existing\n\ndef forwarded(value: int) -> int:\n    return existing(value)\n",
            LoadedStdlibSourceKind::Public,
        ),
    ];

    let compiled = compile_fixture_sources(&sources).expect("source-backed import should compile");
    assert!(
        compiled
            .defs
            .functions
            .get("sifr.fixture")
            .is_some_and(|functions| functions.contains_key("forwarded"))
    );
    assert!(
        compiled
            .code
            .transitive_deps
            .get("sifr.fixture")
            .is_some_and(|deps| deps.contains("_sifr.fixture"))
    );
}

#[test]
fn missing_private_stdlib_member_is_a_structured_bootstrap_failure() {
    let sources = [
        fixture_source(
            "_sifr.fixture",
            "def existing(value: int) -> int:\n    return value\n",
            LoadedStdlibSourceKind::PrivateDeclaration,
        ),
        fixture_source(
            "sifr.fixture",
            "from _sifr.fixture import absent\n",
            LoadedStdlibSourceKind::Public,
        ),
    ];

    let diagnostics = fixture_diagnostics(&sources);
    assert!(!diagnostics.is_empty());
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE.code())
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("_sifr.fixture") && diagnostic.message.contains("absent")
    }));
}

#[test]
fn missing_private_stdlib_module_is_a_structured_bootstrap_failure() {
    let sources = [fixture_source(
        "sifr.fixture",
        "from _sifr.missing import absent\n",
        LoadedStdlibSourceKind::Public,
    )];

    let diagnostics = fixture_diagnostics(&sources);
    assert!(!diagnostics.is_empty());
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE.code())
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("_sifr.missing"))
    );
}
