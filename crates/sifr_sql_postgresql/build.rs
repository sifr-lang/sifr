use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed=SIFR_POSTGRESQL_MAJOR");
    println!("cargo:rerun-if-changed=component-sources.json");
    let major = env::var("SIFR_POSTGRESQL_MAJOR").unwrap_or_else(|_| "18".to_string());
    let manifest = read_manifest()?;
    let entry = manifest["sources"]
        .as_array()
        .and_then(|sources| {
            sources.iter().find(|source| {
                source["server_major"]
                    .as_u64()
                    .map(|value| value.to_string())
                    == Some(major.clone())
            })
        })
        .ok_or_else(|| io::Error::other(format!("unsupported SIFR_POSTGRESQL_MAJOR={major}")))?;
    let relative = entry["path"]
        .as_str()
        .ok_or_else(|| io::Error::other("PostgreSQL source manifest entry has no path"))?;
    let crate_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .ok_or_else(|| io::Error::other("Cargo did not provide CARGO_MANIFEST_DIR"))?,
    );
    let source = crate_dir.join(relative);
    if !source.join("pg_query.h").is_file() {
        return Err(io::Error::other(format!(
            "libpg_query source for PostgreSQL {major} is not initialized at {}",
            source.display()
        ))
        .into());
    }
    compile_libpg_query(&source)?;
    println!("cargo:rustc-env=SIFR_POSTGRESQL_MAJOR={major}");
    println!(
        "cargo:rustc-env=SIFR_LIBPG_QUERY_TAG={}",
        entry["tag"]
            .as_str()
            .ok_or_else(|| io::Error::other("PostgreSQL source has no tag"))?
    );
    println!(
        "cargo:rustc-env=SIFR_LIBPG_QUERY_COMMIT={}",
        entry["commit"]
            .as_str()
            .ok_or_else(|| io::Error::other("PostgreSQL source has no commit"))?
    );
    Ok(())
}

fn read_manifest() -> Result<Value, Box<dyn Error>> {
    let bytes = fs::read("component-sources.json")?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn compile_libpg_query(source: &Path) -> Result<(), Box<dyn Error>> {
    let mut build = cc::Build::new();
    build
        .include(source)
        .include(source.join("vendor"))
        .include(source.join("src/include"))
        .include(source.join("src/postgres/include"))
        .flag_if_supported("-fno-strict-aliasing")
        .flag_if_supported("-fwrapv")
        .warnings(false);
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        // Current macOS SDKs provide strchrnul. Older libpg_query tags only
        // detect that function on FreeBSD in their generated pg_config.h.
        build.define("HAVE_STRCHRNUL", "1");
    }
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        build.include(source.join("src/postgres/include/port/win32"));
    }
    add_c_files(&mut build, &source.join("src"))?;
    add_c_files(&mut build, &source.join("src/postgres"))?;
    for relative in [
        "vendor/protobuf-c/protobuf-c.c",
        "vendor/xxhash/xxhash.c",
        "protobuf/pg_query.pb-c.c",
    ] {
        build.file(source.join(relative));
    }
    build.compile("sifr_pg_query");
    Ok(())
}

fn add_c_files(build: &mut cc::Build, directory: &Path) -> Result<(), io::Error> {
    let mut entries = fs::read_dir(directory)?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    for entry in entries {
        if entry.extension().and_then(|value| value.to_str()) == Some("c")
            && !is_included_translation_unit(&entry)
        {
            println!("cargo:rerun-if-changed={}", entry.display());
            build.file(entry);
        }
    }
    Ok(())
}

fn is_included_translation_unit(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|value| value.to_str()),
        Some(
            "pg_query_enum_defs.c"
                | "pg_query_fingerprint_defs.c"
                | "pg_query_fingerprint_conds.c"
                | "pg_query_outfuncs_defs.c"
                | "pg_query_outfuncs_conds.c"
                | "pg_query_readfuncs_defs.c"
                | "pg_query_readfuncs_conds.c"
                | "pg_query_json_helper.c"
                | "guc-file.c"
                | "scan.c"
        )
    )
}
