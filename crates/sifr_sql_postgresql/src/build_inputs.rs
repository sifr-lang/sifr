//! Cargo inputs for the native parser, including headers and generated C sources.
use std::{fs, io, path::Path};

pub(super) fn track_source(source: &Path) -> io::Result<()> {
    for directory in ["src", "vendor", "protobuf"] {
        println!(
            "cargo:rerun-if-changed={}",
            source.join(directory).display()
        );
    }
    let mut entries = fs::read_dir(source)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort();
    for path in entries {
        // Root headers differ between pinned majors. Git metadata is not a C input.
        if path.is_file() && path.file_name().is_some_and(|name| name != ".git") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    Ok(())
}

pub(super) fn track_wasi(compatibility_headers: &Path) {
    println!("cargo:rerun-if-changed={}", compatibility_headers.display());
    println!("cargo:rerun-if-env-changed=WASI_SDK_PATH");
}
