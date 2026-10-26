//! Shared verification oracle; native assertions remain in the E2E runner.
#![allow(clippy::unwrap_used, clippy::expect_used)]

pub fn full_corpus_exact_emission(root: &std::path::Path) {
    let context = sifr_compiler_services::CompilerContext::for_test();
    let sysroot = context.sysroot().unwrap().clone();
    let sources = sifr_stdlib_manifest::load_stdlib_tooling_sources_from_sysroot(&sysroot).unwrap();
    let source_stdlib =
        sifr_compiler_services::stdlib::compile_stdlib_sources_with_sysroot(&sources, sysroot)
            .unwrap();
    let semantic_target = std::env::var("SIFR_DX8_SEMANTIC_TARGET")
        .unwrap_or_else(|_| sifr_compiler_services::metadata::reader::selection::target().into());
    let prepared = sifr_compiler_services::metadata::ensure_development_metadata(
        context.identity(),
        &context.sysroot().unwrap().root,
        &semantic_target,
        context.cache_root(),
        &|| false,
    )
    .unwrap();
    let provider = sifr_compiler_services::metadata::reader::Provider::new(prepared).unwrap();
    let mut metadata_defs = sifr_lowering::ExternalDefs::default();
    metadata_defs.provider = Some(provider.clone());
    let mut paths = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "sifr"))
        .collect::<Vec<_>>();
    paths.sort();
    assert!(
        paths.len() > 700,
        "maintained corpus unexpectedly empty or incomplete"
    );
    let selected = std::env::var("SIFR_DX8_CORPUS_CASES").ok().map(|text| {
        text.split(',')
            .map(str::to_owned)
            .collect::<std::collections::BTreeSet<_>>()
    });
    if let Some(selected) = &selected {
        assert!(!selected.is_empty());
        let inventory = paths
            .iter()
            .map(|p| p.file_stem().unwrap().to_str().unwrap().to_owned())
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            selected.is_subset(&inventory),
            "unknown selected corpus case"
        );
        paths.retain(|path| selected.contains(path.file_stem().unwrap().to_str().unwrap()));
    }
    let output = std::env::var_os("SIFR_DX8_CORPUS_OUTPUT").map(std::path::PathBuf::from);
    if let Some(output) = &output {
        std::fs::create_dir_all(output).unwrap();
    }
    let mut rows = Vec::new();
    let mut failures = Vec::new();
    for path in paths {
        let source = std::fs::read_to_string(&path).unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();
        let result = (|| -> std::result::Result<serde_json::Value, String> {
            let parsed =
                sifr_frontend::parse_source(&source, None).map_err(|e| format!("{e:?}"))?;
            let reference = sifr_frontend::compile_module_hir_with_source(
                "main",
                &parsed,
                &source_stdlib.defs,
                sifr_frontend::FrontendDiagnosticStyle::Bare,
                Some(sifr_frontend::FrontendSourceContext {
                    display_path: "main",
                    source: &source,
                }),
            )
            .map_err(|e| format!("source: {e:?}"))?;
            let actual = sifr_frontend::compile_module_hir_with_source(
                "main",
                &parsed,
                &metadata_defs,
                sifr_frontend::FrontendDiagnosticStyle::Bare,
                Some(sifr_frontend::FrontendSourceContext {
                    display_path: "main",
                    source: &source,
                }),
            )
            .map_err(|e| format!("metadata: {e:?}"))?;
            let requested = sifr_codegen::stdlib_module_roots(&actual.module);
            let metadata = provider
                .materialize(&requested, context.sysroot().unwrap())
                .map_err(|e| e.to_string())?;
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
            if actual.rust_source != expected.rust_source {
                if let Some(output) = &output {
                    std::fs::write(
                        output.join(format!("{name}.source.rs")),
                        &expected.rust_source,
                    )
                    .unwrap();
                    std::fs::write(
                        output.join(format!("{name}.metadata.rs")),
                        &actual.rust_source,
                    )
                    .unwrap();
                }
                return Err("emitted Rust bytes differ".into());
            }
            assert_eq!(
                actual.used_stdlib_modules, expected.used_stdlib_modules,
                "{name}"
            );
            if let Some(output) = &output {
                std::fs::write(
                    output.join(format!("{name}.source.rs")),
                    &expected.rust_source,
                )
                .unwrap();
                std::fs::write(output.join(format!("{name}.sifr")), &source).unwrap();
            }
            let maps =
                sifr_compiler_services::editor::generated_source_map_files(&actual.rust_source);
            assert_eq!(
                maps,
                sifr_compiler_services::editor::generated_source_map_files(&expected.rust_source)
            );
            let generated_sources = maps
                .iter()
                .map(|file| {
                    serde_json::json!({
                        "path":file.path, "origin":format!("{:?}", file.origin),
                        "source_sha256":sifr_sysroot::sha256_hex(file.source.as_bytes()),
                        "source_bytes":file.source.len()
                    })
                })
                .collect::<Vec<_>>();
            let marker = actual.rust_source.find("// --- stdlib:");
            let end = actual.rust_source.find("\n// --- end stdlib ---");
            assert_eq!(
                marker.is_some(),
                end.is_some(),
                "{name}: source-map marker pair"
            );
            Ok(
                serde_json::json!({"fixture":name,"source_sha256":sifr_sysroot::sha256_hex(source.as_bytes()),
                "rust_sha256":sifr_sysroot::sha256_hex(actual.rust_source.as_bytes()),
                "stdlib_modules":actual.used_stdlib_modules,"generated_source_maps":generated_sources,"preamble_start":marker,"preamble_end":end,
                "final_newline":actual.rust_source.ends_with('\n')}),
            )
        })();
        match result {
            Ok(row) => rows.push(row),
            Err(error) => {
                failures.push(format!("{name}: {error}"));
                rows.push(serde_json::json!({"fixture":name,"error":error}));
            }
        }
    }
    if let Some(output) = &output {
        std::fs::write(output.join("coverage.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "selection":selected,"portable_payload_sha256":provider.metadata.store.portable_payload_digest().unwrap(),"metadata_id":provider.metadata.metadata_id,"semantic_target":semantic_target,"rows":rows,"failures":failures,
            "behavioral_oracle":"maintained native E2E assertions; exact Rust bytes shared by both providers",
            "generic_coverage":"maintained instantiations only"
        })).unwrap()).unwrap();
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
