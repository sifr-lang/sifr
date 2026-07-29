use std::error::Error;
use std::io;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let bindings = bindgen_upstream::Builder::default()
        .header_contents(
            "sifr_bindgen_probe.h",
            "unsigned int sifr_bindgen_probe(void);",
        )
        .allowlist_function("sifr_bindgen_probe")
        .generate_comments(false)
        .layout_tests(false)
        .generate()
        .map_err(|error| io::Error::other(format!("bindgen probe failed: {error}")))?;
    let generated = bindings.to_string();
    if !generated.contains("sifr_bindgen_probe") {
        return Err(io::Error::other("bindgen output omitted the probe function").into());
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    std::fs::write(out_dir.join("sifr-bindgen-bindings.rs"), generated)?;
    std::fs::write(
        out_dir.join("sifr-bindgen-evidence.txt"),
        "bindgen=0.72.1;function=sifr_bindgen_probe",
    )?;
    Ok(())
}
