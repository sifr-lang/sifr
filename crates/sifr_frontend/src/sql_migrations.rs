use sifr_ir::{
    HirExpr, HirSqlMigrationGraph, HirSqlMigrationStep, HirSqlMigrationStepKind, HirStmt,
};
use sifr_sql_contract::{
    CompiledMigrationGraph, CompiledStepKind, MigrationCompileError, MigrationCompileErrorKind,
    MigrationNodeId, MigrationSourceDeclaration, MigrationSourceStep, MigrationSourceStepKind,
};
use sifr_type_system::Type;
use std::collections::BTreeMap;

/// Extract checked migration declarations from ordinary lowered Sifr source.
/// The provider compiler remains the authority for SQL analysis and schema
/// reflection; this layer owns declaration identity, affine source order, and
/// exact executable template text.
pub fn sql_migration_declarations(
    module: &sifr_ir::HirModule,
) -> Result<Vec<MigrationSourceDeclaration>, MigrationCompileError> {
    let mut declarations = BTreeMap::new();
    let mut rollbacks = BTreeMap::new();
    for function in &module.functions {
        if let Some(metadata) = function
            .decorators
            .iter()
            .find(|decorator| decorator.starts_with("sifr.sql.migration:"))
        {
            validate_plan_function(function, "@migration")?;
            let (id, parents, author, created_at) = parse_migration_metadata(metadata)?;
            let steps = source_steps(function, &id, "forward")?;
            let declaration = MigrationSourceDeclaration {
                id: id.clone(),
                parents,
                function: function.name.clone(),
                author,
                created_at,
                steps,
                rollback: None,
            };
            if declarations.insert(id, declaration).is_some() {
                return Err(invalid("migration identity is declared more than once"));
            }
            continue;
        }
        if let Some(metadata) = function
            .decorators
            .iter()
            .find(|decorator| decorator.starts_with("sifr.sql.rollback:"))
        {
            validate_plan_function(function, "@rollback")?;
            let id = parse_rollback_metadata(metadata)?;
            let steps = source_steps(function, &id, "rollback")?;
            if rollbacks.insert(id, steps).is_some() {
                return Err(invalid("migration rollback is declared more than once"));
            }
        }
    }
    for (id, steps) in rollbacks {
        let declaration = declarations
            .get_mut(&id)
            .ok_or_else(|| invalid(format!("rollback names unknown migration '{id}'")))?;
        declaration.rollback = Some(steps);
    }
    Ok(declarations.into_values().collect())
}

fn validate_plan_function(
    function: &sifr_ir::HirFunction,
    decorator: &str,
) -> Result<(), MigrationCompileError> {
    if function.params.len() != 1 || !is_migration_plan(&function.params[0].ty) {
        return Err(invalid(format!(
            "{decorator} functions require exactly one MigrationPlan parameter"
        )));
    }
    Ok(())
}

fn source_steps(
    function: &sifr_ir::HirFunction,
    id: &MigrationNodeId,
    direction: &str,
) -> Result<Vec<MigrationSourceStep>, MigrationCompileError> {
    let mut steps = Vec::new();
    for statement in &function.body {
        match statement {
            HirStmt::Let { value, .. }
            | HirStmt::Assign { value, .. }
            | HirStmt::Return { value: Some(value) } => {
                collect_source_steps(value, id, direction, &mut steps)?;
            }
            HirStmt::Return { value: None } | HirStmt::Pass => {}
            _ => {
                return Err(invalid(
                    "migration bodies accept only ordered plan steps and a final plan return",
                ));
            }
        }
    }
    if steps.is_empty() {
        return Err(invalid("migration declaration has no checked steps"));
    }
    Ok(steps)
}

