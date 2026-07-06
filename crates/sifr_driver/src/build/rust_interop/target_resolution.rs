use sifr_codegen::{RustInteropOwner, RustInteropPlanDeclaration, RustInteropTrustRequirementKind};
use sifr_ir::{RustInteropDeclaration, RustInteropValue, RustTargetPath};
use sifr_package::{BackendCrateMetadata, SifrPackageGraph, SifrPackageId};

pub(super) fn declaration_paths(declaration: &RustInteropDeclaration) -> Vec<&RustTargetPath> {
    let mut paths = Vec::new();
    if let Some(target) = &declaration.target {
        paths.push(target);
    }
    for argument in &declaration.arguments {
        collect_value_paths(&argument.value, &mut paths);
    }
    paths
}

fn collect_value_paths<'a>(value: &'a RustInteropValue, paths: &mut Vec<&'a RustTargetPath>) {
    match value {
        RustInteropValue::TargetPath(path) => paths.push(path),
        RustInteropValue::PolicyCall { argument, .. } => collect_value_paths(argument, paths),
        RustInteropValue::Boolean(_)
        | RustInteropValue::Symbol(_)
        | RustInteropValue::Integer(_)
        | RustInteropValue::IntegerList(_) => {}
    }
}

pub(super) fn backend_for_root<'a>(
    graph: &'a SifrPackageGraph,
    package_id: &SifrPackageId,
    root: &str,
) -> Option<&'a BackendCrateMetadata> {
    graph
        .backend_crates
        .get(package_id)?
        .iter()
        .find(|backend| backend.dependency_name == root)
}

pub(super) fn canonical_sifr_target_path(declaration: &RustInteropPlanDeclaration) -> String {
    let mut path = declaration
        .module_name
        .clone()
        .unwrap_or_else(|| "main".to_string());
    match &declaration.owner {
        RustInteropOwner::Function { name } => {
            path.push('.');
            path.push_str(name);
        }
        RustInteropOwner::Class { name } => {
            path.push('.');
            path.push_str(name);
        }
        RustInteropOwner::Method { class_name, name } => {
            path.push('.');
            path.push_str(class_name);
            path.push('.');
            path.push_str(name);
        }
    }
    path
}

pub(super) fn canonical_trust_target_path(declaration: &RustInteropPlanDeclaration) -> String {
    declaration_paths(&declaration.declaration)
        .first()
        .map(|path| path.dotted())
        .unwrap_or_else(|| canonical_sifr_target_path(declaration))
}

pub(super) fn uses_bridge_root(declaration: &RustInteropDeclaration) -> bool {
    declaration_paths(declaration)
        .iter()
        .any(|path| path.segments.first().is_some_and(|root| root == "bridge"))
}

pub(super) fn trust_kind_name(kind: &RustInteropTrustRequirementKind) -> &'static str {
    match kind {
        RustInteropTrustRequirementKind::BuildScript => "build_script",
        RustInteropTrustRequirementKind::ProcMacro => "proc_macro",
        RustInteropTrustRequirementKind::NativeLinks => "native_links",
        RustInteropTrustRequirementKind::BuildEnv => "build_env",
        RustInteropTrustRequirementKind::UnsafeBridge => "unsafe_bridge",
        RustInteropTrustRequirementKind::NoPanic => "no_panic",
        RustInteropTrustRequirementKind::PanicAbort => "panic_abort",
    }
}
