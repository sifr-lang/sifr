fn main() {
    println!("cargo:rerun-if-env-changed=SIFR_RELEASE_VERSION");

    let version = std::env::var("SIFR_RELEASE_VERSION")
        .or_else(|_| std::env::var("CARGO_PKG_VERSION"))
        .expect("CARGO_PKG_VERSION should be set by Cargo");

    println!("cargo:rustc-env=SIFR_BUILD_VERSION={version}");
}
