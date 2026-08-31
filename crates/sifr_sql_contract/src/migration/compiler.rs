use super::{
    BackfillContract, CompiledMigration, CompiledMigrationGraph, CompiledMigrationPath,
    CompiledMigrationStep, CompiledStepKind, DataCallbackContract, DdlReflection, DdlRisk,
    MIGRATION_GRAPH_FORMAT_VERSION, MigrationCompileError, MigrationCompileErrorKind,
    MigrationDefinition, MigrationGraphDefinition, MigrationImpact, MigrationNodeId,
    MigrationProviderConstraint, MigrationStateIdentity, MigrationStepDefinition,
    MigrationStepKind, ReplayPolicy, TransactionBoundary, TransactionRequirement,
    topological_order,
};
use crate::{
    Cardinality, Nullability, ObjectChangeKind, ObjectId, ProviderAnalysis, QueryEffect, SchemaIr,
    SifrType, schema_fingerprint, semantic_diff,
};
use semver::Version;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

type StepCompilation = (
    SchemaIr,
    CompiledStepKind,
    BTreeSet<ObjectId>,
    BTreeSet<ObjectId>,
    DdlRisk,
);

pub trait MigrationDialect {
    fn family(&self) -> &str;
    fn server_version(&self) -> &str;
    fn capabilities(&self) -> &BTreeSet<String>;

    fn reflect_ddl(
        &self,
        input: &SchemaIr,
        statement: &str,
    ) -> Result<DdlReflection, MigrationCompileError>;
}

pub struct MigrationCompiler<'a, D: MigrationDialect> {
    dialect: &'a D,
}

impl<'a, D: MigrationDialect> MigrationCompiler<'a, D> {
    #[must_use]
    pub fn new(dialect: &'a D) -> Self {
        Self { dialect }
    }

