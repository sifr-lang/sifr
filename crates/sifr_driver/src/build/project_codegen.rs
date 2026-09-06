use super::python_bridges::embedded_bridge_sources;
use super::python_runtime::{PackagePythonRuntime, inject_python_runtime_bootstrap};
use crate::diagnostics::{RenderedDiagnostic, run_codegen_with_boundary};
use crate::frontend::FrontendCompiled;
use crate::project::{
    ProjectLowering, assemble_project_main_rs, ordered_non_main_module_names, rust_module_file_path,
};
use sifr_codegen::{
    StdlibCode, generate_rust_multi_with_metadata, generate_rust_with_stdlib_for_module,
};
use sifr_ir::HirModule;
use sifr_stdlib_manifest::StdlibFeature;
use std::collections::{BTreeMap, HashSet};

pub(super) struct GeneratedBinaryProject {
    pub(super) main_rs: String,
    pub(super) support_modules: BTreeMap<String, String>,
    pub(super) bridge_modules: BTreeMap<String, String>,
    pub(super) used_stdlib_modules: HashSet<String>,
    pub(super) required_features: HashSet<StdlibFeature>,
    pub(super) interop: sifr_codegen::InteropBuildPlan,
    pub(super) cache_key_fragment: Option<String>,
    pub(super) python_runtime: Option<PackagePythonRuntime>,
}

impl GeneratedBinaryProject {
    pub(super) fn emit_source_listing(&self) -> String {
        let mut listing = String::new();
        listing.push_str("// src/main.rs\n");
        listing.push_str(&self.main_rs);
        if !self.main_rs.ends_with('\n') {
            listing.push('\n');
        }
        for (module_name, code) in &self.support_modules {
            listing.push_str("\n// src/");
            listing.push_str(&rust_module_file_path(module_name).display().to_string());
            listing.push('\n');
            listing.push_str(code);
            if !code.ends_with('\n') {
                listing.push('\n');
            }
        }
        listing
    }
}

pub(super) fn codegen_single_file_frontend(
    frontend: &FrontendCompiled,
) -> Result<sifr_codegen::CodegenResult, Vec<RenderedDiagnostic>> {
    let static_programs = frontend.lowering_result.specialization_outputs.clone();
    let mut generated = run_codegen_with_boundary(
        "internal compiler panic during single-file code generation",
        || {
            generate_rust_with_stdlib_for_module(
                &frontend.lowering_result.module,
                &frontend.stdlib.code,
                Some("main"),
            )
        },
    )
    .map_err(|error| vec![*error])?;
    generated.static_programs = static_programs;
    if generated
        .interop
        .rust
        .structural_identity_algorithm_version
        .is_some()
    {
        generated.static_program_structural_owners =
            sifr_codegen::structural_static_program_owners(&frontend.lowering_result.module);
    }
    Ok(generated)
}

pub(super) fn format_generated_binary_project(
    mut generated: GeneratedBinaryProject,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    generated.bridge_modules = super::rust_interop_bridge_sources::generated_bridge_sources(
        &generated.interop.rust.bridge_contracts.generated_types,
    )
    .map_err(|message| {
        vec![crate::diagnostics::diagnostic_with_code(
            message,
            sifr_diagnostics::DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
        )]
    })?;
    super::rust_formatter::canonicalize_project_fields(
        &mut generated.main_rs,
        generated
            .support_modules
            .iter_mut()
            .chain(generated.bridge_modules.iter_mut()),
    )?;
    generated.main_rs = super::rust_formatter::format_canonical_generated_rust(
        &generated.main_rs,
        "project main.rs",
    )?;
    for (module_name, source) in generated
        .support_modules
        .iter_mut()
        .chain(generated.bridge_modules.iter_mut())
    {
        let label = format!("project module {module_name}");
        *source = super::rust_formatter::format_canonical_generated_rust(source, &label)?;
    }
    loop {
        let before = super::rust_formatter::discover_project_const_functions(
            std::iter::once(generated.main_rs.as_str()).chain(
                generated
                    .support_modules
                    .values()
                    .chain(generated.bridge_modules.values())
                    .map(String::as_str),
            ),
        )?;
        generated.main_rs = super::rust_formatter::format_generated_rust_with_project_consts(
            &generated.main_rs,
            "project main.rs",
            &before,
        )?;
        for (module_name, source) in generated
            .support_modules
            .iter_mut()
            .chain(generated.bridge_modules.iter_mut())
        {
            let label = format!("project module {module_name}");
            *source = super::rust_formatter::format_generated_rust_with_project_consts(
                source, &label, &before,
            )?;
        }
        let after = super::rust_formatter::discover_project_const_functions(
            std::iter::once(generated.main_rs.as_str()).chain(
                generated
                    .support_modules
                    .values()
                    .chain(generated.bridge_modules.values())
                    .map(String::as_str),
            ),
        )?;
        if after == before {
            return Ok(generated);
        }
        if !after.is_superset(&before) {
            return Err(vec![crate::diagnostics::diagnostic_with_code(
                "generated project const API discovery was not monotonic".to_string(),
                sifr_diagnostics::DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
            )]);
        }
    }
}

