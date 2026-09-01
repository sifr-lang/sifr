use crate::{CompiledSqlQuery, QueryCompilationInput, SqlQueryCompiler};
use sifr_ir::{
    HirExpr, HirFunction, HirModule, HirStmt, visit_hir_function_exprs_mut,
    visit_hir_stmts_exprs_mut,
};
use sifr_sql_contract::{
    ProfileModuleRegistry, ProviderAnalysis, QueryContractError, SchemaRequirementError,
    SchemaRequirementErrorKind, SchemaRequirementIdentity, SchemaRequirementProof,
    SchemaRequirementRegistry,
};
use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlSchemaWitness {
    module_path: String,
    export_name: String,
    profile_identity: String,
}

impl SqlSchemaWitness {
    #[must_use]
    pub fn from_profile_export(profile: &sifr_sql_contract::RegisteredProfileModule) -> Self {
        let metadata = &profile.module().metadata.schema_witness;
        Self {
            module_path: profile.module().module_path.clone(),
            export_name: metadata.export_name.clone(),
            profile_identity: metadata.profile_identity.clone(),
        }
    }

    #[must_use]
    pub fn profile_identity(&self) -> &str {
        &self.profile_identity
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SqlSchemaWitnessUse {
    DirectNamespaceExport {
        module_path: String,
        export_name: String,
    },
    ConstrainedGenericParameter {
        requirement: SchemaRequirementIdentity,
    },
    RuntimeStorage,
    Return,
    Capture,
    Selection,
    UnconstrainedGenericParameter,
}

pub fn validate_sql_schema_witness_use(
    use_site: &SqlSchemaWitnessUse,
) -> Result<(), SchemaRequirementError> {
    match use_site {
        SqlSchemaWitnessUse::DirectNamespaceExport {
            module_path,
            export_name,
        } if module_path.starts_with("sifr.sql.schemas.") && export_name == "schema" => Ok(()),
        SqlSchemaWitnessUse::ConstrainedGenericParameter { requirement }
            if requirement.validate().is_ok() =>
        {
            Ok(())
        }
        SqlSchemaWitnessUse::DirectNamespaceExport { .. }
        | SqlSchemaWitnessUse::ConstrainedGenericParameter { .. }
        | SqlSchemaWitnessUse::RuntimeStorage
        | SqlSchemaWitnessUse::Return
        | SqlSchemaWitnessUse::Capture
        | SqlSchemaWitnessUse::Selection
        | SqlSchemaWitnessUse::UnconstrainedGenericParameter => Err(SchemaRequirementError::new(
            SchemaRequirementErrorKind::InvalidWitnessUse,
            "SqlSchema witnesses are compile-time-only namespace exports or constrained generic parameters",
        )),
    }
}

/// Validate the source-level witness surface after ordinary type lowering.
/// The caller supplies the function generic bounds recorded by the lowering
/// environment so an unconstrained `SqlSchema[T]` cannot masquerade as a
/// portable requirement.
pub fn validate_sql_schema_witness_module(
    module: &HirModule,
    bounds: Option<&HashMap<String, HashMap<String, Vec<String>>>>,
) -> Result<(), SchemaRequirementError> {
    for class in &module.classes {
        if class.fields.iter().any(|(_, ty)| contains_witness(ty)) {
            return Err(invalid_witness(
                "SqlSchema witnesses cannot be stored in class fields",
            ));
        }
    }
    if module
        .constants
        .iter()
        .any(|(_, ty, _)| contains_witness(ty))
    {
        return Err(invalid_witness(
            "SqlSchema witnesses cannot be stored in module constants",
        ));
    }
    for function in &module.functions {
        validate_witness_function(function, bounds.and_then(|items| items.get(&function.name)))?;
    }
    Ok(())
}

fn validate_witness_function(
    function: &HirFunction,
    bounds: Option<&HashMap<String, Vec<String>>>,
) -> Result<(), SchemaRequirementError> {
    if contains_witness(&function.return_type) {
        return Err(invalid_witness(
            "SqlSchema witnesses cannot be returned as runtime data",
        ));
    }
    let mut witness_params = HashSet::new();
    for parameter in &function.params {
        let Some(type_parameter) = witness_type_parameter(&parameter.ty) else {
            if contains_witness(&parameter.ty) {
                return Err(invalid_witness(
                    "SqlSchema function parameters must use one constrained generic type parameter",
                ));
            }
            continue;
        };
        let constrained = function
            .type_params
            .iter()
            .any(|name| name == type_parameter)
            && bounds
                .and_then(|items| items.get(type_parameter))
                .is_some_and(|items| {
                    items
                        .iter()
                        .any(|item| item == "Schema" || item.ends_with(".Schema"))
                });
        if !constrained {
            return Err(invalid_witness(
                "SqlSchema generic parameters require a declared schema requirement bound",
            ));
        }
        witness_params.insert(parameter.name.clone());
    }
    for statement in &function.body {
        match statement {
            HirStmt::Let { ty, .. } if contains_witness(ty) => {
                return Err(invalid_witness(
                    "SqlSchema witnesses cannot be stored in runtime locals",
                ));
            }
            HirStmt::Assign { value, .. } if contains_witness(value.ty()) => {
                return Err(invalid_witness(
                    "SqlSchema witnesses cannot be assigned as runtime values",
                ));
            }
            HirStmt::Return { value: Some(value) } if contains_witness(value.ty()) => {
                return Err(invalid_witness(
                    "SqlSchema witnesses cannot be returned as runtime data",
                ));
            }
            _ => {}
        }
    }
    let mut cloned = function.clone();
    let mut invalid_selection = false;
    let mut lambda_capture = false;
    visit_hir_function_exprs_mut(&mut cloned, &mut |expression| {
        if matches!(expression, HirExpr::IfExpr { ty, .. } if contains_witness(ty)) {
            invalid_selection = true;
        }
        if let HirExpr::Lambda { body, .. } = expression {
            let mut body = HirStmt::Expr {
                expr: body.as_ref().clone(),
            };
            visit_hir_stmts_exprs_mut(std::slice::from_mut(&mut body), &mut |nested| {
                if matches!(nested, HirExpr::Name { name, .. } if witness_params.contains(name.as_str()))
                {
                    lambda_capture = true;
                }
            });
        }
    });
    if invalid_selection {
        return Err(invalid_witness(
            "SqlSchema witnesses cannot be selected through runtime control flow",
        ));
    }
    if lambda_capture || nested_function_captures(&function.body, &witness_params) {
        return Err(invalid_witness(
            "SqlSchema witnesses cannot be captured by closures",
        ));
    }
    Ok(())
}

fn nested_function_captures(statements: &[HirStmt], witnesses: &HashSet<String>) -> bool {
    for statement in statements {
        match statement {
            HirStmt::NestedFunction { func, .. } => {
                let mut func = func.clone();
                let mut captured = false;
                visit_hir_function_exprs_mut(&mut func, &mut |expression| {
                    if matches!(expression, HirExpr::Name { name, .. } if witnesses.contains(name.as_str()))
                    {
                        captured = true;
                    }
                });
                if captured {
                    return true;
                }
            }
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                if nested_function_captures(then_body, witnesses)
                    || elif_clauses
                        .iter()
                        .any(|(_, body)| nested_function_captures(body, witnesses))
                    || else_body
                        .as_deref()
                        .is_some_and(|body| nested_function_captures(body, witnesses))
                {
                    return true;
                }
            }
            HirStmt::While {
                body, else_body, ..
            }
            | HirStmt::For {
                body, else_body, ..
            }
            | HirStmt::AsyncFor {
                body, else_body, ..
            } => {
                if nested_function_captures(body, witnesses)
                    || else_body
                        .as_deref()
                        .is_some_and(|body| nested_function_captures(body, witnesses))
                {
                    return true;
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                if nested_function_captures(body, witnesses)
                    || handlers
                        .iter()
                        .any(|handler| nested_function_captures(&handler.body, witnesses))
                {
                    return true;
                }
            }
            HirStmt::TryFinally { body, finalbody } => {
                if nested_function_captures(body, witnesses)
                    || nested_function_captures(finalbody, witnesses)
                {
                    return true;
                }
            }
            HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
                if nested_function_captures(body, witnesses) {
                    return true;
                }
            }
            HirStmt::Match { arms, .. } => {
                if arms
                    .iter()
                    .any(|arm| nested_function_captures(&arm.body, witnesses))
                {
                    return true;
                }
            }
            HirStmt::Let { .. }
            | HirStmt::Assign { .. }
            | HirStmt::AugAssign { .. }
            | HirStmt::Return { .. }
            | HirStmt::Expr { .. }
            | HirStmt::Break
            | HirStmt::Continue
            | HirStmt::TupleUnpack { .. }
            | HirStmt::StarUnpack { .. }
            | HirStmt::Pass
            | HirStmt::Assert { .. }
            | HirStmt::Raise { .. }
            | HirStmt::FieldAssign { .. }
            | HirStmt::NestedFieldAssign { .. }
            | HirStmt::SubscriptAssign { .. }
            | HirStmt::NestedSubscriptAssign { .. }
            | HirStmt::AttributeNestedSubscriptAssign { .. }
            | HirStmt::SubscriptAugAssign { .. }
            | HirStmt::AttributeAugAssign { .. }
            | HirStmt::AttributeSubscriptAssign { .. }
            | HirStmt::Delete { .. }
            | HirStmt::Yield { .. } => {}
        }
    }
    false
}

pub(crate) fn witness_type_parameter(ty: &Type) -> Option<&str> {
    let Type::Class {
        identity: Some(identity),
        type_args,
        ..
    } = ty.resolve_alias()
    else {
        return None;
    };
    if identity != "sifr.sql.SqlSchema" {
        return None;
    }
    match type_args.as_slice() {
        [Type::TypeVar(name)] => Some(name),
        _ => None,
    }
}

fn contains_witness(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::Class {
            identity: Some(identity),
            ..
        } if identity == "sifr.sql.SqlSchema" => true,
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Awaitable(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::Newtype { inner, .. } => contains_witness(inner),
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::Task(left, right)
        | Type::TaskResult(left, right)
        | Type::Coroutine(left, right)
        | Type::Select2(left, right)
        | Type::BlockingTask(left, right)
        | Type::JoinSet(left, right)
        | Type::AsyncIterator(left, right)
        | Type::AsyncGenerator(left, right) => contains_witness(left) || contains_witness(right),
        Type::Tuple(items) | Type::Union(items) | Type::Intersection(items) => {
            items.iter().any(contains_witness)
        }
        _ => false,
    }
}

fn invalid_witness(message: impl Into<String>) -> SchemaRequirementError {
    SchemaRequirementError::new(SchemaRequirementErrorKind::InvalidWitnessUse, message)
}

pub struct SchemaSpecializationInput<'a> {
    pub requirement_name: &'a str,
    pub profile_name: &'a str,
    pub witness: &'a SqlSchemaWitness,
    pub query: QueryCompilationInput<'a>,
}