    pub fn compile(
        &self,
        graph: &MigrationGraphDefinition,
    ) -> Result<CompiledMigrationGraph, MigrationCompileError> {
        if graph.format_version != MIGRATION_GRAPH_FORMAT_VERSION {
            return Err(error(
                MigrationCompileErrorKind::FormatVersion,
                "migration graph format version is not supported",
            ));
        }
        let order = topological_order(graph)?;
        self.validate_schema_provider(&graph.target_schema)?;
        let target_fingerprint = fingerprint(&graph.target_schema)?;
        let mut schemas = BTreeMap::<MigrationNodeId, SchemaIr>::new();
        let mut baseline_fingerprints = BTreeMap::new();
        for (id, baseline) in &graph.baselines {
            self.validate_schema_provider(&baseline.schema)?;
            let value = fingerprint(&baseline.schema)?;
            baseline_fingerprints.insert(id.clone(), value);
            schemas.insert(id.clone(), baseline.schema.clone());
        }

        let mut compiled = BTreeMap::new();
        let mut impacts = Vec::new();
        for migration_id in &order {
            let migration = graph.migrations.get(migration_id).ok_or_else(|| {
                error(
                    MigrationCompileErrorKind::InvalidGraph,
                    "topological migration is missing",
                )
            })?;
            self.validate_provider_constraint(&migration.provider)?;
            Self::validate_definition(migration)?;
            let mut paths = BTreeMap::new();
            let mut canonical_output = None::<SchemaIr>;
            for parent in &migration.parents {
                let input = schemas.get(parent).ok_or_else(|| {
                    error(
                        MigrationCompileErrorKind::InvalidGraph,
                        format!(
                            "migration '{}' parent '{parent}' has no schema",
                            migration.id
                        ),
                    )
                })?;
                let actual_input = fingerprint(input)?;
                let declared_input = migration.input_fingerprints.get(parent).ok_or_else(|| {
                    error(
                        MigrationCompileErrorKind::FingerprintMismatch,
                        "migration parent has no declared input fingerprint",
                    )
                })?;
                if declared_input != &actual_input {
                    return Err(error(
                        MigrationCompileErrorKind::FingerprintMismatch,
                        format!(
                            "migration '{}' input fingerprint does not match parent '{parent}'",
                            migration.id
                        ),
                    ));
                }
                let (path, output, mut path_impacts) =
                    self.compile_path(migration, parent, input, false)?;
                if path.output_fingerprint != migration.output_fingerprint {
                    return Err(error(
                        MigrationCompileErrorKind::FingerprintMismatch,
                        format!(
                            "migration '{}' output fingerprint is incorrect",
                            migration.id
                        ),
                    ));
                }
                if let Some(expected) = canonical_output.as_ref() {
                    if !semantic_diff(expected, &output).is_empty() {
                        return Err(error(
                            MigrationCompileErrorKind::InvalidGraph,
                            format!(
                                "merge migration '{}' produces different schemas by parent",
                                migration.id
                            ),
                        ));
                    }
                } else {
                    canonical_output = Some(output.clone());
                }
                impacts.append(&mut path_impacts);
                paths.insert(parent.clone(), path);
            }
            let output = canonical_output.ok_or_else(|| {
                error(
                    MigrationCompileErrorKind::InvalidGraph,
                    "migration has no compiled parent path",
                )
            })?;
            let checksum = checksum(&(
                &migration.id,
                &migration.parents,
                &migration.provider,
                migration.transaction_requirement,
                &paths,
                &migration.author,
                &migration.created_at,
            ))?;
            compiled.insert(
                migration.id.clone(),
                CompiledMigration {
                    id: migration.id.clone(),
                    parents: migration.parents.clone(),
                    provider: migration.provider.clone(),
                    transaction_requirement: migration.transaction_requirement,
                    checksum,
                    paths,
                    author: migration.author.clone(),
                    created_at: migration.created_at.clone(),
                },
            );
            schemas.insert(migration.id.clone(), output);
        }
        let head = order.last().cloned().ok_or_else(|| {
            error(
                MigrationCompileErrorKind::InvalidGraph,
                "migration graph has no head",
            )
        })?;
        let head_schema = schemas.get(&head).ok_or_else(|| {
            error(
                MigrationCompileErrorKind::InvalidGraph,
                "migration head has no schema",
            )
        })?;
        if !semantic_diff(head_schema, &graph.target_schema).is_empty()
            || fingerprint(head_schema)? != target_fingerprint
        {
            return Err(error(
                MigrationCompileErrorKind::FingerprintMismatch,
                "migration head does not reproduce the canonical target schema",
            ));
        }
        impacts.sort_by(|left, right| {
            (&left.migration, &left.step).cmp(&(&right.migration, &right.step))
        });
        Ok(CompiledMigrationGraph {
            format_version: MIGRATION_GRAPH_FORMAT_VERSION,
            provider_family: self.dialect.family().to_string(),
            target_fingerprint,
            head,
            topological_order: order,
            baseline_fingerprints,
            migrations: compiled,
            impacts,
        })
    }

