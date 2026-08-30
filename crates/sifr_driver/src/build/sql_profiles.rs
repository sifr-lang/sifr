use crate::diagnostics::{RenderedDiagnostic, diagnostic_with_code, render_package_diagnostic};
use sifr_compiler_component::{
    COMPONENT_PROTOCOL_MAJOR, ComponentError, ComponentHost, ComponentHostLimits,
    ComponentRequirement, ResolvedComponent,
};
use sifr_package::{
    SifrPackageGraph, SifrPackageId, compiler_component_registrations, resolve_package_component,
    resolve_sql_profiles,
};
use sifr_sql_contract::{
    ProfileModuleRegistry, generate_profile_module, normalized_schema_from_response,
    schema_context_artifact, schema_normalization_request,
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
}

impl PreparedSqlProfiles {
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
    if resolved.is_empty() {
        return Ok(PreparedSqlProfiles::default());
    }
    let registrations = compiler_component_registrations(graph).map_err(component_diagnostics)?;
    let mut host =
        ComponentHost::new(ComponentHostLimits::default(), None).map_err(component_diagnostics)?;
    let mut profiles = BTreeMap::new();
    let mut registry = ProfileModuleRegistry::default();
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
            &profile.config.session.sql_modes,
            &profile.config.extensions,
            &sources,
        )
        .map_err(schema_diagnostics)?;
        let run = host
            .analyze(&component.registration, &component.bytes, &request)
            .map_err(component_diagnostics)?;
        let schema =
            normalized_schema_from_response(profile.provider.clone(), &sources, &run.response)
                .map_err(schema_diagnostics)?;
        let authority = profile
            .build_authority(schema, &sources)
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
    let compiler = sifr_frontend::SqlQueryCompiler::new(&registry);
    for name in profiles.keys() {
        compiler.profile(name).map_err(|error| {
            let code = error.diagnostic_code();
            vec![diagnostic_with_code(error.message, code)]
        })?;
    }
    Ok(PreparedSqlProfiles { profiles, registry })
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
