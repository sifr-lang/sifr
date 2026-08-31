use crate::sql_profiles::resolve_provider_identity;
use crate::{
    PackageDiagnostic, SifrPackageGraph, SifrPackageId, SqlRequirementConfig,
    SqlRequirementProviderConfig,
};
use sifr_sql_contract::{
    ProviderIdentity, ProviderSchemaRequirement, SchemaContractError, SchemaContractErrorKind,
    SchemaDocumentKind, SchemaIr, SchemaRequirementIdentity, SchemaSourceInput,
    build_provider_schema_requirement, project_provider_requirement_schema,
    schema_source_fingerprint,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSqlRequirementProvider {
    pub family: String,
    pub config: SqlRequirementProviderConfig,
    pub provider: ProviderIdentity,
    pub source_path: PathBuf,
}

impl ResolvedSqlRequirementProvider {
    pub fn load_source(&self) -> Result<SchemaSourceInput, SchemaContractError> {
        let document = normalized_relative_path(&self.config.source)?;
        let contents = std::fs::read(&self.source_path).map_err(|error| {
            SchemaContractError::new(
                SchemaContractErrorKind::InvalidProfile,
                format!("cannot read checked-in requirement DDL '{document}': {error}"),
            )
        })?;
        Ok(SchemaSourceInput {
            document,
            kind: SchemaDocumentKind::SqlDdl,
            fingerprint: schema_source_fingerprint(&contents),
            contents,
        })
    }

    pub fn build_artifact(
        &self,
        identity: SchemaRequirementIdentity,
        required_capabilities: BTreeSet<String>,
        source: &SchemaSourceInput,
        normalized: &SchemaIr,
        provider_capabilities: &BTreeSet<String>,
    ) -> Result<ProviderSchemaRequirement, SchemaContractError> {
        if normalized.provider != self.provider
            || normalized.dialect.family != self.family
            || normalized.dialect.server_version != self.config.server_version
            || normalized.dialect.features != self.config.extensions
            || normalized.dialect.modes
                != sifr_sql_contract::dialect_modes_for_session(
                    &normalized.dialect.family,
                    &sifr_sql_contract::SessionContract {
                        sql_modes: self.config.sql_modes.clone(),
                        collation: self.config.collation.clone(),
                        character_set: self.config.character_set.clone(),
                        ..sifr_sql_contract::SessionContract::default()
                    },
                )
            || source.kind != SchemaDocumentKind::SqlDdl
            || source.document != normalized_relative_path(&self.config.source)?
        {
            return Err(SchemaContractError::new(
                SchemaContractErrorKind::InvalidProfile,
                "normalized schema does not match the declared requirement provider artifact",
            ));
        }
        let projected =
            project_provider_requirement_schema(normalized, &source.document).map_err(|error| {
                SchemaContractError::new(SchemaContractErrorKind::InvalidSchema, error.message)
            })?;
        build_provider_schema_requirement(
            identity,
            source.document.clone(),
            source.fingerprint.clone(),
            &projected,
            required_capabilities,
            provider_capabilities,
        )
        .map_err(|error| {
            SchemaContractError::new(SchemaContractErrorKind::InvalidSchema, error.message)
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSqlRequirement {
    pub owner_package_id: SifrPackageId,
    pub name: String,
    pub config: SqlRequirementConfig,
    pub providers: BTreeMap<String, ResolvedSqlRequirementProvider>,
}

impl ResolvedSqlRequirement {
    pub fn identity(&self) -> Result<SchemaRequirementIdentity, SchemaContractError> {
        SchemaRequirementIdentity::new(self.owner_package_id.0.clone(), self.name.clone()).map_err(
            |error| SchemaContractError::new(SchemaContractErrorKind::InvalidSchema, error.message),
        )
    }
}

pub fn resolve_sql_requirements(
    graph: &SifrPackageGraph,
    root_package_id: &SifrPackageId,
) -> Result<BTreeMap<String, ResolvedSqlRequirement>, Vec<PackageDiagnostic>> {
    if !graph.packages.contains_key(root_package_id) {
        return Err(resolve_sql_profiles_for_missing_root(
            graph,
            root_package_id,
        ));
    }
    let mut output = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for owner_id in reachable_packages(graph, root_package_id) {
        let Some(owner) = graph.packages.get(&owner_id) else {
            continue;
        };
        for (name, config) in &owner.manifest.sql.requirements {
            let mut providers = BTreeMap::new();
            for (family, provider_config) in &config.providers {
                let key = format!("sql.requirements.{name}.providers.{family}.provider");
                let provider = match resolve_provider_identity(
                    graph,
                    &owner_id,
                    &provider_config.provider,
                    key,
                ) {
                    Ok(provider) => provider,
                    Err(diagnostic) => {
                        diagnostics.push(diagnostic);
                        continue;
                    }
                };
                providers.insert(
                    family.clone(),
                    ResolvedSqlRequirementProvider {
                        family: family.clone(),
                        config: provider_config.clone(),
                        provider,
                        source_path: owner.package_root.join(&provider_config.source),
                    },
                );
            }
            if providers.len() != config.providers.len() {
                continue;
            }
            let canonical = format!("{}::{name}", owner_id.0);
            output.insert(
                canonical,
                ResolvedSqlRequirement {
                    owner_package_id: owner_id.clone(),
                    name: name.clone(),
                    config: config.clone(),
                    providers,
                },
            );
        }
    }
    if diagnostics.is_empty() {
        Ok(output)
    } else {
        Err(diagnostics)
    }
}

fn reachable_packages(graph: &SifrPackageGraph, root: &SifrPackageId) -> BTreeSet<SifrPackageId> {
    let mut queue = VecDeque::from([root.clone()]);
    let mut visited = BTreeSet::new();
    while let Some(package) = queue.pop_front() {
        if !visited.insert(package.clone()) {
            continue;
        }
        if let Some(dependencies) = graph.cargo_edges.get(&package) {
            queue.extend(dependencies.iter().cloned());
        }
    }
    visited
}

fn resolve_sql_profiles_for_missing_root(
    graph: &SifrPackageGraph,
    root: &SifrPackageId,
) -> Vec<PackageDiagnostic> {
    crate::resolve_sql_profiles(graph, root)
        .err()
        .unwrap_or_default()
}

fn normalized_relative_path(path: &std::path::Path) -> Result<String, SchemaContractError> {
    let value = path.to_str().ok_or_else(|| {
        SchemaContractError::new(
            SchemaContractErrorKind::InvalidProfile,
            "requirement source path must be valid UTF-8",
        )
    })?;
    Ok(value.replace('\\', "/"))
}