    fn compile_path(
        &self,
        migration: &MigrationDefinition,
        parent: &MigrationNodeId,
        input: &SchemaIr,
        rollback: bool,
    ) -> Result<(CompiledMigrationPath, SchemaIr, Vec<MigrationImpact>), MigrationCompileError>
    {
        let definitions = if rollback {
            migration.rollback.as_deref().ok_or_else(|| {
                error(
                    MigrationCompileErrorKind::InvalidRollback,
                    "rollback compilation requires an explicit reverse plan",
                )
            })?
        } else {
            &migration.steps
        };
        let mut schema = input.clone();
        let path_input = fingerprint(input)?;
        let mut steps = Vec::with_capacity(definitions.len());
        let mut impacts = Vec::new();
        let mut seen = BTreeSet::new();
        let mut transaction_open = false;
        let mut transaction_count = 0_u32;
        let mut recovery_names = BTreeSet::new();
        for (index, definition) in definitions.iter().enumerate() {
            if !seen.insert(&definition.id) {
                return Err(error(
                    MigrationCompileErrorKind::InvalidStep,
                    format!(
                        "migration '{}' has duplicate step '{}'",
                        migration.id, definition.id
                    ),
                ));
            }
            let input_fingerprint = fingerprint(&schema)?;
            let input_state =
                state_identity(&migration.id, parent, index, &input_fingerprint, rollback);
            let (next_schema, kind, referenced, affected, risk) = self.compile_step(
                definition,
                &schema,
                &mut transaction_open,
                &mut transaction_count,
                &mut recovery_names,
            )?;
            let output_fingerprint = fingerprint(&next_schema)?;
            let output_state = state_identity(
                &migration.id,
                parent,
                index.saturating_add(1),
                &output_fingerprint,
                rollback,
            );
            let step_checksum = checksum(&(
                &migration.id,
                parent,
                &definition.id,
                &input_fingerprint,
                &output_fingerprint,
                &kind,
                &referenced,
                &affected,
            ))?;
            let destructive_objects = semantic_diff(&schema, &next_schema)
                .objects
                .into_iter()
                .filter(|change| change.kind == ObjectChangeKind::Removed)
                .map(|change| change.identity)
                .collect::<BTreeSet<_>>();
            if !destructive_objects.is_empty()
                || !risk.lock_risks.is_empty()
                || !risk.data_rewrites.is_empty()
            {
                impacts.push(MigrationImpact {
                    migration: migration.id.clone(),
                    step: definition.id.clone(),
                    destructive_objects,
                    lock_risks: risk.lock_risks,
                    data_rewrites: risk.data_rewrites,
                });
            }
            steps.push(CompiledMigrationStep {
                id: definition.id.clone(),
                input_state,
                output_state,
                input_fingerprint,
                output_fingerprint,
                checksum: step_checksum,
                referenced_objects: referenced,
                affected_objects: affected,
                kind,
            });
            schema = next_schema;
        }
        if transaction_open {
            return Err(error(
                MigrationCompileErrorKind::InvalidTransaction,
                "migration leaves an explicit transaction open",
            ));
        }
        Self::validate_transaction_requirement(
            migration.transaction_requirement,
            transaction_count,
            definitions,
        )?;
        let output_fingerprint = fingerprint(&schema)?;
        if rollback {
            let expected = migration.input_fingerprints.get(parent).ok_or_else(|| {
                error(
                    MigrationCompileErrorKind::InvalidRollback,
                    "rollback parent fingerprint is missing",
                )
            })?;
            if &output_fingerprint != expected {
                return Err(error(
                    MigrationCompileErrorKind::InvalidRollback,
                    format!(
                        "rollback for migration '{}' does not reproduce parent '{parent}'",
                        migration.id
                    ),
                ));
            }
        }
        let rollback_steps = if !rollback && migration.rollback.is_some() {
            let (compiled, _, _) = self.compile_path(migration, parent, &schema, true)?;
            Some(compiled.steps)
        } else {
            None
        };
        Ok((
            CompiledMigrationPath {
                parent: parent.clone(),
                input_fingerprint: path_input,
                output_fingerprint,
                steps,
                rollback: rollback_steps,
            },
            schema,
            impacts,
        ))
    }

