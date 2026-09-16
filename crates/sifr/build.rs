fn main() -> Result<(), Box<dyn std::error::Error>> {
    sifr_identity::build::emit_local_token(&["third_party/ruff"])?;
    sifr_identity::build::emit_product_identity()?;

    println!("cargo:rerun-if-env-changed=SIFR_RELEASE_VERSION");

    let version = std::env::var("SIFR_RELEASE_VERSION")
        .or_else(|_| std::env::var("CARGO_PKG_VERSION"))
        .unwrap_or_else(|_| "0.0.0".to_string());

    println!("cargo:rustc-env=SIFR_BUILD_VERSION={version}");
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown-target".to_string());
    println!("cargo:rustc-env=SIFR_BUILD_TARGET={target}");
    Ok(())
}
