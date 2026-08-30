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
    ) -> Result<ProfileAuthority, SchemaContractError> {
        if schema.provider != self.provider {
            return Err(SchemaContractError::new(
                SchemaContractErrorKind::InvalidProvider,
                "normalized schema provider does not match the package-resolved profile provider",
            ));
        }
        if schema.dialect.server_version != self.config.server_version
            || schema.dialect.features != self.config.extensions
            || schema.dialect.modes != self.config.session.sql_modes
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
    let graph_digest = digest_package_graph(graph);
    let scope = graph.direct_dependency_scopes.get(owner_package_id);
    let mut output = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for (name, config) in &owner.manifest.sql.profiles {
        let candidates = scope
            .into_iter()
            .flat_map(|scope| scope.imports.values())
            .filter(|import| {
                import.dependency_name == config.provider || import.import_root.0 == config.provider
            })
            .map(|import| import.package_id.clone())
            .collect::<BTreeSet<_>>();
        if candidates.len() != 1 {
            diagnostics.push(profile_error(
                &owner.cargo_package_id,
                owner.sifr_manifest.clone(),
                format!("sql.profiles.{name}.provider"),
                if candidates.is_empty() {
                    format!(
                        "provider '{}' is not one exact direct Sifr dependency",
                        config.provider
                    )
                } else {
                    format!(
                        "provider '{}' resolves to more than one package identity",
                        config.provider
                    )
                },
            ));
            continue;
        }
        let Some(provider_id) = candidates.into_iter().next() else {
            continue;
        };
        let Some(provider_package) = graph.packages.get(&provider_id) else {
            diagnostics.push(profile_error(
                &owner.cargo_package_id,
                owner.sifr_manifest.clone(),
                format!("sql.profiles.{name}.provider"),
                "resolved provider package metadata is missing",
            ));
            continue;
        };
        let components = provider_package
            .manifest
            .compiler_components
            .iter()
            .map(|(component_name, component)| {
                (
                    format!("{component_name}@{}", component.version),
                    component.sha256.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        if components.is_empty() {
            diagnostics.push(profile_error(
                &owner.cargo_package_id,
                owner.sifr_manifest.clone(),
                format!("sql.profiles.{name}.provider"),
                "SQL provider package has no compiler component",
            ));
            continue;
        }
        let package_version = match Version::parse(&provider_package.cargo_version) {
            Ok(version) => version,
            Err(error) => {
                diagnostics.push(profile_error(
                    &owner.cargo_package_id,
                    owner.sifr_manifest.clone(),
                    format!("sql.profiles.{name}.provider"),
                    format!("provider package version is not exact semantic versioning: {error}"),
                ));
                continue;
            }
        };
        let provider = ProviderIdentity {
            package_id: provider_package.package_id.0.clone(),
            package_version,
            package_source: provider_package
                .cargo_source
                .clone()
                .unwrap_or_else(|| "path".to_string()),
            package_graph_digest: format!("{}:{}", graph_digest.algorithm, graph_digest.hex),
            compiler_components: components,
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

fn profile_error(
    package_id: &CargoPackageId,
    manifest_path: PathBuf,
    key: impl Into<String>,
    reason: impl Into<String>,
) -> PackageDiagnostic {
    PackageDiagnostic::invalid_sifr_manifest(package_id, manifest_path, key, reason)
}