    fn compile_step(
        &self,
        definition: &MigrationStepDefinition,
        schema: &SchemaIr,
        transaction_open: &mut bool,
        transaction_count: &mut u32,
        recovery_names: &mut BTreeSet<String>,
    ) -> Result<StepCompilation, MigrationCompileError> {
        match &definition.kind {
            MigrationStepKind::Ddl {
                statement,
                declared_effect,
            } => {
                if statement.trim().is_empty() {
                    return Err(error(
                        MigrationCompileErrorKind::InvalidStep,
                        "DDL migration steps require a statement",
                    ));
                }
                match self.dialect.reflect_ddl(schema, statement)? {
                    DdlReflection::Reflected {
                        schema: output,
                        risk,
                    } => {
                        if declared_effect
                            .as_deref()
                            .is_some_and(|declared| !semantic_diff(&output, declared).is_empty())
                        {
                            return Err(error(
                                MigrationCompileErrorKind::DdlReflection,
                                "declared DDL effect disagrees with provider reflection",
                            ));
                        }
                        let (referenced, affected) = schema_change_sets(schema, &output);
                        Ok((
                            output,
                            CompiledStepKind::ReflectedDdl {
                                statement: statement.clone(),
                            },
                            referenced,
                            affected,
                            risk,
                        ))
                    }
                    DdlReflection::Opaque => {
                        let output = declared_effect.as_deref().cloned().ok_or_else(|| {
                            error(
                                MigrationCompileErrorKind::DdlReflection,
                                "opaque DDL requires an explicit schema effect",
                            )
                        })?;
                        self.validate_schema_provider(&output)?;
                        if semantic_diff(schema, &output).is_empty() {
                            return Err(error(
                                MigrationCompileErrorKind::DdlReflection,
                                "opaque DDL must declare a non-empty schema effect",
                            ));
                        }
                        let (referenced, affected) = schema_change_sets(schema, &output);
                        Ok((
                            output,
                            CompiledStepKind::DeclaredDdl {
                                statement: statement.clone(),
                            },
                            referenced,
                            affected,
                            DdlRisk::default(),
                        ))
                    }
                }
            }
            MigrationStepKind::SqlData { analysis } => {
                validate_data_analysis(analysis, schema)?;
                Ok((
                    schema.clone(),
                    CompiledStepKind::SqlData {
                        normalized_statement: analysis.normalized_statement.clone(),
                    },
                    analysis.effects.referenced_objects.clone(),
                    analysis.effects.affected_objects.clone(),
                    data_rewrite_risk(analysis.effects.affected_objects.iter()),
                ))
            }
            MigrationStepKind::SifrData { callback } => {
                validate_callback(callback, schema)?;
                Ok((
                    schema.clone(),
                    CompiledStepKind::SifrData {
                        callback: callback.symbol.clone(),
                    },
                    callback.referenced_objects.clone(),
                    callback.affected_objects.clone(),
                    data_rewrite_risk(callback.affected_objects.iter()),
                ))
            }
            MigrationStepKind::Assertion { analysis } => {
                validate_assertion(analysis, schema)?;
                Ok((
                    schema.clone(),
                    CompiledStepKind::Assertion {
                        normalized_statement: analysis.normalized_statement.clone(),
                    },
                    analysis.effects.referenced_objects.clone(),
                    BTreeSet::new(),
                    DdlRisk::default(),
                ))
            }
            MigrationStepKind::Backfill { contract } => {
                validate_backfill(contract, schema)?;
                Ok((
                    schema.clone(),
                    CompiledStepKind::Backfill {
                        normalized_statement: contract.analysis.normalized_statement.clone(),
                        maximum_batch_rows: contract.maximum_batch_rows,
                        replay: contract.replay.clone(),
                    },
                    contract.analysis.effects.referenced_objects.clone(),
                    contract.analysis.effects.affected_objects.clone(),
                    data_rewrite_risk(contract.analysis.effects.affected_objects.iter()),
                ))
            }
            MigrationStepKind::Transaction { boundary } => {
                match boundary {
                    TransactionBoundary::Begin if !*transaction_open => {
                        *transaction_open = true;
                        *transaction_count = transaction_count.saturating_add(1);
                    }
                    TransactionBoundary::Commit if *transaction_open => {
                        *transaction_open = false;
                    }
                    _ => {
                        return Err(error(
                            MigrationCompileErrorKind::InvalidTransaction,
                            "transaction boundaries are nested or unbalanced",
                        ));
                    }
                }
                Ok((
                    schema.clone(),
                    CompiledStepKind::Transaction {
                        boundary: *boundary,
                    },
                    BTreeSet::new(),
                    BTreeSet::new(),
                    DdlRisk::default(),
                ))
            }
            MigrationStepKind::RecoveryPoint { name } => {
                if name.is_empty()
                    || name.len() > 128
                    || name.chars().any(char::is_control)
                    || !recovery_names.insert(name.clone())
                {
                    return Err(error(
                        MigrationCompileErrorKind::InvalidStep,
                        "recovery point names must be unique, bounded, and non-empty",
                    ));
                }
                Ok((
                    schema.clone(),
                    CompiledStepKind::RecoveryPoint { name: name.clone() },
                    BTreeSet::new(),
                    BTreeSet::new(),
                    DdlRisk::default(),
                ))
            }
        }
    }

    fn validate_definition(migration: &MigrationDefinition) -> Result<(), MigrationCompileError> {
        if migration.steps.is_empty()
            || migration.author.trim().is_empty()
            || migration.created_at.trim().is_empty()
            || !valid_fingerprint(&migration.output_fingerprint)
            || migration
                .input_fingerprints
                .values()
                .any(|value| !valid_fingerprint(value))
        {
            return Err(error(
                MigrationCompileErrorKind::InvalidGraph,
                format!("migration '{}' has incomplete metadata", migration.id),
            ));
        }
        Ok(())
    }

