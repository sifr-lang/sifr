//! Caller-owned input admission, retained outside the confined tool's filesystem.
use crate::sql_compiler_inputs::Inputs;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const LIMIT: u64 = 16 * 1024 * 1024;
const CHANGED: &str = "SQL compiler admitted project inputs changed";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Admission {
    root: PathBuf,
    files: BTreeMap<PathBuf, (PathBuf, String)>,
    absent: BTreeSet<PathBuf>,
    profile_names: BTreeSet<String>,
    migrations: BTreeMap<PathBuf, (PathBuf, String)>,
}
impl Admission {
    fn capture(root: &Path) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|error| error.to_string())?;
        let inputs = Inputs::capture(&root)?;
        let absent = ["Cargo.toml", "Cargo.lock", "sifr.toml"]
            .into_iter()
            .map(|name| root.join(name))
            .filter(|path| !inputs.files.contains_key(path))
            .collect();
        let migrations = Self::migration_pins(&root, &inputs.profile_names)?;
        let result = Self {
            root,
            files: inputs.files,
            absent,
            profile_names: inputs.profile_names,
            migrations,
        };
        result.verify()?;
        Ok(result)
    }

    fn migration_pins(
        root: &Path,
        profiles: &BTreeSet<String>,
    ) -> Result<BTreeMap<PathBuf, (PathBuf, String)>, String> {
        crate::sql_compiler_inputs::migration_sources(root, profiles)?
            .into_keys()
            .map(|path| crate::sql_compiler_inputs::pin(&path).map(|pin| (path, pin)))
            .collect()
    }

    pub(crate) fn verify(&self) -> Result<(), String> {
        for (path, expected) in &self.files {
            if crate::sql_compiler_inputs::pin(path)
                .map_err(|error| format!("{CHANGED}: {error}"))?
                != *expected
            {
                return Err(CHANGED.into());
            }
        }
        for path in &self.absent {
            if path.try_exists().map_err(|error| error.to_string())? {
                return Err(CHANGED.into());
            }
        }
        if Self::migration_pins(&self.root, &self.profile_names)
            .map_err(|error| format!("{CHANGED}: {error}"))?
            != self.migrations
        {
            return Err(CHANGED.into());
        }
        Ok(())
    }

    pub(crate) fn verify_inputs(&self, inputs: &Inputs) -> Result<(), String> {
        self.verify()?;
        if inputs.files != self.files || inputs.profile_names != self.profile_names {
            return Err(CHANGED.into());
        }
        Ok(())
    }

    pub(crate) fn load(path: &Path, expected_hash: &str, root: &Path) -> Result<Self, String> {
        use sha2::{Digest, Sha256};
        let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
        let mut bytes = Vec::new();
        file.take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        if bytes.len() as u64 > LIMIT || format!("{:x}", Sha256::digest(&bytes)) != expected_hash {
            return Err("SQL compiler admission snapshot changed or exceeded its bound".into());
        }
        let admitted: Self = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if admitted.root != root.canonicalize().map_err(|error| error.to_string())? {
            return Err("SQL compiler admission belongs to a different project".into());
        }
        admitted.verify()?;
        Ok(admitted)
    }
}

/// The file is never exposed through the tool channel or mounted into its sandbox.
/// Its digest is passed only to the selected compiler; Drop removes the receipt.
pub(crate) struct AdmissionFile {
    file: tempfile::NamedTempFile,
    hash: String,
    admitted: Admission,
}
impl AdmissionFile {
    pub(crate) fn capture(root: &Path) -> Result<Self, String> {
        use sha2::{Digest, Sha256};
        let admitted = Admission::capture(root)?;
        let bytes = serde_json::to_vec(&admitted).map_err(|error| error.to_string())?;
        if bytes.len() as u64 > LIMIT {
            return Err("SQL compiler admission snapshot exceeded its bound".into());
        }
        let hash = format!("{:x}", Sha256::digest(&bytes));
        let mut file = tempfile::NamedTempFile::new().map_err(|error| error.to_string())?;
        if file
            .path()
            .canonicalize()
            .map_err(|error| error.to_string())?
            .starts_with(&admitted.root)
        {
            return Err("SQL compiler admission storage must be outside the project".into());
        }
        file.write_all(&bytes).map_err(|error| error.to_string())?;
        Ok(Self {
            file,
            hash,
            admitted,
        })
    }
    pub(crate) fn verify(&self) -> Result<(), String> {
        self.admitted.verify()
    }
    pub(crate) fn configure(&self, command: &mut std::process::Command) {
        command
            .arg("--admission")
            .arg(self.file.path())
            .arg("--admission-sha256")
            .arg(&self.hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_binds_bytes_root_and_missing_inputs() {
        let root = tempfile::tempdir().unwrap();
        let receipt = AdmissionFile::capture(root.path()).unwrap();
        Admission::load(receipt.file.path(), &receipt.hash, root.path()).unwrap();
        let other = tempfile::tempdir().unwrap();
        assert!(Admission::load(receipt.file.path(), &receipt.hash, other.path()).is_err());
        assert!(Admission::load(receipt.file.path(), "wrong", root.path()).is_err());
        std::fs::write(root.path().join("Cargo.lock"), "new").unwrap();
        assert!(receipt.verify().unwrap_err().contains("changed"));
        assert!(Admission::load(receipt.file.path(), &receipt.hash, root.path()).is_err());
    }
    #[test]
    fn snapshot_rejects_manifest_drift_before_loading_the_graph() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("Cargo.toml"), "# initial").unwrap();
        let receipt = AdmissionFile::capture(root.path()).unwrap();
        std::fs::write(root.path().join("Cargo.toml"), "invalid changed TOML").unwrap();
        let error = Admission::load(receipt.file.path(), &receipt.hash, root.path())
            .err()
            .unwrap();
        assert!(error.contains("changed"), "{error}");
    }
}