#[derive(Clone, Debug)]
pub struct SpecializedSqlQuery {
    pub query: CompiledSqlQuery,
    pub proof: SchemaRequirementProof,
}

/// Provider-neutral schema specialization. The method selects one statically
/// known profile, proves its provider artifact, checks the query's complete
/// object and behavior envelope, and returns a concrete query with no witness.
pub struct SchemaPolymorphicQueryCompiler<'a> {
    profiles: &'a ProfileModuleRegistry,
    requirements: &'a SchemaRequirementRegistry,
}

impl<'a> SchemaPolymorphicQueryCompiler<'a> {
    #[must_use]
    pub fn new(
        profiles: &'a ProfileModuleRegistry,
        requirements: &'a SchemaRequirementRegistry,
    ) -> Self {
        Self {
            profiles,
            requirements,
        }
    }

    pub fn specialize(
        &self,
        input: SchemaSpecializationInput<'_>,
    ) -> Result<SpecializedSqlQuery, SchemaRequirementError> {
        if input.query.profile_name != input.profile_name {
            return Err(profile_mismatch(
                "specialization and query analysis must name the same concrete profile",
            ));
        }
        let query_compiler = SqlQueryCompiler::new(self.profiles);
        let profile = query_compiler
            .profile(input.profile_name)
            .map_err(query_error)?;
        if input.witness.profile_identity != profile.authority().nominal_identity
            || input.witness.module_path != profile.module().module_path
            || input.witness.export_name != "schema"
        {
            return Err(profile_mismatch(
                "schema witness does not belong to the proving profile namespace",
            ));
        }
        let requirement = self.requirements.requirement(input.requirement_name)?;
        let proof = requirement.prove(profile.authority())?;
        validate_query_envelope(&input.query.analysis, &proof)?;
        let query = query_compiler.compile(input.query).map_err(query_error)?;
        if query.contract.profile_identity != proof.profile_identity
            || query.contract.profile_fingerprint != proof.profile_fingerprint
            || query.contract.schema_fingerprint != proof.schema_fingerprint
        {
            return Err(profile_mismatch(
                "specialized query lost its proving profile identity",
            ));
        }
        Ok(SpecializedSqlQuery { query, proof })
    }
}

