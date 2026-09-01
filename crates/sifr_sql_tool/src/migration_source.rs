use sifr_sql_contract::{
    CompiledMigrationGraph, DdlReflection, MIGRATION_GRAPH_FORMAT_VERSION, MigrationBaseline,
    MigrationCompileError, MigrationCompileErrorKind, MigrationCompiler, MigrationDefinition,
    MigrationDialect, MigrationGraphDefinition, MigrationNodeId, MigrationProviderConstraint,
    MigrationSourceDeclaration, MigrationSourceStepKind, MigrationStepDefinition,
    MigrationStepKind, ProviderAnalysis, SchemaIr, TransactionBoundary, TransactionRequirement,
    schema_fingerprint, semantic_diff,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationSourceInputs {
    pub baselines: BTreeMap<MigrationNodeId, MigrationBaseline>,
    pub declarations: Vec<MigrationSourceDeclaration>,
}

/// Loads the closed migration source tree at `migrations/<profile>`. Checked
/// source files live directly in that directory and baseline `SchemaIR` records
/// live in its `baselines` child. Generated artifacts are deliberately written
/// elsewhere under `.sifr`.
pub fn load_migration_source_inputs<C>(
    workspace_root: &Path,
    profile: &str,
    mut compile_source: C,
) -> Result<MigrationSourceInputs, MigrationCompileError>
where
    C: FnMut(&str) -> Result<Vec<MigrationSourceDeclaration>, String>,
{
    let mut profile_components = Path::new(profile).components();
    if profile.is_empty()
        || !matches!(profile_components.next(), Some(Component::Normal(_)))
        || profile_components.next().is_some()
    {
        return Err(error(
            "migration profile must be one normalized path segment",
        ));
    }
    let root = workspace_root.join("migrations").join(profile);
    let source_paths = regular_files_with_extension(&root, "sifr")?;
    if source_paths.is_empty() {
        return Err(error(format!(
            "migration profile '{profile}' has no .sifr sources in '{}'",
            root.display()
        )));
    }
    let mut declarations = Vec::new();
    let mut identities = BTreeSet::new();
    for path in source_paths {
        let source = fs::read_to_string(&path).map_err(|_| {
            error(format!(
                "migration source '{}' is not readable UTF-8",
                path.display()
            ))
        })?;
        let checked = compile_source(&source).map_err(|messages| {
            error(format!(
                "migration source '{}' did not type-check: {messages}",
                path.display()
            ))
        })?;
        if checked.len() != 1 {
            return Err(error(format!(
                "migration source '{}' must declare exactly one migration",
                path.display()
            )));
        }
        for declaration in checked {
            if path.file_stem().and_then(|stem| stem.to_str()) != Some(declaration.id.as_str()) {
                return Err(error(format!(
                    "migration source filename '{}' must equal identity '{}'",
                    path.display(),
                    declaration.id
                )));
            }
            if !identities.insert(declaration.id.clone()) {
                return Err(error(format!(
                    "migration identity '{}' is declared more than once",
                    declaration.id
                )));
            }
            declarations.push(declaration);
        }
    }
    declarations.sort_by(|left, right| left.id.cmp(&right.id));

    let baseline_root = root.join("baselines");
    let baseline_paths = regular_files_with_extension(&baseline_root, "json")?;
    if baseline_paths.is_empty() {
        return Err(error(format!(
            "migration profile '{profile}' has no baseline SchemaIR records in '{}'",
            baseline_root.display()
        )));
    }
    let mut baselines = BTreeMap::new();
    for path in baseline_paths {
        let bytes = fs::read(&path).map_err(|_| {
            error(format!(
                "migration baseline '{}' cannot be read",
                path.display()
            ))
        })?;
        let baseline = serde_json::from_slice::<MigrationBaseline>(&bytes).map_err(|_| {
            error(format!(
                "migration baseline '{}' is not a canonical MigrationBaseline record",
                path.display()
            ))
        })?;
        let file_identity = path.file_stem().and_then(|stem| stem.to_str());
        if file_identity != Some(baseline.id.as_str()) {
            return Err(error(format!(
                "migration baseline filename '{}' must equal identity '{}'",
                path.display(),
                baseline.id
            )));
        }
        if baselines.insert(baseline.id.clone(), baseline).is_some() {
            return Err(error("migration baseline identity is duplicated"));
        }
    }
    Ok(MigrationSourceInputs {
        baselines,
        declarations,
    })
}

fn regular_files_with_extension(
    directory: &Path,
    extension: &str,
) -> Result<Vec<PathBuf>, MigrationCompileError> {
    let directory_metadata = fs::symlink_metadata(directory).map_err(|_| {
        error(format!(
            "migration source directory '{}' cannot be read",
            directory.display()
        ))
    })?;
    if !directory_metadata.file_type().is_dir() {
        return Err(error(format!(
            "migration source directory '{}' must be a real directory",
            directory.display()
        )));
    }
    let entries = fs::read_dir(directory).map_err(|_| {
        error(format!(
            "migration source directory '{}' cannot be read",
            directory.display()
        ))
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| error("migration source directory entry is invalid"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some(extension) {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| error("migration source metadata cannot be read"))?;
        if !metadata.file_type().is_file() {
            return Err(error(format!(
                "migration input '{}' must be a regular file",
                path.display()
            )));
        }
        paths.push(path);
    }
    paths.sort();
    Ok(paths)
}

/// Converts checked Sifr migration declarations into the provider-neutral graph
/// contract, then delegates final validation and compilation to the selected
/// provider dialect. The analysis callback must use the same provider parser
/// and semantic catalog as ordinary application queries.
pub fn compile_migration_sources<D, A>(
    dialect: &D,
    target_schema: SchemaIr,
    baselines: BTreeMap<MigrationNodeId, MigrationBaseline>,
    declarations: Vec<MigrationSourceDeclaration>,
    mut analyze: A,
) -> Result<CompiledMigrationGraph, MigrationCompileError>
where
    D: MigrationDialect,
    A: FnMut(&SchemaIr, &str) -> Result<ProviderAnalysis, String>,
{
    let mut remaining = declarations
        .into_iter()
        .map(|declaration| (declaration.id.clone(), declaration))
        .collect::<BTreeMap<_, _>>();
    if remaining.is_empty() {
        return Err(error("migration source set is empty"));
    }
    let mut schemas = baselines
        .iter()
        .map(|(id, baseline)| (id.clone(), baseline.schema.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut migrations = BTreeMap::new();

    while !remaining.is_empty() {
        let ready = remaining
            .iter()
            .find(|(_, declaration)| {
                declaration
                    .parents
                    .iter()
                    .all(|parent| schemas.contains_key(parent))
            })
            .map(|(id, _)| id.clone())
            .ok_or_else(|| {
                error("migration sources contain an unknown parent, cycle, or disconnected graph")
            })?;
        let declaration = remaining
            .remove(&ready)
            .ok_or_else(|| error("ready migration declaration disappeared"))?;
        let (definition, output) = build_definition(dialect, &schemas, declaration, &mut analyze)?;
        schemas.insert(definition.id.clone(), output);
        migrations.insert(definition.id.clone(), definition);
    }

    MigrationCompiler::new(dialect).compile(&MigrationGraphDefinition {
        format_version: MIGRATION_GRAPH_FORMAT_VERSION,
        baselines,
        migrations,
        target_schema,
    })
}

fn build_definition<D, A>(
    dialect: &D,
    schemas: &BTreeMap<MigrationNodeId, SchemaIr>,
    declaration: MigrationSourceDeclaration,
    analyze: &mut A,
) -> Result<(MigrationDefinition, SchemaIr), MigrationCompileError>
where
    D: MigrationDialect,
    A: FnMut(&SchemaIr, &str) -> Result<ProviderAnalysis, String>,
{
    let parents = declaration.parents.into_iter().collect::<BTreeSet<_>>();
    if parents.is_empty() {
        return Err(error("migration declaration has no parent"));
    }
    let canonical_input = parents
        .iter()
        .next()
        .and_then(|parent| schemas.get(parent))
        .ok_or_else(|| error("migration parent schema is missing"))?;
    for parent in &parents {
        let schema = schemas
            .get(parent)
            .ok_or_else(|| error("migration parent schema is missing"))?;
        if !semantic_diff(canonical_input, schema).is_empty() {
            return Err(error(
                "a source migration can merge only semantically identical parent schemas",
            ));
        }
    }
    let input_fingerprints = parents
        .iter()
        .map(|parent| {
            let schema = schemas
                .get(parent)
                .ok_or_else(|| error("migration parent schema is missing"))?;
            schema_fingerprint(schema)
                .map(|fingerprint| (parent.clone(), fingerprint.as_str().to_string()))
                .map_err(|failure| error(failure.to_string()))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let (schema, steps, mut required_capabilities, transaction_boundaries) =
        compile_source_steps(dialect, canonical_input.clone(), declaration.steps, analyze)?;
    let transaction_requirement = source_transaction_requirement(&steps, transaction_boundaries)?;
    let rollback = if let Some(source_steps) = declaration.rollback {
        let (restored, steps, capabilities, boundaries) =
            compile_source_steps(dialect, schema.clone(), source_steps, analyze)?;
        if !semantic_diff(&restored, canonical_input).is_empty() {
            return Err(error(
                "source rollback does not reproduce the migration parent schema",
            ));
        }
        if transaction_requirement == TransactionRequirement::Required {
            let _ = source_transaction_requirement(&steps, boundaries).and_then(|requirement| {
                if requirement == TransactionRequirement::Required {
                    Ok(requirement)
                } else {
                    Err(error(
                        "transaction-required migration needs an outer rollback transaction",
                    ))
                }
            })?;
        }
        required_capabilities.extend(capabilities);
        Some(steps)
    } else {
        None
    };
    let output_fingerprint = schema_fingerprint(&schema)
        .map_err(|failure| error(failure.to_string()))?
        .as_str()
        .to_string();
    let definition = MigrationDefinition {
        id: declaration.id,
        parents,
        input_fingerprints,
        output_fingerprint,
        provider: MigrationProviderConstraint {
            family: dialect.family().to_string(),
            minimum_server_version: Some(dialect.server_version().to_string()),
            required_capabilities,
        },
        transaction_requirement,
        steps,
        rollback,
        author: declaration.author,
        created_at: declaration.created_at,
    };
    Ok((definition, schema))
}

fn source_transaction_requirement(
    steps: &[MigrationStepDefinition],
    boundaries: u8,
) -> Result<TransactionRequirement, MigrationCompileError> {
    if boundaries == 0 {
        return Ok(TransactionRequirement::Optional);
    }
    let enclosed = matches!(
        steps.first().map(|step| &step.kind),
        Some(MigrationStepKind::Transaction {
            boundary: TransactionBoundary::Begin
        })
    ) && matches!(
        steps.last().map(|step| &step.kind),
        Some(MigrationStepKind::Transaction {
            boundary: TransactionBoundary::Commit
        })
    );
    if boundaries == 2 && enclosed {
        Ok(TransactionRequirement::Required)
    } else {
        Err(error(
            "source migration transaction boundaries must form one complete outer pair",
        ))
    }
}

fn compile_source_steps<D, A>(
    dialect: &D,
    mut schema: SchemaIr,
    source_steps: Vec<sifr_sql_contract::MigrationSourceStep>,
    analyze: &mut A,
) -> Result<(SchemaIr, Vec<MigrationStepDefinition>, BTreeSet<String>, u8), MigrationCompileError>
where
    D: MigrationDialect,
    A: FnMut(&SchemaIr, &str) -> Result<ProviderAnalysis, String>,
{
    let mut steps = Vec::with_capacity(source_steps.len());
    let mut required_capabilities = BTreeSet::new();
    let mut transaction_boundaries = 0_u8;
    for source_step in source_steps {
        let kind = match source_step.kind {
            MigrationSourceStepKind::Ddl { statement } => {
                schema = reflected_schema(dialect, &schema, &statement)?;
                MigrationStepKind::Ddl {
                    statement,
                    declared_effect: None,
                }
            }
            MigrationSourceStepKind::SqlData { statement } => {
                let analysis = analyze(&schema, &statement).map_err(error)?;
                required_capabilities.extend(analysis.required_capabilities.iter().cloned());
                MigrationStepKind::SqlData {
                    statement,
                    analysis,
                }
            }
            MigrationSourceStepKind::Assertion { statement } => {
                let analysis = analyze(&schema, &statement).map_err(error)?;
                required_capabilities.extend(analysis.required_capabilities.iter().cloned());
                MigrationStepKind::Assertion {
                    statement,
                    analysis,
                }
            }
            MigrationSourceStepKind::RecoveryPoint { name } => {
                MigrationStepKind::RecoveryPoint { name }
            }
            MigrationSourceStepKind::Begin => {
                transaction_boundaries = transaction_boundaries.saturating_add(1);
                MigrationStepKind::Transaction {
                    boundary: TransactionBoundary::Begin,
                }
            }
            MigrationSourceStepKind::Commit => {
                transaction_boundaries = transaction_boundaries.saturating_add(1);
                MigrationStepKind::Transaction {
                    boundary: TransactionBoundary::Commit,
                }
            }
        };
        steps.push(MigrationStepDefinition {
            id: source_step.id,
            kind,
        });
    }
    Ok((schema, steps, required_capabilities, transaction_boundaries))
}

fn reflected_schema<D: MigrationDialect>(
    dialect: &D,
    input: &SchemaIr,
    statement: &str,
) -> Result<SchemaIr, MigrationCompileError> {
    match dialect.reflect_ddl(input, statement)? {
        DdlReflection::Reflected { schema, .. } => Ok(schema),
        DdlReflection::Opaque => Err(MigrationCompileError::new(
            MigrationCompileErrorKind::DdlReflection,
            "source-level opaque DDL needs a provider-owned typed schema-effect form",
        )),
    }
}

fn error(message: impl Into<String>) -> MigrationCompileError {
    MigrationCompileError::new(MigrationCompileErrorKind::InvalidGraph, message)
}
