use super::rust_interop_digest::relative_path_string;
use std::fs;
use std::path::Path;

pub(super) fn unsafe_bridge_files(package: &sifr_package::SifrPackageMetadata) -> Vec<String> {
    let mut files = Vec::new();
    for bridge_root in &package.manifest.rust.bridges {
        let root = package.package_root.join(bridge_root);
        collect_unsafe_bridge_files(&package.package_root, &root, &mut files);
    }
    files.sort();
    files.dedup();
    files
}

fn collect_unsafe_bridge_files(package_root: &Path, path: &Path, files: &mut Vec<String>) {
    if path.is_file() {
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            return;
        }
        let Ok(source) = fs::read_to_string(path) else {
            return;
        };
        if source.contains("unsafe") {
            files.push(relative_path_string(package_root, path));
        }
        return;
    }
    let Ok(read_dir) = fs::read_dir(path) else {
        return;
    };
    for entry in read_dir.flatten() {
        collect_unsafe_bridge_files(package_root, &entry.path(), files);
    }
}
