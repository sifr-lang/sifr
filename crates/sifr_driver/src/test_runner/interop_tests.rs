use super::artifacts::try_generate_test_runner_cargo_plan;
use super::execution::execute_test_runner_project;
use super::orchestrator::build_test_runner_project;
use crate::project::discover_test_root_modules;
use sifr_frontend::DiskSourceProvider;
use std::path::{Path, PathBuf};

struct TestSource(PathBuf);

impl TestSource {
    fn new(label: &str, source: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "sifr-test-interop-{label}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create test source");
        std::fs::write(dir.join("test_contract.sifr"), source).expect("write test source");
        Self(dir)
    }
}

impl Drop for TestSource {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn stdlib_interop_test_project_materializes_selected_contracts() {
    let source = TestSource::new(
        "native",
        "from helper import leap\nfrom sifr.math import isfinite\nfrom sifr.process import Command, ProcessError, Status, run\n\ndef test_contract():\n    assert leap(2024)\n    assert not leap(2023)\n    assert isfinite(1.5)\n    try:\n        status: Status = run(Command(\"\"))\n        assert False\n    except ProcessError as error:\n        assert len(error.message) > 0\n",
    );
    std::fs::write(
        source.0.join("helper.sifr"),
        "from sifr.calendar import isleap\n\ndef leap(year: int) -> bool:\n    return isleap(year)\n",
    )
    .expect("write support source");
    let mut provider = DiskSourceProvider::new();
    let roots = discover_test_root_modules(&source.0, &mut provider);
    let project = build_test_runner_project(&source.0, &roots, &mut provider)
        .expect("resolve complete test-project runtime demand");
    assert!(project.interop.stdlib_demand.declarations.is_empty());
    let modules = project
        .interop
        .rust
        .resolved_targets
        .iter()
        .filter_map(|target| target.module_name.as_deref())
        .collect::<Vec<_>>();
    assert!(modules.contains(&"_sifr.calendar"), "{modules:?}");
    assert!(modules.contains(&"_sifr.math"), "{modules:?}");
    assert!(modules.contains(&"_sifr.process"), "{modules:?}");
    assert!(!modules.contains(&"_sifr.python"), "{modules:?}");
    let plan = try_generate_test_runner_cargo_plan(
        &project.all_stdlib_modules,
        &project.all_required_features,
        &project.interop,
    )
    .expect("materialize resolved Cargo demand");
    assert!(plan.cargo_toml.contains("sifr_stdlib"));
    assert!(project.support_rust_files.contains_key("helper"));
    assert!(
        project
            .bridge_rust_files
            .contains_key(Path::new("sifr_generated_bridge/mod.rs"))
    );
    assert!(
        project
            .bridge_rust_files
            .contains_key(Path::new("sifr_generated_bridge/sifr_generated_process.rs"))
    );
    let outcome = execute_test_runner_project(&project)
        .expect("execute selected contracts through the actual Cargo consumer");
    assert!(outcome.success);
    for (path, expected) in &project.bridge_rust_files {
        let actual =
            std::fs::read_to_string(outcome.cache_report.workspace_root().join("src").join(path))
                .expect("canonical bridge file materialized");
        assert_eq!(&actual, expected);
    }
}

#[test]
fn stdlib_interop_test_project_empty_demand_stays_empty() {
    let source = TestSource::new("empty", "def test_contract():\n    assert True\n");
    let mut provider = DiskSourceProvider::new();
    let roots = discover_test_root_modules(&source.0, &mut provider);
    let project = build_test_runner_project(&source.0, &roots, &mut provider)
        .expect("generate test project without interop");
    assert!(project.interop.rust.declarations.is_empty());
    assert!(project.interop.rust.resolved_targets.is_empty());
    assert!(project.interop.rust.cargo_inputs.is_none());
    assert!(project.interop.stdlib_demand.declarations.is_empty());
    assert!(project.bridge_rust_files.is_empty());
    let plan = try_generate_test_runner_cargo_plan(
        &project.all_stdlib_modules,
        &project.all_required_features,
        &project.interop,
    )
    .expect("empty Cargo plan");
    assert!(!plan.cargo_toml.contains("sifr_stdlib"));
}
