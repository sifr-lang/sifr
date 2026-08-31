use crate::{
    CargoPackageId, PackageDiagnostic, SifrPackageGraph, SifrPackageId, SqlProfileConfig,
    digest_package_graph,
};
use semver::Version;
use sifr_sql_contract::{
    ProfileAuthority, ProviderIdentity, SchemaContractError, SchemaContractErrorKind, SchemaIr,
    SchemaProfile, SchemaSourceInput, build_profile_authority, schema_source_fingerprint,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSqlProfile {
    pub owner_package_id: SifrPackageId,
    pub profile_name: String,
    pub config: SqlProfileConfig,
    pub provider: ProviderIdentity,
    pub schema_sources: Vec<PathBuf>,
}

impl ResolvedSqlProfile {
    pub fn load_schema_sources(&self) -> Result<Vec<SchemaSourceInput>, SchemaContractError> {
        self.config
            .sources
            .iter()
            .zip(&self.schema_sources)
            .map(|(relative, absolute)| {
                let document = normalized_relative_path(relative)?;
                let contents = std::fs::read(absolute).map_err(|error| {
                    SchemaContractError::new(
                        SchemaContractErrorKind::InvalidProfile,
                        format!("cannot read checked-in schema source '{document}': {error}"),
                    )
                })?;
                Ok(SchemaSourceInput {
                    document,
                    kind: self.config.source_kind.into(),
                    fingerprint: schema_source_fingerprint(&contents),
                    contents,
                })
            })
            .collect()
    }

    pub fn build_authority(
        &self,
        schema: SchemaIr,
        sources: &[SchemaSourceInput],
        capabilities: BTreeSet<String>,
    ) -> Result<ProfileAuthority, SchemaContractError> {
        if schema.provider != self.provider {
            return Err(SchemaContractError::new(
                SchemaContractErrorKind::InvalidProvider,
                "normalized schema provider does not match the package-resolved profile provider",
            ));
        }
        if schema.dialect.server_version != self.config.server_version
            || schema.dialect.features != self.config.extensions
            || schema.dialect.modes
                != sifr_sql_contract::dialect_modes_for_session(
                    &schema.dialect.family,
                    &self.config.session,
                )
        {
            return Err(SchemaContractError::new(
                SchemaContractErrorKind::InvalidProfile,
                "normalized schema dialect inputs do not match the profile configuration",
            ));
        }
        let expected_sources = self
            .config
            .sources
            .iter()
            .map(|path| normalized_relative_path(path))
            .collect::<Result<BTreeSet<_>, _>>()?;
        let observed_sources = sources
            .iter()
            .map(|source| source.document.clone())
            .collect::<BTreeSet<_>>();
        if expected_sources != observed_sources || sources.len() != expected_sources.len() {
            return Err(SchemaContractError::new(
                SchemaContractErrorKind::InvalidProfile,
                "loaded schema sources do not match the package profile",
            ));
        }
        let source_fingerprints = sources
            .iter()
            .map(|source| (source.document.clone(), source.fingerprint.clone()))
            .collect::<BTreeMap<_, _>>();
        build_profile_authority(SchemaProfile {
            package_id: self.owner_package_id.0.clone(),
            name: self.profile_name.clone(),
            source_files: expected_sources,
            source_fingerprints,
            evidence: self.config.evidence,
            strictness: self.config.strictness,
            pooling: self.config.pooling,
            session: self.config.session.clone(),
            accepted_signers: self.config.accepted_signers.clone(),
            capabilities,
            schema,
        })
    }
}

impl From<crate::SchemaSourceKind> for sifr_sql_contract::SchemaDocumentKind {
    fn from(value: crate::SchemaSourceKind) -> Self {
        match value {
            crate::SchemaSourceKind::SqlDdl => Self::SqlDdl,
            crate::SchemaSourceKind::ProviderMetadata => Self::ProviderMetadata,
            crate::SchemaSourceKind::GeneratedDefinitions => Self::GeneratedDefinitions,
        }
    }
}

fn normalized_relative_path(path: &std::path::Path) -> Result<String, SchemaContractError> {
    let value = path.to_str().ok_or_else(|| {
        SchemaContractError::new(
            SchemaContractErrorKind::InvalidProfile,
            "schema source path must be valid UTF-8",
        )
    })?;
    Ok(value.replace('\\', "/"))
}

pub fn resolve_sql_profiles(
    graph: &SifrPackageGraph,
    owner_package_id: &SifrPackageId,
) -> Result<BTreeMap<String, ResolvedSqlProfile>, Vec<PackageDiagnostic>> {
    let Some(owner) = graph.packages.get(owner_package_id) else {
        return Err(vec![profile_error(
            &CargoPackageId(owner_package_id.0.clone()),
            PathBuf::from("sifr.toml"),
            "sql.profiles",
            "profile owner package is not in the resolved graph",
        )]);
    };
    let mut output = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for (name, config) in &owner.manifest.sql.profiles {
        let provider = match resolve_provider_identity(
            graph,
            owner_package_id,
            &config.provider,
            format!("sql.profiles.{name}.provider"),
        ) {
            Ok(provider) => provider,
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                continue;
            }
        };
        output.insert(
            name.clone(),
            ResolvedSqlProfile {
                owner_package_id: owner_package_id.clone(),
                profile_name: name.clone(),
                config: config.clone(),
                provider,
                schema_sources: config
                    .sources
                    .iter()
                    .map(|path| owner.package_root.join(path))
                    .collect(),
            },
        );
    }
    if diagnostics.is_empty() {
        Ok(output)
    } else {
        Err(diagnostics)
    }
}

