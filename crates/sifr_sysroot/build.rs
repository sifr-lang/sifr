fn main() -> Result<(), Box<dyn std::error::Error>> {
    sifr_identity::build::emit_local_token(&[])?;

    println!("cargo:rerun-if-env-changed=SIFR_RELEASE_VERSION");
    let version = std::env::var("SIFR_RELEASE_VERSION")
        .or_else(|_| std::env::var("CARGO_PKG_VERSION"))
        .unwrap_or_else(|_| "0.0.0".to_owned());
    println!("cargo:rustc-env=SIFR_SYSROOT_COMPILER_VERSION={version}");
    Ok(())
}
