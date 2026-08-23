use super::PythonBridgeImport;
use super::imports::RawImport;
use std::collections::BTreeSet;

pub(super) fn relative_import_escape(
    imports: &[RawImport],
    module: &str,
    is_package: bool,
) -> Option<String> {
    imports
        .iter()
        .find(|import| {
            matches!(
                import,
                RawImport::From { level, .. }
                    if *level > 0 && relative_base(module, is_package, *level).is_none()
            )
        })
        .map(RawImport::display)
}

pub(super) fn classify_imports(
    module: &str,
    is_package: bool,
    raw_imports: &[RawImport],
    known_modules: &BTreeSet<String>,
) -> Vec<PythonBridgeImport> {
    let mut imports = BTreeSet::new();
    for raw in raw_imports {
        match raw {
            RawImport::Absolute(name) => classify_absolute(name, &mut imports),
            RawImport::From {
                level: 0,
                module: Some(name),
                names,
            } => {
                if name == "bridge" {
                    for imported in names {
                        insert_same_package(&mut imports, imported);
                    }
                } else if let Some(module) = name.strip_prefix("bridge.") {
                    insert_same_package(&mut imports, module);
                    insert_known_children(&mut imports, module, names, known_modules);
                } else {
                    classify_absolute(name, &mut imports);
                }
            }
            RawImport::From {
                level,
                module: imported,
                names,
            } => {
                let base = relative_base(module, is_package, *level);
                let explicit = imported.as_deref().filter(|value| !value.is_empty());
                if let Some(explicit) = explicit {
                    if let Some(target) = join_module(base.as_deref(), Some(explicit)) {
                        insert_same_package(&mut imports, &target);
                        insert_known_children(&mut imports, &target, names, known_modules);
                    }
                } else if let Some(base) = base {
                    insert_same_package(&mut imports, &base);
                    insert_known_children(&mut imports, &base, names, known_modules);
                }
            }
        }
    }
    imports.into_iter().collect()
}

fn insert_known_children(
    imports: &mut BTreeSet<PythonBridgeImport>,
    module: &str,
    names: &[String],
    known_modules: &BTreeSet<String>,
) {
    for imported in names {
        let candidate = format!("{module}.{imported}");
        if known_modules.contains(&candidate) {
            insert_same_package(imports, &candidate);
        }
    }
}

fn classify_absolute(name: &str, imports: &mut BTreeSet<PythonBridgeImport>) {
    if name == "bridge" {
        return;
    }
    if let Some(module) = name.strip_prefix("bridge.") {
        insert_same_package(imports, module);
    } else if let Some(root) = name.split('.').next() {
        if !root.is_empty() {
            imports.insert(PythonBridgeImport::ThirdParty {
                root: root.to_string(),
            });
        }
    }
}

fn insert_same_package(imports: &mut BTreeSet<PythonBridgeImport>, module: &str) {
    let mut prefix = String::new();
    for component in module.split('.') {
        if !prefix.is_empty() {
            prefix.push('.');
        }
        prefix.push_str(component);
        imports.insert(PythonBridgeImport::SamePackage {
            module: prefix.clone(),
        });
    }
}

fn relative_base(module: &str, is_package: bool, level: u32) -> Option<String> {
    if level == 0 {
        return None;
    }
    let mut parts = module.split('.').collect::<Vec<_>>();
    if !is_package {
        let _ = parts.pop();
    }
    for _ in 1..level {
        let _ = parts.pop()?;
    }
    (!parts.is_empty()).then(|| parts.join("."))
}

fn join_module(base: Option<&str>, module: Option<&str>) -> Option<String> {
    match (base, module) {
        (Some(base), Some(module)) => Some(format!("{base}.{module}")),
        (Some(base), None) => Some(base.to_string()),
        (None, Some(module)) => Some(module.to_string()),
        (None, None) => None,
    }
}