pub(super) fn generated_single_file_binary_project(
    mut codegen_result: sifr_codegen::CodegenResult,
) -> GeneratedBinaryProject {
    let static_source = sifr_codegen::emit_static_specialization_programs(
        &codegen_result.static_programs,
        &codegen_result.static_program_structural_owners,
    );
    if !static_source.is_empty() {
        codegen_result.rust_source = format!("{static_source}\n{}", codegen_result.rust_source);
    }
    let static_cache = sifr_codegen::static_program_cache_fragment(&codegen_result.static_programs);
    let mut cache_key_fragment = None;
    if !static_cache.is_empty() {
        push_cache_key_fragment(&mut cache_key_fragment, "static-programs", &static_cache);
    }
    let slot_cache = sifr_codegen::method_slot_cache_fragment(&codegen_result.static_programs);
    if !slot_cache.is_empty() {
        push_cache_key_fragment(&mut cache_key_fragment, "slot-tables", &slot_cache);
    }
    GeneratedBinaryProject {
        main_rs: codegen_result.rust_source,
        support_modules: BTreeMap::new(),
        used_stdlib_modules: codegen_result.used_stdlib_modules,
        required_features: codegen_result.required_features,
        interop: codegen_result.interop,
        cache_key_fragment,
        bridge_modules: BTreeMap::new(),
        python_runtime: None,
    }
}

pub(super) fn generated_project_binary_project(
    stdlib_code: &StdlibCode,
    project_lowering: ProjectLowering,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    let ProjectLowering {
        hir_modules,
        external_defs,
        compile_order,
        ..
    } = project_lowering;
    let static_programs = external_defs.specialization_outputs;
    let module_refs: Vec<(&str, &HirModule)> = compile_order
        .iter()
        .filter_map(|module_name| {
            hir_modules
                .get(module_name)
                .map(|module| (module_name.as_str(), module))
        })
        .collect();
    let mut codegen_result = run_codegen_with_boundary(
        "internal compiler panic during project code generation",
        || generate_rust_multi_with_metadata(&module_refs, stdlib_code),
    )
    .map_err(|error| vec![*error])?;
    let structural_programs = codegen_result
        .interop
        .rust
        .structural_identity_algorithm_version
        .is_some();
    let mut static_cache = Vec::new();
    for (module_name, outputs) in &static_programs {
        let structural_owners = if structural_programs {
            hir_modules
                .get(module_name)
                .map(|module| {
                    sifr_codegen::structural_static_program_owners_for_project(module, &module_refs)
                })
                .unwrap_or_default()
        } else {
            Default::default()
        };
        let static_source =
            sifr_codegen::emit_static_specialization_programs(outputs, &structural_owners);
        if static_source.is_empty() {
            continue;
        }
        if let Some(source) = codegen_result.rust_files.get_mut(module_name) {
            *source = format!("{static_source}\n{source}");
        }
        static_cache.extend(outputs.iter().cloned());
    }

    let slot_cache = sifr_codegen::method_slot_cache_fragment(&static_cache);

    let generated_main_rs = assemble_project_main_rs(&compile_order, &codegen_result.rust_files);
    let main_rs = if codegen_result.project_union_prelude.is_empty() {
        generated_main_rs
    } else {
        format!(
            "{}\n{generated_main_rs}",
            codegen_result.project_union_prelude.trim_end()
        )
    };
    let support_modules = ordered_non_main_module_names(&compile_order, &codegen_result.rust_files)
        .into_iter()
        .filter_map(|module_name| {
            codegen_result
                .rust_files
                .get(module_name.as_str())
                .map(|code| (module_name, code.clone()))
        })
        .collect();

    let static_cache = sifr_codegen::static_program_cache_fragment(&static_cache);
    let mut cache_key_fragment = None;
    if !static_cache.is_empty() {
        push_cache_key_fragment(&mut cache_key_fragment, "static-programs", &static_cache);
    }
    if !slot_cache.is_empty() {
        push_cache_key_fragment(&mut cache_key_fragment, "slot-tables", &slot_cache);
    }

    Ok(GeneratedBinaryProject {
        main_rs,
        support_modules,
        used_stdlib_modules: codegen_result.used_stdlib_modules,
        required_features: codegen_result.required_features,
        interop: codegen_result.interop,
        cache_key_fragment,
        bridge_modules: BTreeMap::new(),
        python_runtime: None,
    })
}