pub(crate) fn resolve_provider_identity(
    graph: &SifrPackageGraph,
    owner_package_id: &SifrPackageId,
    alias: &str,
    key: impl Into<String>,
) -> Result<ProviderIdentity, PackageDiagnostic> {
    let key = key.into();
    let owner = graph.packages.get(owner_package_id).ok_or_else(|| {
        profile_error(
            &CargoPackageId(owner_package_id.0.clone()),
            PathBuf::from("sifr.toml"),
            &key,
            "provider owner package is not in the resolved graph",
        )
    })?;
    let candidates = graph
        .direct_dependency_scopes
        .get(owner_package_id)
        .into_iter()
        .flat_map(|scope| scope.imports.values())
        .filter(|import| import.dependency_name == alias || import.import_root.0 == alias)
        .map(|import| import.package_id.clone())
        .collect::<BTreeSet<_>>();
    if candidates.len() != 1 {
        return Err(profile_error(
            &owner.cargo_package_id,
            owner.sifr_manifest.clone(),
            key,
            if candidates.is_empty() {
                format!("provider '{alias}' is not one exact direct Sifr dependency")
            } else {
                format!("provider '{alias}' resolves to more than one package identity")
            },
        ));
    }
    let provider_id = candidates.into_iter().next().ok_or_else(|| {
        profile_error(
            &owner.cargo_package_id,
            owner.sifr_manifest.clone(),
            &key,
            "resolved provider identity is missing",
        )
    })?;
    let provider_package = graph.packages.get(&provider_id).ok_or_else(|| {
        profile_error(
            &owner.cargo_package_id,
            owner.sifr_manifest.clone(),
            &key,
            "resolved provider package metadata is missing",
        )
    })?;
    let compiler_components = provider_package
        .manifest
        .compiler_components
        .iter()
        .map(|(name, component)| {
            (
                format!("{name}@{}", component.version),
                component.sha256.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if compiler_components.is_empty() {
        return Err(profile_error(
            &owner.cargo_package_id,
            owner.sifr_manifest.clone(),
            key,
            "SQL provider package has no compiler component",
        ));
    }
    let package_version = Version::parse(&provider_package.cargo_version).map_err(|error| {
        profile_error(
            &owner.cargo_package_id,
            owner.sifr_manifest.clone(),
            &key,
            format!("provider package version is not exact semantic versioning: {error}"),
        )
    })?;
    let graph_digest = digest_package_graph(graph);
    Ok(ProviderIdentity {
        package_id: provider_package.package_id.0.clone(),
        package_version,
        package_source: provider_package
            .cargo_source
            .clone()
            .unwrap_or_else(|| "path".to_string()),
        package_graph_digest: format!("{}:{}", graph_digest.algorithm, graph_digest.hex),
        compiler_components,
    })
}

fn profile_error(
    package_id: &CargoPackageId,
    manifest_path: PathBuf,
    key: impl Into<String>,
    reason: impl Into<String>,
) -> PackageDiagnostic {
    PackageDiagnostic::invalid_sifr_manifest(package_id, manifest_path, key, reason)
}
