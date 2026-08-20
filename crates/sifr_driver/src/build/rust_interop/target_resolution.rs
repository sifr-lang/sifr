use sifr_codegen::{RustInteropOwner, RustInteropPlanDeclaration, RustInteropTrustRequirementKind};
use sifr_ir::{RustInteropDeclaration, RustInteropDecoratorKind, RustInteropValue, RustTargetPath};
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

pub(super) fn is_primary_target(
    declaration: &RustInteropDeclaration,
    path: &RustTargetPath,
) -> bool {
    declaration
        .target
        .as_ref()
        .is_some_and(|target| target.span == path.span && target.segments == path.segments)
}

pub(super) fn is_concrete_probe_path(
    declaration: &RustInteropDeclaration,
    path: &RustTargetPath,
) -> bool {
    if declaration.kind != RustInteropDecoratorKind::Opaque {
        return true;
    }

    declaration.arguments.iter().any(|argument| {
        argument.name.as_deref() == Some("type")
            && matches!(
                &argument.value,
                RustInteropValue::TargetPath(type_path) if std::ptr::eq(type_path, path)
            )
    })
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

pub(super) fn trust_allowlist_name(kind: &RustInteropTrustRequirementKind) -> &'static str {
    match kind {
        RustInteropTrustRequirementKind::BuildScript => "rust-build-scripts",
        RustInteropTrustRequirementKind::ProcMacro => "rust-proc-macros",
        RustInteropTrustRequirementKind::NativeLinks => "native-links",
        RustInteropTrustRequirementKind::BuildEnv => "build-env",
        RustInteropTrustRequirementKind::UnsafeBridge => "unsafe-rust-bridges",
        RustInteropTrustRequirementKind::NoPanic => "rust-no-panic",
        RustInteropTrustRequirementKind::PanicAbort => "rust-panic-abort",
    }
}

#[cfg(test)]
mod tests {
    use super::{declaration_paths, is_concrete_probe_path};
    use ruff_text_size::TextRange;
    use sifr_ir::{
        RustInteropAbiRequirements, RustInteropArgument, RustInteropDeclaration,
        RustInteropDecoratorKind, RustInteropEffect, RustInteropValue, RustTargetPath,
    };

    fn target_path(path: &str) -> RustTargetPath {
        RustTargetPath {
            segments: path.split('.').map(str::to_string).collect(),
            span: TextRange::default(),
        }
    }

    fn opaque_declaration(type_path: &str, mapping_path: &str) -> RustInteropDeclaration {
        RustInteropDeclaration {
            kind: RustInteropDecoratorKind::Opaque,
            target: None,
            arguments: vec![
                RustInteropArgument {
                    name: Some("type".to_string()),
                    value: RustInteropValue::TargetPath(target_path(type_path)),
                    span: TextRange::default(),
                },
                RustInteropArgument {
                    name: Some("structural".to_string()),
                    value: RustInteropValue::TargetPath(target_path(mapping_path)),
                    span: TextRange::default(),
                },
            ],
            span: TextRange::default(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements::default(),
            consumes_receiver: false,
        }
    }

    #[test]
    fn opaque_probe_selects_only_the_declared_value_type() {
        let declaration = opaque_declaration("bridge.Value", "bridge.Mapping");
        let paths = declaration_paths(&declaration);

        assert!(is_concrete_probe_path(&declaration, paths[0]));
        assert!(!is_concrete_probe_path(&declaration, paths[1]));
    }

    #[test]
    fn opaque_probe_does_not_duplicate_equal_type_and_mapping_paths() {
        let declaration = opaque_declaration("bridge.Value", "bridge.Value");
        let paths = declaration_paths(&declaration);

        assert!(is_concrete_probe_path(&declaration, paths[0]));
        assert!(!is_concrete_probe_path(&declaration, paths[1]));
    }
}
