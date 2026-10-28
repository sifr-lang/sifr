//! Retained native fixture trees, reconciled from final inputs under explicit ownership.
use super::*;
use crate::build::native_child_leases::NativeChildLeases;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{self, Read};

const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_FILES: usize = 4096;
const FIXTURE: &str = "advanced_data_runtime_matrix";
const SCENARIO: &str = "advanced_data_runtime";
const TEST: &str = "rust_interop_advanced_data_roundtrips";
type Inputs = BTreeMap<PathBuf, Vec<u8>>;

#[path = "package_rust_interop_scenario_inputs.rs"]
mod input_builder;
pub(crate) use input_builder::ScenarioInputs;

pub(crate) struct ReusableScenario {
    root: PathBuf,
    #[cfg(unix)]
    _activity: crate::cache_storage::LeaseActivity,
    lease: File,
}

impl std::ops::Deref for ReusableScenario {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.root
    }
}

impl ReusableScenario {
    pub(crate) fn advanced_data(source: &str) -> Self {
        ScenarioInputs::new(FIXTURE, SCENARIO, TEST)
            .with_runtime()
            .with_source(source)
            .finish()
    }

    fn acquire(
        base: &Path,
        owner: &Path,
        fixture: &str,
        scenario: &str,
        test: &str,
        inputs: Inputs,
    ) -> io::Result<Self> {
        validate_inputs(&inputs)?;
        // Keep the pre-existing authority path; revisions never allocate a new tree.
        let mut identity = sifr_identity::IdentityEncoder::new("native-test-fixture-v1");
        for (name, value) in [
            ("fixture", fixture.as_bytes()),
            ("scenario", scenario.as_bytes()),
            ("test", test.as_bytes()),
            ("owner", owner.as_os_str().as_encoded_bytes()),
        ] {
            identity.field(name, value);
        }
        crate::cache_storage::directory(base)?;
        let key = identity.finish();
        let lease = crate::build::native_child_leases::open(base, &key)?;
        #[cfg(unix)]
        crate::cache_storage::lock_bounded(
            &lease,
            &base.join(".locks").join(&key),
            false,
            crate::cache_storage::LEASE_WAIT,
        )?;
        #[cfg(windows)]
        lease.lock()?;
        #[cfg(unix)]
        let activity = crate::cache_storage::LeaseActivity::start(&lease)?;
        let root = base.join(key);
        reconcile(&root, &inputs)?;
        Ok(Self {
            root,
            #[cfg(unix)]
            _activity: activity,
            lease,
        })
    }

    pub(crate) fn entrypoint(&self, name: &str) -> PackageEntrypoint {
        package_entrypoint_with_leases(&self.root, name, &self.leases())
    }

    fn leases(&self) -> NativeChildLeases {
        NativeChildLeases::default()
            .with(&self.lease)
            .expect("clone fixture ownership")
    }

    pub(crate) fn check(
        &self,
        compiler: &crate::CompilerContext,
        entrypoint: &PackageEntrypoint,
    ) -> Vec<crate::diagnostics::RenderedDiagnostic> {
        crate::build::check_package_project_with_leases(
            compiler,
            entrypoint,
            &mut sifr_frontend::DiskSourceProvider::new(),
            self.leases(),
        )
    }

    pub(crate) fn try_build(
        &self,
        compiler: &crate::CompilerContext,
        entrypoint: &PackageEntrypoint,
    ) -> Result<crate::CachedBinaryArtifact, Vec<crate::diagnostics::RenderedDiagnostic>> {
        crate::build::build_cached_package_project_binary(
            compiler,
            entrypoint,
            &mut sifr_frontend::DiskSourceProvider::new(),
            self.leases(),
        )
    }

    pub(crate) fn build(&self, entrypoint: &PackageEntrypoint) -> crate::CachedBinaryArtifact {
        self.try_build(&crate::CompilerContext::for_test(), entrypoint)
            .expect("owned Rust interop fixture should build")
    }

