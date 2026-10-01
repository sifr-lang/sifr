use std::process::Command;
fn main() {
    let output = Command::new(std::env::var_os("RUSTC").expect("Cargo provides RUSTC"))
        .args(["--print", "sysroot"])
        .output()
        .expect("query selected rustc");
    assert!(output.status.success(), "selected sysroot query failed");
    let sysroot = String::from_utf8(output.stdout).expect("UTF-8 sysroot");
    let lib = format!("{}/lib", sysroot.trim());
    println!("cargo:rustc-link-search=native={lib}");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{lib}");
    println!("cargo:rerun-if-env-changed=RUSTC");
}