fn parse_migration_metadata(
    metadata: &str,
) -> Result<(MigrationNodeId, Vec<MigrationNodeId>, String, String), MigrationCompileError> {
    let payload = metadata
        .strip_prefix("sifr.sql.migration:")
        .ok_or_else(|| invalid("migration metadata identity is invalid"))?;
    let value = serde_json::from_str::<serde_json::Value>(payload)
        .map_err(|_| invalid("migration metadata is not valid JSON"))?;
    let object = value
        .as_object()
        .ok_or_else(|| invalid("migration metadata must be one JSON object"))?;
    let expected = ["author", "created_at", "id", "parents"]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    if object
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>()
        != expected
    {
        return Err(invalid(
            "migration metadata contains an unknown or missing field",
        ));
    }
    let id = MigrationNodeId::new(
        object
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| invalid("migration metadata has no identity"))?,
    )?;
    let parents = object
        .get("parents")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| invalid("migration metadata has no parents"))?
        .iter()
        .map(|parent| {
            parent
                .as_str()
                .ok_or_else(|| invalid("migration parent identity must be text"))
                .and_then(MigrationNodeId::new)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if parents.is_empty() {
        return Err(invalid("migration declaration needs at least one parent"));
    }
    let author = required_metadata_text(object, "author")?;
    let created_at = required_metadata_text(object, "created_at")?;
    Ok((id, parents, author, created_at))
}

fn parse_rollback_metadata(metadata: &str) -> Result<MigrationNodeId, MigrationCompileError> {
    let payload = metadata
        .strip_prefix("sifr.sql.rollback:")
        .ok_or_else(|| invalid("rollback metadata identity is invalid"))?;
    let value = serde_json::from_str::<serde_json::Value>(payload)
        .map_err(|_| invalid("rollback metadata is not valid JSON"))?;
    let object = value
        .as_object()
        .ok_or_else(|| invalid("rollback metadata must be one JSON object"))?;
    if object.len() != 1 || !object.contains_key("of") {
        return Err(invalid(
            "rollback metadata contains an unknown or missing field",
        ));
    }
    MigrationNodeId::new(
        object
            .get("of")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| invalid("rollback metadata has no migration identity"))?,
    )
}

fn required_metadata_text(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<String, MigrationCompileError> {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| invalid(format!("migration metadata has no {field}")))
}

fn collect_source_steps(
    expression: &HirExpr,
    migration: &MigrationNodeId,
    direction: &str,
    output: &mut Vec<MigrationSourceStep>,
) -> Result<(), MigrationCompileError> {
    let HirExpr::MethodCall {
        object,
        method,
        args,
        ..
    } = expression
    else {
        return Ok(());
    };
    if matches!(object.as_ref(), HirExpr::MethodCall { .. }) {
        collect_source_steps(object, migration, direction, output)?;
    }
    let kind = match method.as_str() {
        "ddl" => MigrationSourceStepKind::Ddl {
            statement: static_template_argument(args, method)?,
        },
        "sql_step" => MigrationSourceStepKind::SqlData {
            statement: static_template_argument(args, method)?,
        },
        "assert_sql" => MigrationSourceStepKind::Assertion {
            statement: static_template_argument(args, method)?,
        },
        "recovery_point" => {
            let [HirExpr::StringLiteral(name)] = args.as_slice() else {
                return Err(invalid("recovery_point requires one static string name"));
            };
            MigrationSourceStepKind::RecoveryPoint { name: name.clone() }
        }
        "begin" if args.is_empty() => MigrationSourceStepKind::Begin,
        "commit" if args.is_empty() => MigrationSourceStepKind::Commit,
        _ => return Ok(()),
    };
    let index = output.len();
    output.push(MigrationSourceStep {
        id: MigrationNodeId::new(format!(
            "{}.{}.{}.{}",
            migration.as_str(),
            direction,
            index,
            method
        ))?,
        kind,
    });
    Ok(())
}

