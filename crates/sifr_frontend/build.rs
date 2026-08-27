use sha2::{Digest as _, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const REVISION_ENV: &str = "SIFR_SOURCE_REVISION";
const COMPILER_SOURCE_ROOTS: &[&str] = &[
    "crates/sifr_source",
    "crates/sifr_diagnostics",
    "crates/sifr_type_system",
    "crates/sifr_ir",
    "crates/sifr_lowering",
    "crates/sifr_syntax",
    "crates/sifr_frontend",
];

fn main() {
    println!("cargo:rerun-if-env-changed={REVISION_ENV}");
    let revision = std::env::var(REVISION_ENV).unwrap_or_else(|_| source_revision());
    println!("cargo:rustc-env=SIFR_FRONTEND_BUILD_REVISION={revision}");
}

fn source_revision() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repository_root = manifest_dir
        .ancestors()
        .nth(2)
        .map_or_else(|| manifest_dir.clone(), Path::to_path_buf);
    let mut files = Vec::new();
    for relative in COMPILER_SOURCE_ROOTS {
        collect_source_files(&repository_root.join(relative), &mut files);
    }
    for relative in ["Cargo.toml", "Cargo.lock"] {
        let path = repository_root.join(relative);
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();

    let mut digest = Sha256::new();
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let relative = path.strip_prefix(&repository_root).unwrap_or(&path);
        update_framed(&mut digest, relative.to_string_lossy().as_bytes());
        if let Ok(bytes) = fs::read(&path) {
            update_framed(&mut digest, &bytes);
        }
    }
    format!("source-sha256:{}", lower_hex(&digest.finalize()))
}

fn collect_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_source_files(&path, files);
        } else if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("rs" | "toml")
        ) {
            files.push(path);
        }
    }
}

fn update_framed(digest: &mut Sha256, value: &[u8]) {
    digest.update(value.len().to_le_bytes());
    digest.update(value);
}

fn lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}
