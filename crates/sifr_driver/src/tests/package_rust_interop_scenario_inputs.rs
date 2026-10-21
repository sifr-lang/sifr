//! Assemble canonical inputs and static overlays before retaining a fixture tree.
use super::*;

pub(crate) struct ScenarioInputs {
    fixture: String,
    scenario: String,
    test: String,
    files: Inputs,
}

impl ScenarioInputs {
    pub(crate) fn new(fixture: &str, scenario: &str, test: &str) -> Self {
        Self {
            fixture: fixture.to_owned(),
            scenario: scenario.to_owned(),
            test: test.to_owned(),
            files: read_inputs(&fixture_scenario_root(fixture, scenario))
                .expect("current bounded canonical fixture"),
        }
    }

    pub(crate) fn text(&self, path: &str) -> &str {
        std::str::from_utf8(&self.files[Path::new(path)]).expect("UTF-8 fixture input")
    }

    pub(crate) fn write(&mut self, path: &str, bytes: impl AsRef<[u8]>) {
        self.files.insert(path.into(), bytes.as_ref().to_vec());
    }

    pub(crate) fn with_source(mut self, source: &str) -> Self {
        self.write("src/main.sifr", source);
        self
    }

    pub(crate) fn with_runtime(mut self) -> Self {
        let manifest = rebased_runtime_manifest(self.text("Cargo.toml"));
        self.write("Cargo.toml", manifest);
        self
    }

    pub(crate) fn finish(self) -> ReusableScenario {
        let owner = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("owned compiler checkout");
        ReusableScenario::acquire(
            &crate::cache_storage::root().join("test-fixtures"),
            &owner,
            &self.fixture,
            &self.scenario,
            &self.test,
            self.files,
        )
        .expect("bounded native fixture ownership")
    }
}

#[test]
fn builder_applies_final_overlays_without_rewriting_unchanged_inputs() {
    let canonical = fixture_scenario_root(FIXTURE, SCENARIO);
    let before = read_inputs(&canonical).unwrap();
    let make = || {
        let mut inputs = ScenarioInputs::new(FIXTURE, SCENARIO, TEST)
            .with_runtime()
            .with_source("final source\n");
        inputs.write("src/bridges/control.rs", "final bridge\n");
        inputs.write("sifr.toml", "final trust\n");
        inputs.write("Cargo.lock", "final lock\n");
        inputs
    };
    let temp = super::tests::private_root();
    let base = temp.path().join("cache");
    let final_inputs = make();
    let first = ReusableScenario::acquire(
        &base,
        temp.path(),
        &final_inputs.fixture,
        &final_inputs.scenario,
        &final_inputs.test,
        final_inputs.files.clone(),
    )
    .unwrap();
    let root = first.root.clone();
    let stamp = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(8765);
    for path in final_inputs.files.keys() {
        File::options()
            .write(true)
            .open(root.join(path))
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(stamp))
            .unwrap();
    }
    drop(first);
    let repeat = make();
    let second = ReusableScenario::acquire(
        &base,
        temp.path(),
        &repeat.fixture,
        &repeat.scenario,
        &repeat.test,
        repeat.files.clone(),
    )
    .unwrap();
    assert_eq!(second.root, root);
    assert_eq!(read_inputs(&root).unwrap(), final_inputs.files);
    for path in repeat.files.keys() {
        assert_eq!(
            fs::metadata(root.join(path)).unwrap().modified().unwrap(),
            stamp
        );
    }
    assert_eq!(read_inputs(&canonical).unwrap(), before);
}
