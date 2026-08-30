use std::process::Command;

#[test]
fn direct_sql_namespace_runs_locked_host_tool_and_validates_manifest() {
    let workspace = tempfile::tempdir().expect("workspace");
    write_fixture(workspace.path());
    let lock = Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(workspace.path())
        .output()
        .expect("generate lockfile");
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .args(["sql", "test", "provision", "--profile", "app"])
        .current_dir(workspace.path())
        .output()
        .expect("run tool namespace");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest = sifr_sql_contract::TestConnectionManifest::from_json(
        String::from_utf8_lossy(&output.stdout).trim(),
    )
    .expect("canonical connection manifest");
    assert_eq!(manifest.profile, "app");
    assert_eq!(manifest.cleanup.tool_namespace, "sql");

    let unknown = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .arg("unknown-tool")
        .current_dir(workspace.path())
        .output()
        .expect("run unknown namespace");
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown tool namespace"));
}

fn write_fixture(root: &std::path::Path) {
    std::fs::create_dir_all(root.join("tools/src")).expect("tools source");
    std::fs::create_dir_all(root.join("provider/src")).expect("provider source");
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"tools\", \"provider\"]\nresolver = \"2\"\n\n[workspace.metadata.sifr]\ntools-package = \"project-tools\"\n",
    )
    .expect("workspace manifest");
    std::fs::write(
        root.join("tools/Cargo.toml"),
        "[package]\nname = \"project-tools\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nprovider-tools = { path = \"../provider\" }\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n",
    )
    .expect("tools Cargo manifest");
    std::fs::write(root.join("tools/src/lib.rs"), "pub fn marker() {}\n").expect("tools marker");
    std::fs::write(
        root.join("tools/sifr.toml"),
        "[tools.sql]\npackage = \"provider-tools\"\nentrypoint = \"sql-tool\"\ncapabilities = [\"credentials\", \"network\", \"project-write\"]\n",
    )
    .expect("tools Sifr manifest");
    std::fs::write(
        root.join("provider/Cargo.toml"),
        "[package]\nname = \"provider-tools\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[[bin]]\nname = \"sql-tool\"\npath = \"src/main.rs\"\n",
    )
    .expect("provider manifest");
    std::fs::write(
        root.join("provider/src/lib.rs"),
        "pub const TOOL_PACKAGE_MARKER: &str = \"sql\";\n",
    )
    .expect("provider library source");
    std::fs::write(
        root.join("provider/src/main.rs"),
        r##"fn main() {
    let capabilities = std::env::var("SIFR_TOOL_CAPABILITIES").unwrap_or_default();
    if capabilities != "credentials,network,project-write" {
        std::process::exit(9);
    }
    println!(r#"{{"schema-version":1,"provider":"postgresql","profile":"app","schema-fingerprint":"sha256:test","connection":{{"transport":"tcp","host":"127.0.0.1","port":5432,"database":"test","user":"tester","credential":{{"source":"environment","variable":"SIFR_TEST_PASSWORD"}},"tls":false}},"cleanup":{{"tool-namespace":"sql","resource-id":"fixture-1"}}}}"#);
}
"##,
    )
    .expect("provider source");
}
