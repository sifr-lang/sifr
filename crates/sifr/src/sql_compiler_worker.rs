//! Canonical SQL semantic operations for an invocation-owned native tool.
use sifr_semantic_services::sql_editor::{PreparedSqlProfiles, prepare_sql_profiles};
use sifr_sql_contract::{MigrationSourceDeclaration, ProfileAuthority};
use sifr_sql_tool::compiler_protocol::{CompilerOperations, serve};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::sql_compiler_admission::Admission;
use crate::sql_compiler_inputs::{self as inputs, Inputs};

struct Pipes;
impl Read for Pipes {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        io::stdin().read(bytes)
    }
}
impl Write for Pipes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        io::stdout().write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

struct Prepared {
    inputs: Inputs,
    profiles: PreparedSqlProfiles,
}
struct Worker {
    root: PathBuf,
    semantic: sifr_semantic_services::SemanticContext,
    prepared: Option<Prepared>,
    admission: Result<Option<Admission>, String>,
    migrations: Option<std::collections::BTreeMap<PathBuf, String>>,
}

pub(super) fn run(root: &Path, admission: Option<(&Path, &str)>) -> i32 {
    let mut worker = Worker {
        root: root.to_owned(),
        semantic: sifr_semantic_services::SemanticContext::new(crate::compiler_identity()),
        prepared: None,
        admission: admission
            .map(|(path, hash)| Admission::load(path, hash, root))
            .transpose(),
        migrations: None,
    };
    // Diagnostics are carried in Failed frames. Do not print a second copy or
    // put text on the protocol stdout stream.
    i32::from(serve(&mut Pipes, env!("SIFR_COMPILER_SOURCE_ID"), &mut worker).is_err())
}

impl Worker {
    fn prepare(&mut self) -> Result<&Prepared, String> {
        let admission = self.admission.as_ref().map_err(Clone::clone)?;
        if let Some(admission) = admission {
            admission.verify()?;
        }
        if self.prepared.is_none() {
            let inputs = match admission {
                Some(admission) => admission.inputs()?,
                None => Inputs::capture(&self.root)?,
            };
            let profiles = match &inputs.snapshot {
                Some((snapshot, owner)) => {
                    prepare_sql_profiles(&snapshot.graph, owner).map_err(messages)?
                }
                None => PreparedSqlProfiles::default(),
            };
            inputs.verify(&self.root)?;
            self.prepared = Some(Prepared { inputs, profiles });
        }
        self.prepared
            .as_ref()
            .ok_or_else(|| "SQL compiler preparation is missing".into())
    }
}
impl CompilerOperations for Worker {
    fn profile(&mut self, name: &str) -> Result<ProfileAuthority, String> {
        self.prepare()?;
        let prepared = self
            .prepared
            .as_ref()
            .ok_or("SQL compiler preparation is missing")?;
        prepared.inputs.verify(&self.root)?;
        let result = prepared
            .profiles
            .registry()
            .profile(name)
            .map(|registered| registered.authority().clone())
            .map_err(|error| error.to_string());
        prepared.inputs.verify(&self.root)?;
        result
    }
    fn migration(&mut self, source: &str) -> Result<Vec<MigrationSourceDeclaration>, String> {
        self.prepare()?;
        let prepared = self
            .prepared
            .as_ref()
            .ok_or("SQL compiler preparation is missing")?;
        prepared.inputs.verify(&self.root)?;
        let before = inputs::migration_sources(&self.root, &prepared.inputs.profile_names)?;
        if self
            .migrations
            .as_ref()
            .is_some_and(|previous| previous != &before)
        {
            return Err("SQL compiler migration inputs changed during the session".into());
        }
        self.migrations.get_or_insert_with(|| before.clone());
        if !before.values().any(|bytes| bytes == source) {
            return Err("SQL compiler source is not an admitted migration input".into());
        }
        let result = sifr_semantic_services::compile_sql_migration_source(&self.semantic, source)
            .map_err(messages);
        prepared.inputs.verify(&self.root)?;
        if before != inputs::migration_sources(&self.root, &prepared.inputs.profile_names)? {
            return Err("SQL compiler migration inputs changed during compilation".into());
        }
        result
    }
}

fn messages(failures: Vec<sifr_diagnostics::RenderedDiagnostic>) -> String {
    failures
        .into_iter()
        .map(|failure| failure.message)
        .collect::<Vec<_>>()
        .join("\n")
}
