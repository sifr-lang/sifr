fn main() {
    println!("cargo:rustc-link-lib=zstd");
    println!("cargo:rerun-if-changed=build.rs");
}
