use crate::diag::PackageDiagnostic;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AppTarget {
    pub name: String,
    pub path: PathBuf,
}

pub(super) fn discover_app_targets(
    source_roots: &[PathBuf],
    package_name: &str,
) -> Result<Vec<AppTarget>, PackageDiagnostic> {
    let mut targets = Vec::new();
    for source_root in source_roots {
        let main = source_root.join("main.sifr");
        if main.is_file() {
            targets.push(AppTarget {
                name: package_name.to_string(),
                path: main,
            });
        }
        let bin_root = source_root.join("bin");
        collect_bin_targets(&bin_root, &bin_root, &mut targets)?;
    }
    Ok(targets)
}

fn collect_bin_targets(
    root: &Path,
    current: &Path,
    targets: &mut Vec<AppTarget>,
) -> Result<(), PackageDiagnostic> {
    let Ok(entries) = std::fs::read_dir(current) else {
        return Ok(());
    };
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            collect_bin_targets(root, &path, targets)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "sifr")
        {
            let Some(name) = target_name(root, &path) else {
                continue;
            };
            if !valid_target_name(&name) {
                return Err(PackageDiagnostic::invalid_app_target_name(&name));
            }
            targets.push(AppTarget { name, path });
        }
    }
    targets.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(())
}

fn target_name(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?.with_extension("");
    let parts = relative
        .components()
        .map(|component| component.as_os_str().to_str().map(str::to_string))
        .collect::<Option<Vec<_>>>()?;
    Some(parts.join("/"))
}

fn valid_target_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        })
}
