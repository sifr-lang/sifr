//! Produce native images only from maintained Wasm during trusted construction.
#![allow(clippy::print_stdout)]
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, fs, path::PathBuf, time::Instant};
#[path = "src/policy.rs"]
mod policy;

const MAX_BUNDLE_BYTES: usize = 128 * 1024 * 1024;
const PROVIDERS: &[&str] = &["sifr_sql_mysql", "sifr_sql_postgresql", "sifr_sql_sqlite"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let root = package
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or("component engine has no workspace root")?;
    let mut extra_inputs = Vec::new();
    let mut unique = BTreeMap::new();
    let mut input_count = 0;
    for provider in PROVIDERS {
        let directory = root.join("crates").join(provider);
        extra_inputs.push(format!("crates/{provider}/components"));
        extra_inputs.push(format!("crates/{provider}/component-artifacts.json"));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("component-artifacts.json"))?)?;
        let artifacts = manifest["artifacts"]
            .as_array()
            .ok_or("missing artifact inventory")?;
        for artifact in artifacts {
            let relative = artifact["path"].as_str().ok_or("missing artifact path")?;
            let expected = artifact["sha256"].as_str().ok_or("missing artifact hash")?;
            let relative = std::path::Path::new(relative);
            if !relative.starts_with("components")
                || relative
                    .components()
                    .any(|part| !matches!(part, std::path::Component::Normal(_)))
                || relative.extension().and_then(|value| value.to_str()) != Some("wasm")
            {
                return Err(
                    "component artifact must stay inside its owned components directory".into(),
                );
            }
            let path = directory.join(relative);
            let bytes = fs::read(&path)?;
            let actual = sha256(&bytes)?;
            if actual != expected {
                return Err(format!("component source hash mismatch: {}", path.display()).into());
            }
            unique.entry(actual).or_insert(bytes);
            input_count += 1;
        }
    }
    let inputs = extra_inputs.iter().map(String::as_str).collect::<Vec<_>>();
    sifr_identity::build::emit_local_token(&inputs)?;
    let out = PathBuf::from(env::var("OUT_DIR")?);
    let image_dir = out.join("images");
    fs::create_dir_all(&image_dir)?;
    let start = Instant::now();
    let native = env::var("HOST")? == env::var("TARGET")?;
    let mut records = Vec::new();
    let mut hashes = Vec::new();
    let mut code = String::from(
        "pub(crate) fn image(hash: &str) -> Option<&'static [u8]> {\n    match hash {\n",
    );
    let mut key = 0;
    let mut total_bytes = 0usize;
    if native {
        let engine = policy::engine(policy::DEFAULT_STACK)?;
        key = policy::key(&engine);
        for (index, (hash, wasm)) in unique.into_iter().enumerate() {
            let started = Instant::now();
            let bytes = engine.precompile_component(&wasm)?;
            total_bytes = total_bytes
                .checked_add(bytes.len())
                .ok_or("bundle size overflow")?;
            if total_bytes > MAX_BUNDLE_BYTES {
                return Err("embedded component images exceed the 128MiB bundle budget".into());
            }
            let file = image_dir.join(format!("component-{index:03}.cwasm"));
            fs::write(&file, &bytes)?;
            code.push_str(&format!(
                "        {hash:?} => Some(include_bytes!({file:?})),\n"
            ));
            hashes.push(hash.clone());
            records.push(serde_json::json!({
                "source_sha256": hash,
                "native_sha256": sha256(&bytes)?,
                "native_bytes": bytes.len(),
                "seconds": started.elapsed().as_secs_f64()
            }));
        }
    }
    // Only this producer owns this exact indexed namespace. Never delete unknown
    // files, and never accumulate one native file per historical source hash.
    for entry in fs::read_dir(&image_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("non-UTF8 owned image name")?;
        let index: usize = name
            .strip_prefix("component-")
            .and_then(|name| name.strip_suffix(".cwasm"))
            .ok_or("unexpected owned image file")?
            .parse()?;
        if name != format!("component-{index:03}.cwasm") {
            return Err("unexpected owned image name".into());
        }
        if index >= records.len() {
            fs::remove_file(entry.path())?;
        }
    }
    code.push_str("        _ => None,\n    }\n}\n");
    code.push_str(&format!("pub(crate) const ENGINE_KEY: u64 = {key};\n"));
    code.push_str(&format!(
        "#[cfg(test)]\npub(crate) const NATIVE_BUNDLE: bool = {native};\n"
    ));
    code.push_str(&format!(
        "#[cfg(test)]\npub(crate) const SOURCE_HASHES: &[&str] = &{hashes:?};\n"
    ));
    fs::write(out.join("bundle.rs"), code)?;
    fs::write(
        out.join("generation.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "host": env::var("HOST")?,
            "target": env::var("TARGET")?,
            "native_build": native,
            "engine_key": key,
            "stack": policy::DEFAULT_STACK,
            "input_count": input_count,
            "image_bytes": total_bytes,
            "elapsed_seconds": start.elapsed().as_secs_f64(),
            "images": records
        }))?,
    )?;
    Ok(())
}

fn sha256(bytes: &[u8]) -> Result<String, std::fmt::Error> {
    use std::fmt::Write as _;
    let mut result = String::with_capacity(64);
    for byte in Sha256::digest(bytes).iter() {
        write!(&mut result, "{byte:02x}")?;
    }
    Ok(result)
}
