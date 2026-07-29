use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    std::fs::write(
        out_dir.join("sifr-cxx-evidence.txt"),
        "cxx=1.0.198;bridge=sifr_cxx_probe",
    )?;
    Ok(())
}
