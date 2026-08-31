use crate::diagnostics::{RenderedDiagnostic, diagnostic_with_code, render_package_diagnostic};
use sifr_compiler_component::{
    COMPONENT_PROTOCOL_MAJOR, ComponentError, ComponentHost, ComponentHostLimits,
    ComponentRequirement, ResolvedComponent,
};
use sifr_package::{
    SifrPackageGraph, SifrPackageId, compiler_component_registrations, resolve_package_component,
    resolve_sql_profiles, resolve_sql_requirements,
};
use sifr_sql_contract::{
    ProfileModuleRegistry, SchemaRequirement, SchemaRequirementRegistry, generate_profile_module,
    schema_context_artifact, schema_normalization_from_response, schema_normalization_request,
};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

#[derive(Clone, Debug)]
struct PreparedSqlProfile {
    context: sifr_compiler_component::ContextArtifact,
    query_component: ResolvedComponent,
}

#[derive(Clone, Debug, Default)]
pub struct PreparedSqlProfiles {
    profiles: BTreeMap<String, PreparedSqlProfile>,
    registry: ProfileModuleRegistry,
    requirements: SchemaRequirementRegistry,
    initialization_diagnostics: Vec<RenderedDiagnostic>,
}

impl PreparedSqlProfiles {
    #[must_use]
    pub fn from_initialization_failure(diagnostics: Vec<RenderedDiagnostic>) -> Self {
        Self {
            initialization_diagnostics: diagnostics,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn initialization_diagnostics(&self) -> &[RenderedDiagnostic] {
        &self.initialization_diagnostics
    }
    #[must_use]
    #[cfg(test)]
    pub(super) fn module(
        &self,
        profile: &str,
    ) -> Option<&sifr_sql_contract::GeneratedProfileModule> {
        self.registry
            .profile(profile)
            .ok()
            .map(sifr_sql_contract::RegisteredProfileModule::module)
    }

    #[must_use]
    pub fn registry(&self) -> &ProfileModuleRegistry {
        &self.registry
    }

    #[must_use]
    pub fn requirements(&self) -> &SchemaRequirementRegistry {
        &self.requirements
    }

    #[must_use]
    pub fn query_component(&self, profile: &str) -> Option<&ResolvedComponent> {
        self.profiles
            .get(profile)
            .map(|prepared| &prepared.query_component)
    }

    #[must_use]
    pub fn schema_context(
        &self,
        profile: &str,
    ) -> Option<&sifr_compiler_component::ContextArtifact> {
        self.profiles.get(profile).map(|prepared| &prepared.context)
    }

    #[must_use]
    pub fn sole_profile_name(&self) -> Option<&str> {
        (self.profiles.len() == 1)
            .then(|| self.profiles.keys().next().map(String::as_str))
            .flatten()
    }

    #[must_use]
    pub(super) fn cache_fragment(&self) -> String {
        let mut fragment = String::new();
        for (name, registered) in self.registry.entries() {
            let Some(prepared) = self.profiles.get(name) else {
                continue;
            };
            let _ = writeln!(
                fragment,
                "{name}\t{}\t{}\t{}\t{}",
                registered.authority().profile_fingerprint.as_str(),
                registered.authority().schema_fingerprint.as_str(),
                registered.module().module_path,
                prepared.context.fingerprint
            );
        }
        for (name, requirement) in self.requirements.entries() {
            for (family, artifact) in &requirement.providers {
                let _ = writeln!(
                    fragment,
                    "requirement\t{name}\t{family}\t{}",
                    artifact.artifact_fingerprint,
                );
            }
        }
        fragment
    }
}

pub fn load_sql_editor_profiles(
    workspace_root: &Path,
    entrypoint: &Path,
) -> Result<PreparedSqlProfiles, Vec<RenderedDiagnostic>> {
    if !workspace_root.join("sifr.toml").is_file() || !workspace_root.join("Cargo.toml").is_file() {
        return Ok(PreparedSqlProfiles::default());
    }
    let mut provider = sifr_frontend::DiskSourceProvider::new();
    let snapshot = sifr_package::load_package_graph_snapshot(
        workspace_root,
        sifr_package::CargoLockMode::Frozen,
        &mut provider,
    )
    .map_err(|failure| match failure.kind {
        sifr_package::PackageGraphLoadFailureKind::Package { diagnostics, .. } => diagnostics
            .into_iter()
            .map(render_package_diagnostic)
            .collect(),
        sifr_package::PackageGraphLoadFailureKind::Spawn { message } => vec![diagnostic_with_code(
            format!("cannot load the SQL editor package graph: {message}"),
            sifr_diagnostics::DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
        )],
        sifr_package::PackageGraphLoadFailureKind::Command { output, .. } => {
            vec![diagnostic_with_code(
                format!(
                    "cannot load the SQL editor package graph: {}",
                    output.trim()
                ),
                sifr_diagnostics::DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]
        }
    })?;
    let owner = snapshot
        .graph
        .packages
        .values()
        .filter(|package| {
            entrypoint.starts_with(&package.package_root)
                || workspace_root.starts_with(&package.package_root)
        })
        .max_by_key(|package| package.package_root.components().count())
        .map(|package| package.package_id.clone());
    let Some(owner) = owner else {
        return Ok(PreparedSqlProfiles::default());
    };
    prepare_sql_profiles(&snapshot.graph, &owner)
}

pub fn prepare_sql_profiles(
    graph: &SifrPackageGraph,
    owner_package_id: &SifrPackageId,
) -> Result<PreparedSqlProfiles, Vec<RenderedDiagnostic>> {
    let resolved = resolve_sql_profiles(graph, owner_package_id).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(render_package_diagnostic)
            .collect::<Vec<_>>()
    })?;
    let resolved_requirements =
        resolve_sql_requirements(graph, owner_package_id).map_err(|diagnostics| {
            diagnostics
                .into_iter()
                .map(render_package_diagnostic)
                .collect::<Vec<_>>()
        })?;
    if resolved.is_empty() && resolved_requirements.is_empty() {
        return Ok(PreparedSqlProfiles::default());
    }
    let registrations = compiler_component_registrations(graph).map_err(component_diagnostics)?;
    let mut host =
        ComponentHost::new(ComponentHostLimits::default(), None).map_err(component_diagnostics)?;
    let mut profiles = BTreeMap::new();
    let mut registry = ProfileModuleRegistry::default();
    let mut requirements = SchemaRequirementRegistry::default();
    for (name, profile) in resolved {
        let candidates = registrations
            .values()
            .filter(|component| {
                component.package_id.0 == profile.provider.package_id
                    && processor_kind(&component.registration.identity.processor) == Some("schema")
            })
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return Err(vec![diagnostic_with_code(
                format!(
                    "SQL profile '{name}' requires one exact provider processor ending in '.schema'; found {}",
                    candidates.len()
                ),
                sifr_diagnostics::DiagnosticCode::COMPONENT_REGISTRATION,
            )]);
        }
        let registration = &candidates[0].registration;
        let requirement = ComponentRequirement {
            identity: registration.identity.clone(),
            protocol_major: COMPONENT_PROTOCOL_MAJOR,
        };
        let component =
            resolve_package_component(graph, &requirement).map_err(component_diagnostics)?;
        let query_candidates = registrations
            .values()
            .filter(|candidate| {
                candidate.package_id.0 == profile.provider.package_id
                    && processor_kind(&candidate.registration.identity.processor) == Some("sql")
            })
            .collect::<Vec<_>>();
        if query_candidates.len() != 1 {
            return Err(vec![diagnostic_with_code(
                format!(
                    "SQL profile '{name}' requires one exact provider processor ending in '.sql'; found {}",
                    query_candidates.len()
                ),
                sifr_diagnostics::DiagnosticCode::COMPONENT_REGISTRATION,
            )]);
        }
        let query_registration = &query_candidates[0].registration;
        let query_component = resolve_package_component(
            graph,
            &ComponentRequirement {
                identity: query_registration.identity.clone(),
                protocol_major: COMPONENT_PROTOCOL_MAJOR,
            },
        )
        .map_err(component_diagnostics)?;
        let sources = profile.load_schema_sources().map_err(schema_diagnostics)?;
        let request = schema_normalization_request(
            &component.registration,
            env!("CARGO_PKG_VERSION"),
            &format!("{}::{name}", profile.owner_package_id.0),
            &profile.config.server_version,
            &profile.config.session,
            &profile.config.extensions,
            &sources,
        )
        .map_err(schema_diagnostics)?;
        let run = host
            .analyze(&component.registration, &component.bytes, &request)
            .map_err(component_diagnostics)?;
        let normalized =
            schema_normalization_from_response(profile.provider.clone(), &sources, &run.response)
                .map_err(schema_diagnostics)?;
        let authority = profile
            .build_authority(normalized.schema, &sources, normalized.capabilities)
            .map_err(schema_diagnostics)?;
        let module = generate_profile_module(&authority).map_err(schema_diagnostics)?;
        crate::frontend::parse_source(&module.source)?;
        let context = schema_context_artifact(&authority).map_err(schema_diagnostics)?;
        registry
            .register(authority, module)
            .map_err(schema_diagnostics)?;
        profiles.insert(
            name,
            PreparedSqlProfile {
                context,
                query_component,
            },
        );
    }
    for requirement in resolved_requirements.values() {
        let identity = requirement.identity().map_err(schema_diagnostics)?;
        let mut artifacts = Vec::new();
        for provider in requirement.providers.values() {
            let candidates = registrations
                .values()
                .filter(|component| {
                    component.package_id.0 == provider.provider.package_id
                        && processor_kind(&component.registration.identity.processor)
                            == Some("schema")
                })
                .collect::<Vec<_>>();
            if candidates.len() != 1 {
                return Err(vec![diagnostic_with_code(
                    format!(
                        "SQL requirement '{}::{}' provider '{}' needs one exact '.schema' processor; found {}",
                        requirement.owner_package_id.0,
                        requirement.name,
                        provider.family,
                        candidates.len()
                    ),
                    sifr_diagnostics::DiagnosticCode::COMPONENT_REGISTRATION,
                )]);
            }
            let registration = &candidates[0].registration;
            let component = resolve_package_component(
                graph,
                &ComponentRequirement {
                    identity: registration.identity.clone(),
                    protocol_major: COMPONENT_PROTOCOL_MAJOR,
                },
            )
            .map_err(component_diagnostics)?;
            let source = provider.load_source().map_err(schema_diagnostics)?;
            let request = schema_normalization_request(
                &component.registration,
                env!("CARGO_PKG_VERSION"),
                &format!(
                    "{}::requirements::{}::{}",
                    requirement.owner_package_id.0, requirement.name, provider.family
                ),
                &provider.config.server_version,
                &sifr_sql_contract::SessionContract {
                    sql_modes: provider.config.sql_modes.clone(),
                    collation: provider.config.collation.clone(),
                    character_set: provider.config.character_set.clone(),
                    ..sifr_sql_contract::SessionContract::default()
                },
                &provider.config.extensions,
                std::slice::from_ref(&source),
            )
            .map_err(schema_diagnostics)?;
            let run = host
                .analyze(&component.registration, &component.bytes, &request)
                .map_err(component_diagnostics)?;
            let normalized = schema_normalization_from_response(
                provider.provider.clone(),
                std::slice::from_ref(&source),
                &run.response,
            )
            .map_err(schema_diagnostics)?;
            artifacts.push(
                provider
                    .build_artifact(
                        identity.clone(),
                        requirement.config.capabilities.clone(),
                        &source,
                        &normalized.schema,
                        &normalized.capabilities,
                    )
                    .map_err(schema_diagnostics)?,
            );
        }
        let requirement =
            SchemaRequirement::new(identity, artifacts).map_err(requirement_diagnostics)?;
        requirements
            .register(requirement)
            .map_err(requirement_diagnostics)?;
    }
    let compiler = sifr_frontend::SqlQueryCompiler::new(&registry);
    for name in profiles.keys() {
        compiler.profile(name).map_err(|error| {
            let code = error.diagnostic_code();
            vec![diagnostic_with_code(error.message, code)]
        })?;
    }
    Ok(PreparedSqlProfiles {
        profiles,
        registry,
        requirements,
        initialization_diagnostics: Vec::new(),
    })
}

fn requirement_diagnostics(
    error: sifr_sql_contract::SchemaRequirementError,
) -> Vec<RenderedDiagnostic> {
    vec![diagnostic_with_code(
        error.message,
        sifr_diagnostics::DiagnosticCode::COMPONENT_PROTOCOL_ENVELOPE,
    )]
}

fn processor_kind(identity: &str) -> Option<&str> {
    identity.rsplit_once('.').map(|(_, kind)| kind)
}

fn component_diagnostics(error: ComponentError) -> Vec<RenderedDiagnostic> {
    let ComponentError { kind, message } = error;
    vec![diagnostic_with_code(
        format!("{}: {message}", kind.code()),
        kind.diagnostic_code(),
    )]
}

fn schema_diagnostics(error: sifr_sql_contract::SchemaContractError) -> Vec<RenderedDiagnostic> {
    let sifr_sql_contract::SchemaContractError { message, .. } = error;
    vec![diagnostic_with_code(
        message,
        sifr_diagnostics::DiagnosticCode::COMPONENT_PROTOCOL_ENVELOPE,
    )]
}