fn validate_query_envelope(
    analysis: &ProviderAnalysis,
    proof: &SchemaRequirementProof,
) -> Result<(), SchemaRequirementError> {
    let accessed_objects = &analysis.accessed_objects;
    let undeclared = accessed_objects
        .difference(&proof.declared_objects)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if !undeclared.is_empty() {
        return Err(SchemaRequirementError::new(
            SchemaRequirementErrorKind::UndeclaredObject,
            format!(
                "specialized query reaches objects absent from its schema requirement: {}",
                undeclared.join(", ")
            ),
        ));
    }
    if analysis.required_capabilities.is_empty() {
        return Err(SchemaRequirementError::new(
            SchemaRequirementErrorKind::UndeclaredBehavior,
            "portable query must declare every provider capability it uses",
        ));
    }
    let undeclared = analysis
        .required_capabilities
        .difference(&proof.required_capabilities)
        .cloned()
        .collect::<Vec<_>>();
    if !undeclared.is_empty() {
        return Err(SchemaRequirementError::new(
            SchemaRequirementErrorKind::UndeclaredBehavior,
            format!(
                "portable query uses undeclared SQL capabilities: {}",
                undeclared.join(", ")
            ),
        ));
    }
    Ok(())
}

fn query_error(error: QueryContractError) -> SchemaRequirementError {
    profile_mismatch(error.message)
}

fn profile_mismatch(message: impl Into<String>) -> SchemaRequirementError {
    SchemaRequirementError::new(
        SchemaRequirementErrorKind::ExecutionProfileMismatch,
        message,
    )
}