pub(super) fn apply_package_runtime_metadata(
    mut generated: GeneratedBinaryProject,
    python_runtime: Option<PackagePythonRuntime>,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    if let Some(mut metadata) = python_runtime {
        metadata.set_start_async_loop(generated.interop.python.requires_async_loop);
        metadata.set_bridge_sources(embedded_bridge_sources(
            &generated.interop.python.bridge_packages,
        ));
        generated
            .required_features
            .insert(StdlibFeature::PythonRuntime);
        generated.main_rs = inject_python_runtime_bootstrap(&generated.main_rs, &metadata)
            .map_err(|message| {
                vec![crate::diagnostics::diagnostic_with_code(
                    message,
                    sifr_diagnostics::DiagnosticCode::INTERNAL_COMPILER_PANIC,
                )]
            })?;
        push_cache_key_fragment(
            &mut generated.cache_key_fragment,
            "python-runtime",
            metadata.probe_digest(),
        );
        if !metadata.arrow_certification_identity().is_empty() {
            push_cache_key_fragment(
                &mut generated.cache_key_fragment,
                "python-arrow-certifications",
                metadata.arrow_certification_identity(),
            );
        }
        if !metadata.dlpack_certification_identity().is_empty() {
            push_cache_key_fragment(
                &mut generated.cache_key_fragment,
                "python-dlpack-certifications",
                metadata.dlpack_certification_identity(),
            );
        }
        if !metadata.binding_identity().is_empty() {
            push_cache_key_fragment(
                &mut generated.cache_key_fragment,
                "python-bindings",
                metadata.binding_identity(),
            );
        }
        generated.python_runtime = Some(metadata);
    }
    Ok(generated)
}

pub(super) fn attach_package_runtime_metadata_for_check(
    mut generated: GeneratedBinaryProject,
    python_runtime: Option<PackagePythonRuntime>,
) -> GeneratedBinaryProject {
    generated.python_runtime = python_runtime;
    generated
}

