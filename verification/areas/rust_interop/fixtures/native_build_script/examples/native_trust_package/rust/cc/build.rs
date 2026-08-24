use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=native/probe.c");
    cc_upstream::Build::new()
        .file("native/probe.c")
        .warnings(false)
        .compile("sifr_cc_probe");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    std::fs::write(
        out_dir.join("sifr-cc-evidence.txt"),
        "cc=1.4.4;compiled=sifr_cc_probe",
    )?;
    Ok(())
}
