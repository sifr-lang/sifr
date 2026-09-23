use super::project_codegen::GeneratedBinaryProject;
use crate::diagnostics::RenderedDiagnostic;
use sifr_codegen::{InteropBuildPlan, RustInteropTrustRequirementKind};
use sifr_diagnostics::DiagnosticCode;
use sifr_stdlib_manifest::{SysrootCrate, SysrootDependencyPlan};
use std::collections::BTreeSet;

pub(crate) fn validate_test_native_link_evidence(
    stdout: &[u8],
    interop: &InteropBuildPlan,
    dependency_plan: &SysrootDependencyPlan,
) -> Result<(), Vec<RenderedDiagnostic>> {
    if !should_validate_interop_native_link_evidence(interop) {
        return Ok(());
    }
    validate_native_link_evidence(
        stdout,
        &trusted_native_links_for_interop(interop, dependency_plan),
    )
}

pub(super) fn trusted_native_links(
    generated_project: &GeneratedBinaryProject,
    dependency_plan: &SysrootDependencyPlan,
) -> BTreeSet<String> {
    let mut trusted = trusted_native_links_for_interop(&generated_project.interop, dependency_plan);
    if let Some(python_runtime) = &generated_project.python_runtime {
        trusted.extend(python_runtime.trusted_native_link_names());
    }
    trusted
}

fn trusted_native_links_for_interop(
    interop: &InteropBuildPlan,
    dependency_plan: &SysrootDependencyPlan,
) -> BTreeSet<String> {
    let mut trusted = interop
        .rust
        .trust_requirements
        .iter()
        .filter(|requirement| {
            requirement.trusted && requirement.kind == RustInteropTrustRequirementKind::NativeLinks
        })
        .map(|requirement| requirement.required_entry.clone())
        .collect::<BTreeSet<_>>();
    trusted.extend(sysroot_trusted_native_links(dependency_plan));
    trusted
}

pub(super) fn sysroot_trusted_native_links(
    dependency_plan: &SysrootDependencyPlan,
) -> BTreeSet<String> {
    let tls_selected = dependency_plan.crates.iter().any(|dependency| {
        matches!(
            dependency.krate,
            SysrootCrate::SifrRuntime | SysrootCrate::SifrStdlib
        ) && (dependency.features.contains("tls") || dependency.features.contains("http"))
    });
    if tls_selected {
        return BTreeSet::from(["aws_lc_0_44_0_crypto".to_string()]);
    }
    BTreeSet::new()
}

pub(super) fn should_validate_native_link_evidence(
    generated_project: &GeneratedBinaryProject,
) -> bool {
    should_validate_interop_native_link_evidence(&generated_project.interop)
}

fn should_validate_interop_native_link_evidence(interop: &InteropBuildPlan) -> bool {
    let rust = &interop.rust;
    !rust.declarations.is_empty()
        || !rust.resolved_targets.is_empty()
        || !rust.trust_requirements.is_empty()
        || !rust.probe_plan.probes.is_empty()
        || !rust.bridge_sources.is_empty()
        || rust.cargo_inputs.is_some()
}

pub(super) fn validate_native_link_evidence(
    stdout: &[u8],
    trusted_native_links: &BTreeSet<String>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    for line in String::from_utf8_lossy(stdout).lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if value.get("reason").and_then(serde_json::Value::as_str) != Some("build-script-executed")
        {
            continue;
        }
        let Some(linked_libs) = value
            .get("linked_libs")
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        for linked_lib in linked_libs {
            let Some(linked_lib) = linked_lib.as_str() else {
                continue;
            };
            let link_name = normalized_link_name(linked_lib);
            if !trusted_native_links.contains(&link_name) {
                return Err(vec![crate::diagnostics::diagnostic_with_code(
                    format!(
                        "untrusted native link evidence `{link_name}` emitted by Rust build script"
                    ),
                    DiagnosticCode::RUST_TRUST_MISSING,
                )]);
            }
        }
    }
    Ok(())
}

fn normalized_link_name(linked_lib: &str) -> String {
    linked_lib
        .rsplit_once('=')
        .map_or(linked_lib, |(_, name)| name)
        .to_string()
}
