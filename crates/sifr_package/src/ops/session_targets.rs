use crate::diag::PackageDiagnostic;
use sifr_frontend::SourceProvider;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AppTarget {
    pub name: String,
    pub path: PathBuf,
}

pub(super) fn discover_app_targets(
    source_root: &Path,
    package_name: &str,
    provider: &mut impl SourceProvider,
) -> Result<Vec<AppTarget>, PackageDiagnostic> {
    let mut targets = Vec::new();
    let main = source_root.join("main.sifr");
    if provider.is_file(&main) {
        targets.push(AppTarget {
            name: package_name.to_string(),
            path: main,
        });
    }
    let bin_root = source_root.join("bin");
    collect_bin_targets(&bin_root, &bin_root, &mut targets, provider)?;
    Ok(targets)
}

fn collect_bin_targets(
    root: &Path,
    current: &Path,
    targets: &mut Vec<AppTarget>,
    provider: &mut impl SourceProvider,
) -> Result<(), PackageDiagnostic> {
    let Ok(entries) = provider.read_dir(current) else {
        return Ok(());
    };
    for entry in entries {
        let path = entry.path;
        if entry.is_dir {
            collect_bin_targets(root, &path, targets, provider)?;
        } else if entry.is_file
            && path
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