    pub(crate) fn built_package_output(
        &self,
        entrypoint: &PackageEntrypoint,
    ) -> std::process::Output {
        let artifact = self.build(entrypoint);
        let output = artifact
            .command()
            .expect("owned runtime command")
            .output()
            .expect("Rust interop package binary should run");
        assert!(
            output.status.success(),
            "package binary should pass: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    pub(crate) fn stdout(&self, entrypoint: &PackageEntrypoint) -> String {
        String::from_utf8_lossy(&self.built_package_output(entrypoint).stdout)
            .trim()
            .to_string()
    }

    pub(crate) fn cargo(&self) -> std::process::Command {
        let mut command = std::process::Command::new("cargo");
        command.current_dir(&self.root);
        self.leases()
            .bind(&mut command)
            .expect("bind fixture Cargo ownership");
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            #[allow(unsafe_code)]
            // SAFETY: umask is allocation-free and runs only in this owned child
            // after fork. The concurrent test parent's creation policy is unchanged.
            unsafe {
                command.pre_exec(|| {
                    libc::umask(0o077);
                    Ok(())
                });
            }
        }
        command
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::other(message)
}

fn validate_inputs(inputs: &Inputs) -> io::Result<()> {
    if inputs.len() > MAX_FILES {
        return Err(invalid("fixture file count exceeds limit"));
    }
    let mut size = 0usize;
    for (path, bytes) in inputs {
        sifr_cache_storage::relative(path)?;
        size = size
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("fixture size overflow"))?;
        if size > MAX_BYTES {
            return Err(invalid("fixture byte size exceeds limit"));
        }
        if path
            .ancestors()
            .skip(1)
            .any(|parent| inputs.contains_key(parent))
        {
            return Err(invalid("fixture input is both file and directory"));
        }
    }
    Ok(())
}

fn read_inputs(root: &Path) -> io::Result<Inputs> {
    fn visit(
        root: &Path,
        path: &Path,
        inputs: &mut Inputs,
        size: &mut usize,
        entries: &mut usize,
    ) -> io::Result<()> {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() {
            return Err(invalid("fixture source symlink"));
        }
        if path != root {
            *entries += 1;
            if *entries > MAX_FILES {
                return Err(invalid("fixture input inventory exceeds limit"));
            }
        }
        if metadata.is_dir() {
            // Existing compiler artifacts are not canonical fixture inputs.
            if path != root && path.file_name().is_some_and(|name| name == "target") {
                return Ok(());
            }
            for entry in fs::read_dir(path)? {
                visit(root, &entry?.path(), inputs, size, entries)?;
            }
        } else if metadata.is_file() {
            let mut bytes = Vec::new();
            File::open(path)?
                .take((MAX_BYTES - *size + 1) as u64)
                .read_to_end(&mut bytes)?;
            *size += bytes.len();
            if *size > MAX_BYTES {
                return Err(invalid("fixture byte size exceeds limit"));
            }
            inputs.insert(
                path.strip_prefix(root)
                    .map_err(|_| invalid("fixture path escaped root"))?
                    .to_owned(),
                bytes,
            );
        } else {
            return Err(invalid("nonregular fixture input"));
        }
        Ok(())
    }
    let mut inputs = Inputs::new();
    visit(root, root, &mut inputs, &mut 0, &mut 0)?;
    validate_inputs(&inputs)?;
    Ok(inputs)
}

/// Validate the entire retained tree before mutation. All writes remain inside
/// the fixture lease, and existing aliases never become a deletion shortcut.
fn retained_paths(root: &Path) -> io::Result<Vec<(PathBuf, bool)>> {
    fn visit(path: &Path, rows: &mut Vec<(PathBuf, bool)>) -> io::Result<()> {
        crate::cache_storage::check_owned(path)?;
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_dir() && !metadata.is_file() {
            return Err(invalid("nonregular retained fixture input"));
        }
        #[cfg(unix)]
        if metadata.is_file() {
            use std::os::unix::fs::MetadataExt;
            if metadata.nlink() != 1 {
                return Err(invalid("retained fixture file has aliases"));
            }
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(path)? {
                visit(&entry?.path(), rows)?;
            }
        }
        rows.push((path.to_owned(), metadata.is_dir()));
        Ok(())
    }
    let mut rows = Vec::new();
    visit(root, &mut rows)?;
    Ok(rows)
}

fn reconcile(root: &Path, inputs: &Inputs) -> io::Result<()> {
    validate_inputs(inputs)?;
    crate::cache_storage::directory(root)?;
    let retained = retained_paths(root)?;
    let directories: BTreeSet<PathBuf> = inputs
        .keys()
        .flat_map(|path| {
            path.ancestors()
                .skip(1)
                .filter(|p| !p.as_os_str().is_empty())
                .map(Path::to_owned)
        })
        .collect();
    // Postorder removes stale directory trees and type conflicts before writes.
    for (path, directory) in retained {
        if path == root {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| invalid("retained input escaped root"))?;
        if directory {
            if !directories.contains(relative) {
                fs::remove_dir(path)?;
            }
        } else if !inputs.contains_key(relative) {
            fs::remove_file(path)?;
        }
    }
    for directory in directories {
        crate::cache_storage::directory(&root.join(directory))?;
    }
    for (path, bytes) in inputs {
        crate::build::native_storage::write_changed(&root.join(path), bytes)?;
    }
    Ok(())
}

#[path = "package_rust_interop_reusable_tests.rs"]
mod tests;
pub(super) fn assert_reconciliation() {
    tests::assert_reconciliation();
}
