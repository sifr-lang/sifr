use super::*;
#[test]
fn dx7_m01_check_demands_semantics_without_rust_or_unrelated_modules() {
    let context = crate::CompilerContext::for_test();
    let provider = context.metadata_provider().unwrap();
    assert!(provider.loaded_semantic_modules().is_empty());
    let no_import = crate::check(&context, "def main():\n    value: int = 1\n");
    assert!(no_import.is_empty(), "{no_import:?}");
    assert!(provider.loaded_semantic_modules().is_empty());
    let diagnostics = crate::check(
        &context,
        "from sifr.math import sqrt\ndef main():\n    value: float = sqrt(4.0)\n",
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let loaded = provider.loaded_semantic_modules();
    assert!(loaded.iter().any(|name| name == "sifr.math"), "{loaded:?}");
    assert!(
        !loaded.iter().any(|name| name.contains("datetime")),
        "{loaded:?}"
    );
    assert_eq!(
        provider
            .metadata
            .store
            .payload_read_count::<wire::RustPayload>(),
        0
    );
    assert_eq!(
        provider
            .metadata
            .store
            .payload_read_count::<wire::HirModule>(),
        0
    );
    assert_eq!(
        provider.metadata.store.decoded_count::<wire::HirModule>(),
        0
    );
}
#[test]
fn dx7_m02_m04_m15_source_metadata_diagnostics_and_emission_agree() {
    let context = crate::CompilerContext::for_test();
    let source_stdlib = crate::stdlib::compile_stdlib_uncached().unwrap();
    let provider = context.metadata_provider().unwrap();
    let cases = [
        "from sifr.math import sqrt\ndef main():\n    assert sqrt(4.0) == 2.0\n",
        "from sifr.bisect import bisect_left\ndef main():\n    assert bisect_left([1,3,5],3) == 1\n",
        "from sifr.collections import Counter, from_list\ndef main():\n    counts: Counter[str] = from_list([\"a\",\"a\"])\n    assert counts.get(\"a\") == 2\n",
        "from sifr.datetime import date\ndef main():\n    value: date = date(2024,1,2)\n    assert value.year == 2024\n",
        "from sifr.json import dumps_exact, from_int\ndef main():\n    assert dumps_exact(from_int(42)) == \"42\"\n",
        "from sifr.re import search\ndef main():\n    try:\n        assert search(\"[0-9]+\", \"value42\") == \"42\"\n    except RegexError as error:\n        assert False\n",
    ];
    for (case, source) in cases.into_iter().enumerate() {
        let parsed = crate::parse_source(source).unwrap();
        let reference = sifr_frontend::compile_module_hir(
            "main",
            &parsed,
            &source_stdlib.defs,
            sifr_frontend::FrontendDiagnosticStyle::Bare,
        )
        .unwrap();
        let actual = crate::lower_source(&context, source).unwrap();
        assert_eq!(
            reference.module.generic_functions,
            actual.module.generic_functions
        );
        assert_eq!(
            reference.module.type_param_bounds,
            actual.module.type_param_bounds
        );
        assert_eq!(
            format!("{:?}", reference.module.functions),
            format!("{:?}", actual.module.functions)
        );
        assert_eq!(
            format!("{:?}", reference.module.classes),
            format!("{:?}", actual.module.classes)
        );
        assert_eq!(
            format!("{:?}", reference.module.constants),
            format!("{:?}", actual.module.constants)
        );
        assert_eq!(
            format!("{:?}", reference.module.imports),
            format!("{:?}", actual.module.imports)
        );
        let requested = actual
            .module
            .imports
            .iter()
            .map(|import| import.module.clone())
            .collect::<Vec<_>>();
        let metadata = provider
            .materialize(&requested, context.sysroot().unwrap())
            .unwrap();
        let expected = sifr_codegen::generate_rust_with_stdlib_for_module(
            &reference.module,
            &source_stdlib.code,
            Some("main"),
        );
        let actual = sifr_codegen::generate_rust_with_stdlib_for_module(
            &actual.module,
            &metadata.code,
            Some("main"),
        );
        assert_eq!(actual.rust_source, expected.rust_source, "{source}");
        if let Some(output) = std::env::var_os("SIFR_DX7_PARITY_OUTPUT") {
            let output = std::path::PathBuf::from(output);
            std::fs::create_dir_all(&output).unwrap();
            std::fs::write(output.join(format!("case-{case}.sifr")), source).unwrap();
            std::fs::write(
                output.join(format!("case-{case}.source.rs")),
                &expected.rust_source,
            )
            .unwrap();
        }
        assert_eq!(actual.used_stdlib_modules, expected.used_stdlib_modules);
    }
    assert!(provider.metadata.store.decoded_count::<wire::RustPayload>() > 0);
    assert!(provider.metadata.store.decoded_count::<wire::HirModule>() > 0);
    assert!(
        provider
            .metadata
            .store
            .decoded_count::<wire::TemplatePayload>()
            > 0
    );
    assert!(
        !provider
            .loaded_semantic_modules()
            .iter()
            .any(|module| module == "sifr.calendar")
    );
    for source in [
        "from sifr.math import MISSING\ndef main():\n    pass\n",
        "from sifr.math import sqrt\ndef main():\n    value: float = sqrt(\"wrong\")\n",
        "from _sifr.math import sqrt\ndef main():\n    pass\n",
    ] {
        let parsed = crate::parse_source(source).unwrap();
        let reference = sifr_frontend::compile_module_hir(
            "main",
            &parsed,
            &source_stdlib.defs,
            sifr_frontend::FrontendDiagnosticStyle::Bare,
        )
        .err()
        .unwrap();
        let actual = crate::check(&context, source);
        assert_eq!(
            actual
                .iter()
                .map(|d| (&d.code, &d.message))
                .collect::<Vec<_>>(),
            reference
                .iter()
                .map(|d| (&d.code, &d.message))
                .collect::<Vec<_>>()
        );
    }
}
#[test]
fn dx7_m17_failed_override_retries_and_never_reuses_another_owner() {
    let context = crate::CompilerContext::for_test();
    let original = context.metadata_provider().unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("selected.sifrmeta");
    let overridden = context.clone().with_metadata_override(path.clone());
    assert!(overridden.metadata_provider().is_err());
    std::fs::copy(&original.metadata.path, &path).unwrap();
    let selected = overridden.metadata_provider().unwrap();
    assert_eq!(selected.metadata.metadata_id, original.metadata.metadata_id);
    assert!(!std::sync::Arc::ptr_eq(&selected, &original));
    let corrupted = temporary.path().join("corrupt.sifrmeta");
    std::fs::write(&corrupted, b"incomplete").unwrap();
    assert!(
        context
            .with_metadata_override(corrupted)
            .metadata_provider()
            .is_err()
    );
}

#[test]
fn dx7_navigation_demand_shares_index_and_reads_only_selected_source() {
    let context = crate::CompilerContext::for_test();
    let first = context.stdlib_navigation().unwrap();
    let second = context.stdlib_navigation().unwrap();
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!(first.loaded_sources(), 0);
    let symbol = first
        .symbols
        .iter()
        .find(|symbol| symbol.module == "sifr.math" && symbol.name == "sqrt")
        .unwrap();
    assert!(first.path(symbol.file).unwrap().ends_with("math.sifr"));
    assert!(first.source(symbol.file).unwrap().unwrap().contains("sqrt"));
    assert_eq!(first.loaded_sources(), 1);
    let provider = context.metadata_provider().unwrap();
    assert!(provider.loaded_semantic_modules().is_empty());
    assert_eq!(
        provider
            .metadata
            .store
            .payload_read_count::<wire::HirModule>(),
        0
    );
    assert_eq!(
        provider
            .metadata
            .store
            .payload_read_count::<wire::RustPayload>(),
        0
    );
}

#[test]
fn dx7_m17_metadata_projects_keep_failed_deleted_and_repaired_exports_isolated() {
    use sifr_frontend::{
        FrontendContext, FrontendInput, FrontendMode, ModuleId, SourcePath, SourceText,
    };
    let context = crate::CompilerContext::for_test();
    let defs = crate::stdlib::external_defs(&context).unwrap();
    let source = "from sifr.math import sqrt

def value() -> float:
    return sqrt(4.0)
";
    let make = || {
        FrontendContext::load_single_file_with_external_defs(
            FrontendInput {
                path: SourcePath::new("main.sifr"),
                source: SourceText::new(source),
                mode: FrontendMode::SingleFile,
            },
            defs.clone(),
        )
        .unwrap()
    };
    let mut first = make();
    let mut second = make();
    let module = ModuleId::new(0);
    assert!(
        first
            .diagnostics_for_module(module)
            .value()
            .diagnostics
            .is_empty()
    );
    assert!(
        second
            .diagnostics_for_module(module)
            .value()
            .diagnostics
            .is_empty()
    );
    let provider = context.metadata_provider().unwrap();
    let baseline = provider.semantic("sifr.math").unwrap();
    first
        .update_module_source(
            module,
            SourceText::new(source.replace("sqrt(4.0)", "\"bad\"")),
            None,
        )
        .unwrap();
    assert!(
        !first
            .diagnostics_for_module(module)
            .value()
            .diagnostics
            .is_empty()
    );
    assert!(
        second
            .diagnostics_for_module(module)
            .value()
            .diagnostics
            .is_empty()
    );
    first
        .update_module_source(module, SourceText::new(""), None)
        .unwrap();
    assert!(
        first
            .diagnostics_for_module(module)
            .value()
            .diagnostics
            .is_empty()
    );
    assert!(first.analysis_for_module(module).value().symbols.is_empty());
    assert!(
        !second
            .analysis_for_module(module)
            .value()
            .symbols
            .is_empty()
    );
    first
        .update_module_source(module, SourceText::new(source), None)
        .unwrap();
    assert!(
        first
            .diagnostics_for_module(module)
            .value()
            .diagnostics
            .is_empty()
    );
    assert!(std::sync::Arc::ptr_eq(
        &baseline,
        &provider.semantic("sifr.math").unwrap()
    ));
    assert!(
        defs.functions.is_empty(),
        "the provider handle is not a mutable project"
    );
}