pub(super) fn push_cache_key_fragment(fragment: &mut Option<String>, label: &str, value: &str) {
    let mut next = fragment.take().unwrap_or_default();
    next.push('[');
    next.push_str(label);
    next.push_str("]\n");
    next.push_str(value);
    if !value.ends_with('\n') {
        next.push('\n');
    }
    *fragment = Some(next);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_project() -> GeneratedBinaryProject {
        GeneratedBinaryProject {
            main_rs: "fn main() {\n    println!(\"ok\");\n}\n".to_string(),
            support_modules: BTreeMap::new(),
            used_stdlib_modules: HashSet::new(),
            required_features: HashSet::new(),
            interop: sifr_codegen::InteropBuildPlan::default(),
            cache_key_fragment: None,
            bridge_modules: BTreeMap::new(),
            python_runtime: None,
        }
    }

    #[test]
    fn package_python_runtime_metadata_enables_feature_and_bootstrap() {
        let metadata = PackagePythonRuntime::for_tests("/tmp/sifr-py/bin/python", "digest-a");

        let generated = apply_package_runtime_metadata(base_project(), Some(metadata))
            .expect("metadata should apply");

        assert!(
            generated
                .required_features
                .contains(&StdlibFeature::PythonRuntime)
        );
        assert_eq!(
            generated.cache_key_fragment.as_deref(),
            Some("[python-runtime]\ndigest-a\n")
        );
        assert!(
            generated
                .main_rs
                .contains("__sifr_initialize_python_runtime")
        );
        assert!(generated.python_runtime.is_some());
    }

    #[test]
    fn package_python_runtime_starts_owned_loop_only_when_planned() {
        let mut project = base_project();
        project.interop.python.requires_async_loop = true;
        let generated = apply_package_runtime_metadata(
            project,
            Some(PackagePythonRuntime::for_tests(
                "/tmp/sifr-py/bin/python",
                "digest-a",
            )),
        )
        .expect("metadata should apply");
        assert!(generated.main_rs.contains("start_async_loop: true"));

        let generated = apply_package_runtime_metadata(
            base_project(),
            Some(PackagePythonRuntime::for_tests(
                "/tmp/sifr-py/bin/python",
                "digest-a",
            )),
        )
        .expect("metadata should apply");
        assert!(generated.main_rs.contains("start_async_loop: false"));
    }

    #[test]
    fn arrow_certification_changes_package_cache_identity() {
        let certification = |producer_type: &str| sifr_package::ArrowCertification {
            target: "pyarrow.array".to_string(),
            kind: sifr_package::ArrowCertifiedKind::Array,
            fixture: "fixtures/arrow.py".to_string(),
            fixture_digest: "fixture-digest".to_string(),
            producer_module: "pyarrow.lib".to_string(),
            producer_type: producer_type.to_string(),
            distributions: vec![sifr_package::ArrowCertifiedDistribution {
                name: "pyarrow".to_string(),
                version: "25.0.1".to_string(),
            }],
            schema_mode: sifr_package::ArrowCertifiedSchemaMode::Omitted,
            identity_method: sifr_package::ArrowCertifiedIdentityMethod::BufferAddress,
            pointer_identity_verified: true,
            exact_release_count: 1,
            copy_performed: false,
        };
        let mut first = PackagePythonRuntime::for_tests("/tmp/sifr-py/bin/python", "digest-a");
        first.set_arrow_certifications(vec![certification("Int64Array")]);
        let first = apply_package_runtime_metadata(base_project(), Some(first))
            .expect("metadata should apply")
            .cache_key_fragment;
        let mut second = PackagePythonRuntime::for_tests("/tmp/sifr-py/bin/python", "digest-a");
        second.set_arrow_certifications(vec![certification("StringArray")]);
        let second = apply_package_runtime_metadata(base_project(), Some(second))
            .expect("metadata should apply")
            .cache_key_fragment;

        assert_ne!(first, second);
    }

    #[test]
    fn binding_artifact_changes_package_cache_identity() {
        let mut first = PackagePythonRuntime::for_tests("/tmp/sifr-py/bin/python", "digest-a");
        first.set_binding_identity("binding-a".to_string());
        let mut second = PackagePythonRuntime::for_tests("/tmp/sifr-py/bin/python", "digest-a");
        second.set_binding_identity("binding-b".to_string());

        let first = apply_package_runtime_metadata(base_project(), Some(first))
            .expect("first metadata should apply");
        let second = apply_package_runtime_metadata(base_project(), Some(second))
            .expect("second metadata should apply");
        assert_ne!(first.cache_key_fragment, second.cache_key_fragment);
        assert!(
            first
                .cache_key_fragment
                .as_deref()
                .is_some_and(|value| value.contains("[python-bindings]\nbinding-a"))
        );
    }

    #[test]
    fn package_python_runtime_metadata_requires_main_function() {
        let mut project = base_project();
        project.main_rs = "fn helper() {}\n".to_string();

        let result = apply_package_runtime_metadata(
            project,
            Some(PackagePythonRuntime::for_tests(
                "/tmp/sifr-py/bin/python",
                "digest-a",
            )),
        );
        let Err(errors) = result else {
            panic!("missing main should fail");
        };

        assert!(errors[0].message.contains("no main function"));
    }
}
