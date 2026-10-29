use super::*;
use crate::cli_model_and_entrypoint::InvocationWorkspace;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;

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

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, dir: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(root, &path, result);
            } else {
                result.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    collect(root, root, &mut result);
    result
}

fn batch(
    root: &Path,
    name: &str,
    sources: &[PathBuf],
    compiler: &CompilerContext,
) -> (i32, Value, PathBuf) {
    let manifest = root.join(format!("{name}.json"));
    fs::write(
        &manifest,
        serde_json::to_vec(&json!({"schema":1,"files":sources})).unwrap(),
    )
    .unwrap();
    let output = root.join(name);
    let status = execute(&manifest, &output, compiler).unwrap();
    let report = serde_json::from_slice(&fs::read(output.join("report.json")).unwrap()).unwrap();
    (status, report, output)
}

#[test]
fn materialization_matches_scalar_and_refreshes_each_project_after_edits() {
    let owner = InvocationWorkspace::create("materialize_batch_isolation").unwrap();
    let root = owner.path();
    let a = fixture(root, "a", 10);
    let b = fixture(root, "b", 20);
    let compiler = crate::compiler_context();
    let sources = [a.clone(), b.clone(), a.clone()];
    let (status, report, output) = batch(root, "first", &sources, &compiler);
    assert_eq!(status, 0);
    for (index, source) in sources.iter().enumerate() {
        let scalar = root.join(format!("scalar-{index}"));
        assert_eq!(
            materialize_with_context(
                &crate::compiler_context(),
                source,
                &scalar,
                sifr_package::CargoLockMode::Normal,
                true,
                DiagnosticFormat::Human
            ),
            0
        );
        let generated = output.join(index.to_string()).join("sifr_output");
        assert_eq!(files(&generated), files(&scalar.join("sifr_output")));
        assert_eq!(
            report["entries"][index]["source"],
            source.to_string_lossy().as_ref()
        );
        assert_eq!(
            report["entries"][index]["project"],
            generated.to_string_lossy().as_ref()
        );
    }
    let first = files(&output.join("0/sifr_output"));
    assert_eq!(first, files(&output.join("2/sifr_output")));
    assert_ne!(first, files(&output.join("1/sifr_output")));
    fs::write(
        a.with_file_name("helper.sifr"),
        "def value() -> int:\n    return 11\n",
    )
    .unwrap();
    let (status, _, edited) = batch(root, "edited", &[a], &compiler);
    assert_eq!(status, 0);
    assert_ne!(first, files(&edited.join("0/sifr_output")));
    assert_eq!(first, files(&output.join("0/sifr_output")));
}

#[test]
fn materialization_reports_every_error_and_independent_success_without_reusing_output() {
    let owner = InvocationWorkspace::create("materialize_batch_errors").unwrap();
    let root = owner.path();
    let good = fixture(root, "good", 30);
    let bad = fixture(root, "bad", 40);
    fs::write(&bad, "def main():\n    print(undefined_name)\n").unwrap();
    let sources = [root.join("missing.sifr"), bad, good.clone(), good];
    let (status, report, output) = batch(root, "result", &sources, &crate::compiler_context());
    assert_ne!(status, 0);
    let entries = report["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 4);
    assert_ne!(entries[0]["exit_code"], 0);
    assert_ne!(entries[1]["exit_code"], 0);
    assert_eq!(entries[2]["exit_code"], 0);
    assert_eq!(entries[3]["exit_code"], 0);
    assert!(!output.join("0/sifr_output/Cargo.toml").exists());
    assert!(!output.join("1/sifr_output/Cargo.toml").exists());
    assert_eq!(
        files(&output.join("2/sifr_output")),
        files(&output.join("3/sifr_output"))
    );
    let before = fs::read(output.join("report.json")).unwrap();
    assert!(
        execute(
            &root.join("result.json"),
            &output,
            &crate::compiler_context()
        )
        .is_err()
    );
    assert_eq!(before, fs::read(output.join("report.json")).unwrap());
}
