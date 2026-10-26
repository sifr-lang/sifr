use super::*;
use crate::cli_model_and_entrypoint::InvocationWorkspace;
use serde_json::{Value, json};

fn fixture(root: &Path, name: &str, value: i32) -> PathBuf {
    let dir = root.join(name);
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("sifr.toml"), "[source]\nroot = \".\"\n").unwrap();
    fs::write(
        dir.join("helper.sifr"),
        format!("def value() -> int:\n    return {value}\n"),
    )
    .unwrap();
    let main = dir.join("main.sifr");
    fs::write(
        &main,
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .unwrap();
    main
}

fn scalar(compiler: &CompilerContext, file: &Path) -> String {
    match emit_entrypoint_with_context(compiler, file, &mut DiskSourceProvider::new()) {
        CompileResult::Success { rust_source } => rust_source,
        CompileResult::Errors { errors } => panic!("scalar emission failed: {errors:?}"),
    }
}

#[test]
fn sources_are_isolated_and_edits_and_errors_do_not_leak() {
    let owner = InvocationWorkspace::create("batch_isolation").unwrap();
    let root = owner.path();
    let output = root.join("output");
    fs::create_dir(&output).unwrap();
    let a = fixture(root, "a", 10);
    let b = fixture(root, "b", 20);
    let compiler = crate::compiler_context();
    let first = scalar(&compiler, &a);
    let second = scalar(&compiler, &b);
    assert_ne!(first, second);
    for (index, file, expected) in [(0, &a, &first), (1, &b, &second), (2, &a, &first)] {
        assert_eq!(
            emit_one(index, file, &output, &compiler).unwrap().exit_code,
            0
        );
        assert_eq!(
            fs::read_to_string(output.join(format!("{index}.rs"))).unwrap(),
            *expected
        );
    }
    fs::write(
        a.with_file_name("helper.sifr"),
        "def value() -> int:\n    return 11\n",
    )
    .unwrap();
    assert_eq!(emit_one(3, &a, &output, &compiler).unwrap().exit_code, 0);
    let edited = fs::read_to_string(output.join("3.rs")).unwrap();
    assert_ne!(edited, first);
    assert_eq!(edited, scalar(&compiler, &a));
    fs::write(&a, "def main():\n    x: int = \"bad\"\n    print(x)\n").unwrap();
    let error = emit_one(4, &a, &output, &compiler).unwrap();
    assert_eq!(error.exit_code, 1);
    let CompileResult::Errors { errors } =
        emit_entrypoint_with_context(&compiler, &a, &mut DiskSourceProvider::new())
    else {
        panic!("expected scalar diagnostics")
    };
    assert_eq!(
        error.stderr,
        render_diagnostic_output(&errors, DiagnosticFormat::Json).unwrap()
    );
    assert!(!output.join("4.rs").exists());
    assert_eq!(emit_one(5, &b, &output, &compiler).unwrap().exit_code, 0);
    assert_eq!(fs::read_to_string(output.join("5.rs")).unwrap(), second);
}

#[test]
fn complete_report_preserves_repeats_and_continues_after_missing_and_invalid_sources() {
    let owner = InvocationWorkspace::create("batch_report").unwrap();
    let root = owner.path();
    let good = root.join("good.sifr");
    let bad = root.join("bad.sifr");
    fs::write(&good, "def main():\n    print(30)\n").unwrap();
    fs::write(&bad, "def main():\n    print(undefined_name)\n").unwrap();
    let manifest = root.join("manifest.json");
    let files = vec![good.clone(), root.join("missing.sifr"), bad, good];
    fs::write(
        &manifest,
        serde_json::to_vec(&json!({"schema":1, "files": files})).unwrap(),
    )
    .unwrap();
    let output = root.join("output");
    assert_eq!(
        execute(&manifest, &output, &crate::compiler_context()).unwrap(),
        2
    );
    let report: Value =
        serde_json::from_slice(&fs::read(output.join("report.json")).unwrap()).unwrap();
    let entries = report["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 4);
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry["exit_code"].as_i64().unwrap())
            .collect::<Vec<_>>(),
        [0, 2, 1, 0]
    );
    for (index, file) in files.iter().enumerate() {
        assert_eq!(entries[index]["source"], file.to_string_lossy().as_ref());
        assert_eq!(entries[index]["index"], index);
    }
    assert_eq!(
        fs::read(output.join("0.rs")).unwrap(),
        fs::read(output.join("3.rs")).unwrap()
    );
    assert!(!output.join("1.rs").exists());
    assert!(!output.join("2.rs").exists());
    let before = fs::read(output.join("report.json")).unwrap();
    assert!(execute(&manifest, &output, &crate::compiler_context()).is_err());
    assert_eq!(before, fs::read(output.join("report.json")).unwrap());
}

#[test]
fn malformed_manifest_never_creates_an_output_owner() {
    let owner = InvocationWorkspace::create("batch_manifest").unwrap();
    let root = owner.path();
    for (index, input) in [
        r#"{"schema":2,"files":["/valid.sifr"]}"#,
        r#"{"schema":1,"files":[]}"#,
        r#"{"schema":1,"files":["relative.sifr"]}"#,
        r#"{"schema":1,"files":["/valid.sifr"],"ignored":true}"#,
        r#"{"schema":1,"files":[42]}"#,
        "{",
    ]
    .iter()
    .enumerate()
    {
        let manifest = root.join(format!("{index}.json"));
        let output = root.join(format!("output-{index}"));
        fs::write(&manifest, input).unwrap();
        assert!(execute(&manifest, &output, &crate::compiler_context()).is_err());
        assert!(!output.exists());
    }
}
