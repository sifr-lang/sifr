use super::python_bridges::embedded_bridge_sources;
use super::python_runtime::{inject_python_runtime_bootstrap, PackagePythonRuntime};
use crate::diagnostics::{run_codegen_with_boundary, RenderedDiagnostic};
use crate::project::{
    assemble_project_main_rs, ordered_non_main_module_names, rust_module_file_path, ProjectLowering,
};
use sifr_codegen::{generate_rust_multi_with_metadata, StdlibCode};
use sifr_ir::HirModule;
use sifr_stdlib_manifest::StdlibFeature;
use std::collections::{BTreeMap, HashSet};

pub(super) struct GeneratedBinaryProject {
    pub(super) main_rs: String,
    pub(super) support_modules: BTreeMap<String, String>,
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

pub(super) fn generated_single_file_binary_project(
    codegen_result: sifr_codegen::CodegenResult,
) -> GeneratedBinaryProject {
    GeneratedBinaryProject {
        main_rs: codegen_result.rust_source,
        support_modules: BTreeMap::new(),
        used_stdlib_modules: codegen_result.used_stdlib_modules,
        required_features: codegen_result.required_features,
        interop: codegen_result.interop,
        cache_key_fragment: None,
        python_runtime: None,
    }
}

pub(super) fn generated_project_binary_project(
    stdlib_code: &StdlibCode,
    project_lowering: ProjectLowering,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    let ProjectLowering {
        hir_modules,
        compile_order,
        ..
    } = project_lowering;
    let module_refs: Vec<(&str, &HirModule)> = compile_order
        .iter()
        .filter_map(|module_name| {
            hir_modules
                .get(module_name)
                .map(|module| (module_name.as_str(), module))
        })
        .collect();
    let codegen_result = run_codegen_with_boundary(
        "internal compiler panic during project code generation",
        || generate_rust_multi_with_metadata(&module_refs, stdlib_code),
    )
    .map_err(|error| vec![*error])?;

    let main_rs = assemble_project_main_rs(&compile_order, &codegen_result.rust_files);
    let support_modules = ordered_non_main_module_names(&compile_order, &codegen_result.rust_files)
        .into_iter()
        .filter_map(|module_name| {
            codegen_result
                .rust_files
                .get(module_name.as_str())
                .map(|code| (module_name, code.clone()))
        })
        .collect();

    Ok(GeneratedBinaryProject {
        main_rs,
        support_modules,
        used_stdlib_modules: codegen_result.used_stdlib_modules,
        required_features: codegen_result.required_features,
        interop: codegen_result.interop,
        cache_key_fragment: None,
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
        generated.python_runtime = Some(metadata);
    }
    Ok(generated)
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
            python_runtime: None,
        }
    }

    #[test]
    fn package_python_runtime_metadata_enables_feature_and_bootstrap() {
        let metadata = PackagePythonRuntime::for_tests("/tmp/sifr-py/bin/python", "digest-a");

        let generated = apply_package_runtime_metadata(base_project(), Some(metadata))
            .expect("metadata should apply");

        assert!(generated
            .required_features
            .contains(&StdlibFeature::PythonRuntime));
        assert_eq!(
            generated.cache_key_fragment.as_deref(),
            Some("[python-runtime]\ndigest-a\n")
        );
        assert!(generated
            .main_rs
            .contains("__sifr_initialize_python_runtime"));
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
                version: "22.0.0".to_string(),
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
