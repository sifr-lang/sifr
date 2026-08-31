use crate::{CompiledSqlQuery, QueryCompilationInput, SqlQueryCompiler};
use sifr_sql_contract::{
    ProfileModuleRegistry, ProviderAnalysis, QueryContractError, SchemaRequirementError,
    SchemaRequirementErrorKind, SchemaRequirementIdentity, SchemaRequirementProof,
    SchemaRequirementRegistry,
};

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
