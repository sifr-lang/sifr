use crate::{SourceDirEntry, SourceProvider, SourceProviderError, SourceText};
use serde::{Deserialize, Serialize};
use sifr_identity::IdentityEncoder;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Ordered operations are the actual resolver's observations, including absence.
/// Directory membership is sorted; search order is never sorted.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Observation {
    File {
        path: PathBuf,
        identity: String,
    },
    Directory {
        path: PathBuf,
        entries: Vec<DirectoryMember>,
    },
    FileProbe {
        path: PathBuf,
        exists: bool,
    },
    DirectoryProbe {
        path: PathBuf,
        exists: bool,
    },
    Canonical {
        path: PathBuf,
        result: PathBuf,
    },
    Failed {
        path: PathBuf,
        operation: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectoryMember {
    pub path: PathBuf,
    pub file: bool,
    pub directory: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedSource {
    pub path: PathBuf,
    pub text: String,
}
impl CapturedSource {
    pub fn identity(&self) -> String {
        let mut id = IdentityEncoder::new("project-source-v1");
        id.field("path", self.path.as_os_str().as_encoded_bytes());
        id.field("bytes", self.text.as_bytes());
        id.finish()
    }
}
/// A capture owns each read's exact bytes. Repeated reads use that same snapshot,
/// including failed reads, rather than racing hashing against a second read.
pub struct CapturingSourceProvider<'a> {
    inner: &'a mut dyn SourceProvider,
    files: BTreeMap<PathBuf, Result<SourceText, SourceProviderError>>,
    observations: Vec<Observation>,
    seen: BTreeSet<Observation>,
    overflow: bool,
}
impl<'a> CapturingSourceProvider<'a> {
    pub fn new(inner: &'a mut dyn SourceProvider) -> Self {
        Self {
            inner,
            files: BTreeMap::new(),
            observations: Vec::new(),
            seen: BTreeSet::new(),
            overflow: false,
        }
    }
    /// Preserve first-occurrence order and distinct outcomes for each operation.
    /// Overflow disables publication, never source computation.
    fn observe(&mut self, observation: Observation) {
        if self.seen.contains(&observation) {
            return;
        }
        if self.seen.len() == 16384 {
            self.overflow = true;
            return;
        }
        self.seen.insert(observation.clone());
        self.observations.push(observation);
    }
    pub fn observations_complete(&self) -> bool {
        !self.overflow
    }
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }
    pub fn sources(&self) -> Vec<CapturedSource> {
        self.files
            .iter()
            .filter_map(|(path, source)| {
                source.as_ref().ok().map(|text| CapturedSource {
                    path: path.clone(),
                    text: text.as_str().to_owned(),
                })
            })
            .collect()
    }
    /// Replay against current authoritative inputs before publishing. A changed
    /// observation is an explicit changed-input outcome, never a reusable record.
    pub fn unchanged(&mut self) -> bool {
        !self.overflow && observations_match(&self.observations, self.inner)
    }
}
fn members(entries: &[SourceDirEntry]) -> Vec<DirectoryMember> {
    let mut output: Vec<_> = entries
        .iter()
        // The compiler's own non-source hint cannot invalidate its publication.
        // Keep the provider's resolver-visible entries unchanged.
        .filter(|entry| {
            !entry.path.file_name().is_some_and(|name| {
                name == ".sifrbuildinfo"
                    || name.to_string_lossy().starts_with(".sifrbuildinfo.stage-")
            })
        })
        .map(|entry| DirectoryMember {
            path: entry.path.clone(),
            file: entry.is_file,
            directory: entry.is_dir,
        })
        .collect();
    output.sort();
    output
}
impl SourceProvider for CapturingSourceProvider<'_> {
    fn read_file(&mut self, path: &Path) -> Result<SourceText, SourceProviderError> {
        let value = self
            .files
            .entry(path.to_owned())
            .or_insert_with(|| self.inner.read_file(path))
            .clone();
        self.observe(match &value {
            Ok(text) => Observation::File {
                path: path.to_owned(),
                identity: CapturedSource {
                    path: path.to_owned(),
                    text: text.as_str().to_owned(),
                }
                .identity(),
            },
            Err(_) => Observation::Failed {
                path: path.to_owned(),
                operation: "file".into(),
            },
        });
        value
    }
    fn read_dir(&mut self, path: &Path) -> Result<Vec<SourceDirEntry>, SourceProviderError> {
        let value = self.inner.read_dir(path);
        self.observe(match &value {
            Ok(entries) => Observation::Directory {
                path: path.to_owned(),
                entries: members(entries),
            },
            Err(_) => Observation::Failed {
                path: path.to_owned(),
                operation: "directory".into(),
            },
        });
        value
    }
    fn is_file(&mut self, path: &Path) -> bool {
        let exists = self.inner.is_file(path);
        self.observe(Observation::FileProbe {
            path: path.to_owned(),
            exists,
        });
        exists
    }
    fn is_dir(&mut self, path: &Path) -> bool {
        let exists = self.inner.is_dir(path);
        self.observe(Observation::DirectoryProbe {
            path: path.to_owned(),
            exists,
        });
        exists
    }
    fn canonicalize(&mut self, path: &Path) -> Result<PathBuf, SourceProviderError> {
        let value = self.inner.canonicalize(path);
        self.observe(match &value {
            Ok(result) => Observation::Canonical {
                path: path.to_owned(),
                result: result.clone(),
            },
            Err(_) => Observation::Failed {
                path: path.to_owned(),
                operation: "canonical".into(),
            },
        });
        value
    }
}
pub fn observations_match(observations: &[Observation], provider: &mut dyn SourceProvider) -> bool {
    observations.iter().all(|observation| match observation {
        Observation::File { path, identity } => provider.read_file(path).is_ok_and(|text| {
            CapturedSource {
                path: path.clone(),
                text: text.as_str().into(),
            }
            .identity()
                == *identity
        }),
        Observation::Directory { path, entries } => provider
            .read_dir(path)
            .is_ok_and(|actual| members(&actual) == *entries),
        Observation::FileProbe { path, exists } => provider.is_file(path) == *exists,
        Observation::DirectoryProbe { path, exists } => provider.is_dir(path) == *exists,
        Observation::Canonical { path, result } => provider
            .canonicalize(path)
            .is_ok_and(|actual| actual == *result),
        // Failed reads are not deterministic source errors: do not authorize
        // publication based on the continued presence of an environmental error.
        Observation::Failed { .. } => false,
    })
}