    fn validate_schema_provider(&self, schema: &SchemaIr) -> Result<(), MigrationCompileError> {
        if schema.dialect.family != self.dialect.family() {
            return Err(error(
                MigrationCompileErrorKind::ProviderMismatch,
                "migration schema dialect does not match the selected provider",
            ));
        }
        Ok(())
    }

    fn validate_provider_constraint(
        &self,
        constraint: &MigrationProviderConstraint,
    ) -> Result<(), MigrationCompileError> {
        if constraint.family != self.dialect.family() {
            return Err(error(
                MigrationCompileErrorKind::ProviderMismatch,
                "migration provider constraint does not match the selected dialect",
            ));
        }
        if !constraint
            .required_capabilities
            .is_subset(self.dialect.capabilities())
        {
            return Err(error(
                MigrationCompileErrorKind::CapabilityMismatch,
                "migration requires an unsupported provider capability",
            ));
        }
        if let Some(minimum) = &constraint.minimum_server_version {
            let required = Version::parse(minimum).map_err(|_| {
                error(
                    MigrationCompileErrorKind::ProviderMismatch,
                    "migration minimum server version is invalid",
                )
            })?;
            let actual = Version::parse(self.dialect.server_version()).map_err(|_| {
                error(
                    MigrationCompileErrorKind::ProviderMismatch,
                    "provider server version is invalid",
                )
            })?;
            if actual < required {
                return Err(error(
                    MigrationCompileErrorKind::ProviderMismatch,
                    "provider server version is below the migration minimum",
                ));
            }
        }
        Ok(())
    }

    fn validate_transaction_requirement(
        requirement: TransactionRequirement,
        transaction_count: u32,
        steps: &[MigrationStepDefinition],
    ) -> Result<(), MigrationCompileError> {
        match requirement {
            TransactionRequirement::Required => {
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
                if transaction_count != 1 || !enclosed {
                    return Err(error(
                        MigrationCompileErrorKind::InvalidTransaction,
                        "transaction-required migrations need one complete outer boundary",
                    ));
                }
            }
            TransactionRequirement::Forbidden if transaction_count != 0 => {
                return Err(error(
                    MigrationCompileErrorKind::InvalidTransaction,
                    "non-transactional migrations cannot declare transaction boundaries",
                ));
            }
            TransactionRequirement::Optional | TransactionRequirement::Forbidden => {}
        }
        Ok(())
    }
}

fn validate_data_analysis(
    analysis: &ProviderAnalysis,
    schema: &SchemaIr,
) -> Result<(), MigrationCompileError> {
    if analysis.normalized_statement.trim().is_empty()
        || !analysis.result_fields.is_empty()
        || !matches!(analysis.cardinality, Cardinality::Empty | Cardinality::ZERO)
        || !matches!(
            analysis.effects.effect,
            QueryEffect::Write | QueryEffect::ReadWrite
        )
    {
        return Err(error(
            MigrationCompileErrorKind::InvalidStep,
            "typed SQL data steps require a row-free write analysis",
        ));
    }
    validate_objects(
        schema,
        analysis
            .effects
            .referenced_objects
            .iter()
            .chain(&analysis.effects.affected_objects),
    )
}

fn validate_callback(
    callback: &DataCallbackContract,
    schema: &SchemaIr,
) -> Result<(), MigrationCompileError> {
    if callback.symbol.trim().is_empty()
        || !callback.is_async
        || !callback.returns_result
        || !callback.nonescaping
        || callback.affected_objects.is_empty()
    {
        return Err(error(
            MigrationCompileErrorKind::InvalidDataCallback,
            "data callbacks must be async, nonescaping, Result-returning writes",
        ));
    }
    validate_objects(
        schema,
        callback
            .referenced_objects
            .iter()
            .chain(&callback.affected_objects),
    )
}

