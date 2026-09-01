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

    let missing_lock = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .args(["sql", "test", "provision", "--profile", "app"])
        .current_dir(workspace.path())
        .output()
        .expect("reject missing host-tool lock");
    assert_eq!(missing_lock.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&missing_lock.stderr).contains("sifr tools lock"),
        "{}",
        String::from_utf8_lossy(&missing_lock.stderr)
    );

    let tool_lock = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .args(["tools", "lock"])
        .current_dir(workspace.path())
        .output()
        .expect("write host-tool lock");
    assert!(
        tool_lock.status.success(),
        "{}",
        String::from_utf8_lossy(&tool_lock.stderr)
    );
    let lock_check = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .args(["tools", "lock", "--check"])
        .current_dir(workspace.path())
        .output()
        .expect("verify host-tool lock");
    assert!(lock_check.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .args(["sql", "test", "provision", "--profile=app"])
        .current_dir(workspace.path())
        .output()
        .expect("run tool namespace");
    assert!(
        output.status.success(),
        "status={:?} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest = sifr_sql_contract::TestConnectionManifest::from_json(
        String::from_utf8_lossy(&output.stdout).trim(),
    )
    .expect("canonical connection manifest");
    assert_eq!(manifest.profile, "app");
    assert_eq!(manifest.cleanup.tool_namespace, "sql");
    assert_eq!(
        manifest.cleanup.command_arguments(),
        ["test", "cleanup", "--resource-id", "fixture-1"]
    );

    let probe = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .arg("probe")
        .current_dir(workspace.path())
        .env("SIFR_TEST_SECRET_TOKEN", "must-not-leak")
        .output()
        .expect("run confined probe");
    assert!(
        probe.status.success(),
        "status={:?} stdout={} stderr={}",
        probe.status,
        String::from_utf8_lossy(&probe.stdout),
        String::from_utf8_lossy(&probe.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "confined");

    let legal_near_name = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .arg("biuld")
        .current_dir(workspace.path())
        .output()
        .expect("run declared namespace near built-in spelling");
    assert!(legal_near_name.status.success());
    assert_eq!(
        String::from_utf8_lossy(&legal_near_name.stdout).trim(),
        "confined"
    );

    let bounded = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .args(["probe", "flood"])
        .current_dir(workspace.path())
        .output()
        .expect("bound combined tool output");
    assert!(!bounded.status.success());
    assert!(String::from_utf8_lossy(&bounded.stderr).contains("10 MiB limit"));

    let unknown = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .arg("unknown-tool")
        .current_dir(workspace.path())
        .output()
        .expect("run unknown namespace");
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown tool namespace"));

    let typo = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .arg("chekc")
        .current_dir(workspace.path().join("provider"))
        .output()
        .expect("diagnose built-in typo");
    assert_eq!(typo.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&typo.stderr).contains("did you mean 'check'"));

    std::fs::write(
        workspace.path().join("provider/src/drift.rs"),
        "pub const DRIFT: u8 = 1;\n",
    )
    .expect("mutate tool source");
    let drift = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .arg("probe")
        .current_dir(workspace.path())
        .output()
        .expect("detect persisted tool drift");
    assert_eq!(drift.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&drift.stderr).contains("does not match"));
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
        "[tools.sql]\npackage = \"provider-tools\"\nentrypoint = \"sql-tool\"\ncapabilities = [\"credentials\", \"network\", \"project-write\"]\n\n[tools.probe]\npackage = \"provider-tools\"\nentrypoint = \"probe-tool\"\ncapabilities = []\n\n[tools.biuld]\npackage = \"provider-tools\"\nentrypoint = \"probe-tool\"\ncapabilities = []\n",
    )
    .expect("tools Sifr manifest");
    std::fs::write(
        root.join("provider/Cargo.toml"),
        "[package]\nname = \"provider-tools\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[[bin]]\nname = \"sql-tool\"\npath = \"src/main.rs\"\n\n[[bin]]\nname = \"probe-tool\"\npath = \"src/probe.rs\"\n",
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
    let workspace_secret = root.join("workspace-secret.txt");
    std::fs::write(&workspace_secret, "secret\n").expect("workspace secret");
    std::fs::write(
        root.join("provider/src/probe.rs"),
        format!(
            r#"fn main() {{
    let mode = std::env::args().nth(1);
    if mode.as_deref() == Some("child") {{ return; }}
    if mode.as_deref() == Some("flood") {{
        use std::io::Write as _;
        let bytes = vec![b'x'; 6 * 1024 * 1024];
        std::io::stdout().write_all(&bytes).unwrap();
        std::io::stderr().write_all(&bytes).unwrap();
        return;
    }}
    if std::env::var_os("SIFR_TEST_SECRET_TOKEN").is_some() {{ std::process::exit(10); }}
    if std::fs::read_to_string({workspace_secret:?}).is_ok() {{ std::process::exit(11); }}
    if std::process::Command::new("/usr/bin/true").status().is_ok() {{ std::process::exit(12); }}
    if std::net::TcpListener::bind("127.0.0.1:0").is_ok() {{ std::process::exit(13); }}
    if std::env::current_exe().ok().and_then(|path| std::process::Command::new(path).arg("child").status().ok()).is_some() {{ std::process::exit(14); }}
    println!("confined");
}}
"#,
            workspace_secret = workspace_secret,
        ),
    )
    .expect("probe source");
}