/// Explicitly resolved inputs supplied by package/component owners. Presentation
/// choices have no field here. Diagnostic/lint policy does.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticInputs {
    pub compiler: String,
    pub metadata: String,
    pub target: String,
    pub workspace_and_source_policy: String,
    pub package_and_lock: String,
    pub language_options: String,
    pub diagnostic_policy: String,
    pub components: BTreeMap<String, String>,
    /// The subsystem owner declares every SQL/Python/interop observation it
    /// requires; an unresolved observation never yields a complete input contract.
    pub required_external: BTreeSet<String>,
    pub external: BTreeMap<String, String>,
}
impl SemanticInputs {
    pub fn complete(&self) -> bool {
        [
            &self.compiler,
            &self.metadata,
            &self.target,
            &self.workspace_and_source_policy,
            &self.package_and_lock,
            &self.language_options,
            &self.diagnostic_policy,
        ]
        .iter()
        .all(|value| !value.is_empty())
            && self.components.values().all(|value| !value.is_empty())
            && self.required_external.iter().all(|name| {
                self.external
                    .get(name)
                    .is_some_and(|value| !value.is_empty())
            })
    }

    pub fn identity(&self) -> Result<String, serde_json::Error> {
        identity("project-context-v1", self)
    }
}
pub fn identity<T: Serialize>(domain: &str, value: &T) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    let mut id = IdentityEncoder::new(domain);
    id.field("record", &bytes);
    Ok(id.finish())
}