fn static_template_argument(
    args: &[HirExpr],
    method: &str,
) -> Result<String, MigrationCompileError> {
    let [HirExpr::TemplateString(template)] = args else {
        return Err(invalid(format!(
            "migration {method} requires one typed template"
        )));
    };
    if !template.interpolations.is_empty() {
        return Err(invalid(format!(
            "migration {method} SQL must be static at graph-build time"
        )));
    }
    let statement = template
        .segments
        .iter()
        .map(|segment| segment.value.as_str())
        .collect::<String>();
    if statement.trim().is_empty() {
        return Err(invalid(format!("migration {method} SQL is empty")));
    }
    Ok(statement)
}

fn is_migration_plan(ty: &Type) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::Class {
            identity: Some(identity),
            ..
        } if identity == "sifr.sql.migration.MigrationPlan"
    )
}

pub fn lower_migration_graph(
    graph: &CompiledMigrationGraph,
) -> Result<HirSqlMigrationGraph, MigrationCompileError> {
    let mut steps = Vec::new();
    for migration_id in &graph.topological_order {
        let migration = graph
            .migrations
            .get(migration_id)
            .ok_or_else(|| invalid("compiled migration is missing"))?;
        for path in migration.paths.values() {
            for step in &path.steps {
                if step.input_state.as_str().is_empty() || step.output_state.as_str().is_empty() {
                    return Err(invalid(
                        "compiled migration step has an empty nominal state",
                    ));
                }
                let input_state = nominal_type(step.input_state.as_str(), Vec::new());
                let output_state = nominal_type(step.output_state.as_str(), Vec::new());
                let callback_db_type =
                    matches!(step.kind, CompiledStepKind::SifrData { .. }).then(|| {
                        nominal_type("sifr.sql.migration.MigrationDb", vec![input_state.clone()])
                    });
                steps.push(HirSqlMigrationStep {
                    migration_identity: migration.id.to_string(),
                    parent_identity: path.parent.to_string(),
                    step_identity: step.id.to_string(),
                    input_state_identity: step.input_state.as_str().to_string(),
                    output_state_identity: step.output_state.as_str().to_string(),
                    input_plan_type: nominal_type(
                        "sifr.sql.migration.MigrationPlan",
                        vec![input_state],
                    ),
                    output_plan_type: nominal_type(
                        "sifr.sql.migration.MigrationPlan",
                        vec![output_state],
                    ),
                    callback_db_type,
                    referenced_objects: step
                        .referenced_objects
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    affected_objects: step
                        .affected_objects
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    kind: lower_kind(&step.kind),
                });
            }
        }
    }
    Ok(HirSqlMigrationGraph {
        provider_family: graph.provider_family.clone(),
        head: graph.head.to_string(),
        target_fingerprint: graph.target_fingerprint.clone(),
        steps,
    })
}

fn lower_kind(kind: &CompiledStepKind) -> HirSqlMigrationStepKind {
    match kind {
        CompiledStepKind::ReflectedDdl { .. } | CompiledStepKind::DeclaredDdl { .. } => {
            HirSqlMigrationStepKind::Ddl
        }
        CompiledStepKind::SqlData { .. } => HirSqlMigrationStepKind::SqlData,
        CompiledStepKind::SifrData { .. } => HirSqlMigrationStepKind::SifrData,
        CompiledStepKind::Assertion { .. } => HirSqlMigrationStepKind::Assertion,
        CompiledStepKind::Backfill { .. } => HirSqlMigrationStepKind::Backfill,
        CompiledStepKind::Transaction { .. } => HirSqlMigrationStepKind::Transaction,
        CompiledStepKind::RecoveryPoint { .. } => HirSqlMigrationStepKind::RecoveryPoint,
    }
}

fn nominal_type(identity: &str, type_args: Vec<Type>) -> Type {
    Type::Class {
        identity: Some(identity.to_string()),
        type_args,
        name: identity.rsplit('.').next().unwrap_or(identity).to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    }
}

fn invalid(message: impl Into<String>) -> MigrationCompileError {
    MigrationCompileError {
        kind: MigrationCompileErrorKind::InvalidStep,
        message: message.into(),
    }
}
