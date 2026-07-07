use crate::build::project_codegen::GeneratedBinaryProject;
use sifr_codegen::RustInteropResolvedRoot;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn package_bridge_dependency_name(
    package: &sifr_package::SifrPackageMetadata,
) -> String {
    let mut dependency = "__sifr_bridge_package_".to_string();
    for ch in package.cargo_package_name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            dependency.push(ch);
        } else {
            dependency.push('_');
        }
    }
    dependency
}

pub(super) fn inject_package_bridge_aliases(generated: &mut GeneratedBinaryProject) {
    let mut aliases_by_module: BTreeMap<Option<String>, BTreeSet<String>> = BTreeMap::new();
    for target in &generated.interop.rust.resolved_targets {
        let RustInteropResolvedRoot::PackageBridge {
            dependency_name, ..
        } = &target.root
        else {
            continue;
        };
        aliases_by_module
            .entry(target.module_name.clone())
            .or_default()
            .insert(dependency_name.clone());
    }

    for (module_name, dependencies) in aliases_by_module {
        // Bridge roots are resolved from the declaring module's owning package, so a
        // generated module can only need one package bridge alias. The set dedupes
        // repeated bridge declarations from that same module/package.
        let prefix = bridge_alias_prefix(&dependencies);
        if prefix.is_empty() {
            continue;
        }
        match module_name.as_deref() {
            None | Some("main") => prepend_once(&mut generated.main_rs, &prefix),
            Some(module_name) => {
                if let Some(source) = generated.support_modules.get_mut(module_name) {
                    prepend_once(source, &prefix);
                }
            }
        }
    }
}

fn bridge_alias_prefix(dependencies: &BTreeSet<String>) -> String {
    let mut prefix = String::new();
    for dependency in dependencies {
        prefix.push_str("use ");
        prefix.push_str(dependency);
        prefix.push_str("::bridges as bridge;\n");
    }
    prefix
}

fn prepend_once(source: &mut String, prefix: &str) {
    if source.starts_with(prefix) {
        return;
    }
    *source = format!("{prefix}{source}");
}