fn validate_assertion(
    analysis: &ProviderAnalysis,
    schema: &SchemaIr,
) -> Result<(), MigrationCompileError> {
    let valid_field = analysis.result_fields.as_slice();
    if !analysis.parameters.is_empty()
        || valid_field.len() != 1
        || valid_field[0].name != "valid"
        || valid_field[0].sifr_type != SifrType::Bool
        || valid_field[0].nullability != Nullability::NonNull
        || analysis.effects.effect != QueryEffect::Read
        || !analysis.effects.affected_objects.is_empty()
    {
        return Err(error(
            MigrationCompileErrorKind::InvalidAssertion,
            "migration assertions require one non-null Boolean field named 'valid'",
        ));
    }
    validate_objects(schema, analysis.effects.referenced_objects.iter())
}

fn validate_backfill(
    backfill: &BackfillContract,
    schema: &SchemaIr,
) -> Result<(), MigrationCompileError> {
    validate_data_analysis(&backfill.analysis, schema).map_err(|error| {
        MigrationCompileError::new(MigrationCompileErrorKind::InvalidBackfill, error.message)
    })?;
    if backfill.maximum_batch_rows == 0 {
        return Err(error(
            MigrationCompileErrorKind::InvalidBackfill,
            "backfill batches require a positive row bound",
        ));
    }
    match &backfill.replay {
        ReplayPolicy::Never => {}
        ReplayPolicy::Idempotent { progress_key } => {
            if progress_key.is_empty() {
                return Err(error(
                    MigrationCompileErrorKind::InvalidBackfill,
                    "resumable backfills require a progress key",
                ));
            }
            validate_objects(schema, progress_key.iter())?;
        }
    }
    Ok(())
}

fn validate_objects<'a>(
    schema: &SchemaIr,
    objects: impl IntoIterator<Item = &'a ObjectId>,
) -> Result<(), MigrationCompileError> {
    for object in objects {
        if !schema.objects.contains_key(object) {
            return Err(error(
                MigrationCompileErrorKind::UnknownSchemaObject,
                format!("migration step references unavailable schema object '{object}'"),
            ));
        }
    }
    Ok(())
}

fn schema_change_sets(
    input: &SchemaIr,
    output: &SchemaIr,
) -> (BTreeSet<ObjectId>, BTreeSet<ObjectId>) {
    let diff = semantic_diff(input, output);
    let affected = diff
        .objects
        .iter()
        .map(|change| change.identity.clone())
        .collect::<BTreeSet<_>>();
    let referenced = diff
        .objects
        .iter()
        .flat_map(|change| change.before.iter().chain(&change.after))
        .flat_map(|object| object.dependencies.iter().cloned())
        .collect();
    (referenced, affected)
}

fn data_rewrite_risk<'a>(objects: impl IntoIterator<Item = &'a ObjectId>) -> DdlRisk {
    DdlRisk {
        lock_risks: BTreeSet::new(),
        data_rewrites: objects.into_iter().map(ToString::to_string).collect(),
    }
}

fn state_identity(
    migration: &MigrationNodeId,
    parent: &MigrationNodeId,
    index: usize,
    fingerprint: &str,
    rollback: bool,
) -> MigrationStateIdentity {
    let direction = if rollback { "reverse" } else { "forward" };
    let short = fingerprint.get(..16).unwrap_or(fingerprint);
    MigrationStateIdentity::new(format!(
        "sifr.sql.migration.state.{migration}.{parent}.{direction}.{index}.{short}"
    ))
}

fn fingerprint(schema: &SchemaIr) -> Result<String, MigrationCompileError> {
    schema_fingerprint(schema)
        .map(|value| value.as_str().to_string())
        .map_err(|error| {
            MigrationCompileError::new(MigrationCompileErrorKind::Serialization, error.to_string())
        })
}

fn checksum(value: &impl Serialize) -> Result<String, MigrationCompileError> {
    let encoded = serde_json::to_vec(value).map_err(|error| {
        MigrationCompileError::new(
            MigrationCompileErrorKind::Serialization,
            format!("cannot serialize migration checksum input: {error}"),
        )
    })?;
    Ok(hex(&Sha256::digest(encoded)))
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn error(kind: MigrationCompileErrorKind, message: impl Into<String>) -> MigrationCompileError {
    MigrationCompileError::new(kind, message)
}
