use super::*;
use sifr_frontend::DiskSourceProvider;
use sifr_stdlib_manifest::StdlibFeature;

fn mktemp_dir(name: &str) -> PathBuf {
    let unique = format!(
        "sifr_rooted_entrypoint_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_single_file_entrypoint_plan_generates_main_only_project() {
    let plan = RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::SingleFile {
        source: "def main():\n    print(\"ok\")\n",
        display_path: "main",
        lowering_options: LoweringOptions::default(),
    })
    .expect("single-file entrypoint should compile");

    let generated_project = plan
        .into_generated_binary_project()
        .expect("single-file generated project should succeed");

    assert!(generated_project.support_modules.is_empty());
    assert!(generated_project.main_rs.contains("fn main"));
    assert!(generated_project.used_stdlib_modules.is_empty());
    assert!(generated_project.required_features.is_empty());
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_project_entrypoint_plan_generates_support_modules() {
    let dir = mktemp_dir("project_positive");
    let main_file = dir.join("main.sifr");
    std::fs::write(
        &main_file,
        "from helper import message\n\ndef main():\n    print(message())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def message() -> str:\n    return \"ok\"\n",
    )
    .expect("helper should be written");

    let plan = RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::Project {
        main_file: &main_file,
        provider: &mut DiskSourceProvider::new(),
    })
    .expect("project entrypoint should compile");
    let generated_project = plan
        .into_generated_binary_project()
        .expect("project generated project should succeed");

    assert_eq!(
        generated_project
            .support_modules
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        vec!["helper".to_string()]
    );
    assert!(generated_project.main_rs.starts_with("mod helper;"));
    assert!(generated_project.main_rs.contains("fn main"));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_project_entrypoint_plan_reports_reachable_frontend_errors() {
    let dir = mktemp_dir("project_negative");
    let main_file = dir.join("main.sifr");
    std::fs::write(
        &main_file,
        "from helper import broken\n\ndef main():\n    print(broken())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def broken() -> int:\n    return \"bad\"\n",
    )
    .expect("helper should be written");

    let errors = match RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::Project {
        main_file: &main_file,
        provider: &mut DiskSourceProvider::new(),
    }) {
        Ok(_) => panic!("reachable project type error should fail plan construction"),
        Err(errors) => errors,
    };

    assert!(errors
        .iter()
        .any(|error| error.message.contains("[helper] return type mismatch")));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_project_entrypoint_plan_aggregates_reachable_dependency_metadata() {
    let dir = mktemp_dir("project_metadata_positive");
    let main_file = dir.join("main.sifr");
    std::fs::write(
        &main_file,
        "from helper import helper\n\ndef main():\n    print(helper())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "from sifr.statistics import mean\n\n\
def helper() -> int:\n    return 1\n",
    )
    .expect("helper should be written");

    let plan = RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::Project {
        main_file: &main_file,
        provider: &mut DiskSourceProvider::new(),
    })
    .expect("project entrypoint should compile");
    let generated_project = plan
        .into_generated_binary_project()
        .expect("project metadata aggregation should succeed");

    assert!(generated_project
        .used_stdlib_modules
        .contains("sifr.statistics"));
    assert!(generated_project.used_stdlib_modules.contains("sifr.math"));
    assert!(generated_project
        .required_features
        .contains(&StdlibFeature::NumBigint));
    assert!(generated_project
        .required_features
        .contains(&StdlibFeature::NumTraits));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_project_entrypoint_plan_ignores_unreachable_dependency_metadata() {
    let dir = mktemp_dir("project_metadata_negative");
    let main_file = dir.join("main.sifr");
    std::fs::write(
        &main_file,
        "from helper import helper\n\ndef main():\n    print(helper())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def helper() -> int:\n    return 1\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("unused_dependency.sifr"),
        "from sifr.json import dumps\n\ndef unused() -> str:\n    return dumps({\"x\": 1})\n",
    )
    .expect("unused dependency should be written");

    let plan = RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::Project {
        main_file: &main_file,
        provider: &mut DiskSourceProvider::new(),
    })
    .expect("project entrypoint should compile");
    let generated_project = plan
        .into_generated_binary_project()
        .expect("project metadata aggregation should succeed");

    assert!(!generated_project.used_stdlib_modules.contains("sifr.json"));
    assert!(!generated_project
        .required_features
        .contains(&StdlibFeature::SerdeJson));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_cached_project_binary_reuses_workspace_for_unchanged_input() {
    let dir = mktemp_dir("cached_project_reuse");
    let main_file = dir.join("main.sifr");
    std::fs::write(
        &main_file,
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def value() -> int:\n    return 11\n",
    )
    .expect("helper should be written");

    let first = build_cached_project_binary(&main_file, &mut DiskSourceProvider::new())
        .expect("first cached build should succeed");
    assert!(first.binary_path().exists());
    assert!(!first.build_report().cache_hit());

    let first_output = std::process::Command::new(first.binary_path())
        .output()
        .expect("first cached binary should run");
    assert!(first_output.status.success());
    assert_eq!(String::from_utf8_lossy(&first_output.stdout).trim(), "11");

    let second = build_cached_project_binary(&main_file, &mut DiskSourceProvider::new())
        .expect("second cached build should succeed");
    assert!(second.build_report().cache_hit());
    assert_eq!(first.binary_path(), second.binary_path());

    let second_output = std::process::Command::new(second.binary_path())
        .output()
        .expect("second cached binary should run");
    assert!(second_output.status.success());
    assert_eq!(String::from_utf8_lossy(&second_output.stdout).trim(), "11");

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_cached_project_binary_invalidates_when_sources_change() {
    let dir = mktemp_dir("cached_project_invalidation");
    let main_file = dir.join("main.sifr");
    std::fs::write(
        &main_file,
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main should be written");
    let helper = dir.join("helper.sifr");
    std::fs::write(&helper, "def value() -> int:\n    return 21\n")
        .expect("helper should be written");

    let first = build_cached_project_binary(&main_file, &mut DiskSourceProvider::new())
        .expect("first cached build should succeed");
    assert!(!first.build_report().cache_hit());

    std::fs::write(&helper, "def value() -> int:\n    return 22\n")
        .expect("helper should be updated");
    let second = build_cached_project_binary(&main_file, &mut DiskSourceProvider::new())
        .expect("second cached build should succeed");
    assert!(!second.build_report().cache_hit());
    assert_ne!(first.binary_path(), second.binary_path());

    let output = std::process::Command::new(second.binary_path())
        .output()
        .expect("updated cached binary should run");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "22");

    let _ = std::fs::remove_dir_all(dir);
}
