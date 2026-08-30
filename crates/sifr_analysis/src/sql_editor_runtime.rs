use crate::{SqlAnalysisDependency, SqlIncrementalAnalysisCache};
use sifr_compiler_component::{
    AnalysisContext, COMPONENT_PROTOCOL_MAJOR, ComponentError, ComponentHost, ComponentHostLimits,
    EmbeddedAnalysisRequest, HoleDescriptor, PlanKind, SourceSpan, TemplatePart,
};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_driver::PreparedSqlProfiles;
use sifr_frontend::{
    CacheKeyContext, CacheKeyFingerprint, EmbeddedAnalysisKey, EmbeddedProviderOperationError,
    SqlEditorCatalog, SqlEditorDocumentView, TemplateSourceMapKind, run_embedded_provider_items,
};
use sifr_sql_contract::{
    ProviderAnalysis, provider_analysis_from_response, schema_object_fingerprint,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct SqlEditorRuntime {
    profiles: PreparedSqlProfiles,
    host: ComponentHost,
    cache: SqlIncrementalAnalysisCache<ProviderAnalysis>,
    cancellation: Option<Arc<AtomicBool>>,
    diagnostics: BTreeMap<String, Vec<RenderedDiagnostic>>,
    slice_keys: BTreeMap<CacheKeyFingerprint, CacheKeyFingerprint>,
}

pub(super) fn sql_editor_initialization_diagnostic(error: &ComponentError) -> RenderedDiagnostic {
    let code = sifr_diagnostics::DiagnosticCode::COMPONENT_EXECUTION;
    let message = format!("cannot initialize SQL editor component host: {error}");
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message: message.clone(),
        message_template: "{message}".to_string(),
        args: BTreeMap::from([(
            "message".to_string(),
            sifr_diagnostics::DiagnosticArg::String(message),
        )]),
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

impl SqlEditorRuntime {
    pub(super) fn new(profiles: PreparedSqlProfiles) -> Result<Self, ComponentError> {
        let limits = ComponentHostLimits {
            fuel: 100_000_000,
            ..ComponentHostLimits::default()
        };
        Ok(Self {
            profiles,
            host: ComponentHost::new(limits, None)?,
            cache: SqlIncrementalAnalysisCache::open_default(),
            cancellation: None,
            diagnostics: BTreeMap::new(),
            slice_keys: BTreeMap::new(),
        })
    }

    pub(super) fn set_cancellation(&mut self, cancellation: Option<Arc<AtomicBool>>) {
        self.cancellation = cancellation;
    }

    pub(super) fn diagnostics_for_source(&self, source: &str) -> Vec<RenderedDiagnostic> {
        self.diagnostics.get(source).cloned().unwrap_or_default()
    }

    pub(super) fn enrich(
        &mut self,
        mut documents: Vec<SqlEditorDocumentView>,
        cache_context: &CacheKeyContext,
        source_document: &str,
    ) -> Result<Vec<SqlEditorDocumentView>, SqlEditorRuntimeError> {
        self.diagnostics.remove(source_document);
        let observed = self.observed_dependencies()?;
        let invalidated = self.cache.invalidate_dependencies(&observed);
        self.slice_keys
            .retain(|_, slice_key| !invalidated.contains(slice_key));
        let mut misses = Vec::new();
        for (index, document) in documents.iter_mut().enumerate() {
            let profile_name = document
                .profile_name
                .as_deref()
                .filter(|name| !name.is_empty())
                .or_else(|| self.profiles.sole_profile_name())
                .map(str::to_string);
            let Some(profile_name) = profile_name else {
                continue;
            };
            let Ok(registered) = self.profiles.registry().profile(&profile_name) else {
                continue;
            };
            let schema = &registered.authority().profile.schema;
            *document = document
                .clone()
                .with_catalog(SqlEditorCatalog::from_schema(schema));
            let Some(component) = self.profiles.query_component(&profile_name).cloned() else {
                return Err(SqlEditorRuntimeError::Contract(format!(
                    "SQL editor profile '{profile_name}' has no query component"
                )));
            };
            let Some(context_artifact) = self.profiles.schema_context(&profile_name).cloned()
            else {
                return Err(SqlEditorRuntimeError::Contract(format!(
                    "SQL editor profile '{profile_name}' has no schema context"
                )));
            };
            let Some(request) = request_for_document(
                document,
                source_document,
                registered.authority().nominal_identity.as_str(),
                registered.authority().schema_fingerprint.as_str(),
                &registered.authority().profile,
                component.registration.clone(),
                context_artifact,
            ) else {
                continue;
            };
            let base_request = dependency_scoped_cache_request(&request, None)?;
            let base_key = EmbeddedAnalysisKey::new(&base_request, (*cache_context).clone())
                .map_err(SqlEditorRuntimeError::Component)?
                .fingerprint();
            if let Some(analysis) = self
                .slice_keys
                .get(&base_key)
                .and_then(|slice_key| self.cache.get(slice_key))
            {
                *document = document
                    .clone()
                    .with_provider_analysis(schema, analysis.as_ref());
                continue;
            }
            misses.push((index, profile_name, base_key, component, request));
        }
        let cancelled = self.cancellation.clone();
        let runs = run_embedded_provider_items(
            move |_| {
                cancelled
                    .as_ref()
                    .is_some_and(|flag| flag.load(Ordering::Acquire))
            },
            misses
                .iter()
                .map(|(_, _, _, component, request)| (component, request)),
            |(component, request)| {
                self.host
                    .analyze(&component.registration, &component.bytes, request)
            },
        )
        .map_err(SqlEditorRuntimeError::Operation)?;
        for (miss, run) in misses.into_iter().zip(runs) {
            let (index, profile_name, base_key, _, request) = miss;
            if !run.response.plan.diagnostics.is_empty() {
                self.diagnostics
                    .entry(source_document.to_string())
                    .or_default()
                    .extend(
                        run.response
                            .plan
                            .diagnostics
                            .iter()
                            .map(render_provider_diagnostic),
                    );
                continue;
            }
            let analysis = provider_analysis_from_response(&run.response)
                .map_err(|error| SqlEditorRuntimeError::Contract(error.to_string()))?;
            let dependencies = run
                .response
                .plan
                .dependencies
                .iter()
                .map(|dependency| {
                    SqlAnalysisDependency::new(
                        dependency_identity(&profile_name, &dependency.identity),
                        dependency.fingerprint.clone(),
                    )
                })
                .collect::<Vec<_>>();
            let slice_request =
                dependency_scoped_cache_request(&request, Some(&run.response.plan.dependencies))?;
            let slice_key = EmbeddedAnalysisKey::new(&slice_request, (*cache_context).clone())
                .map_err(SqlEditorRuntimeError::Component)?
                .fingerprint();
            let analysis = self
                .cache
                .insert(slice_key.clone(), analysis, dependencies)
                .map_err(|error| SqlEditorRuntimeError::Cache(error.to_string()))?;
            self.slice_keys.insert(base_key, slice_key);
            let registered = self
                .profiles
                .registry()
                .profile(&profile_name)
                .map_err(|error| SqlEditorRuntimeError::Contract(error.to_string()))?;
            documents[index] = documents[index]
                .clone()
                .with_provider_analysis(&registered.authority().profile.schema, analysis.as_ref());
        }
        Ok(documents)
    }

    fn observed_dependencies(&self) -> Result<BTreeMap<String, String>, SqlEditorRuntimeError> {
        let mut observed = BTreeMap::new();
        for (profile_name, registered) in self.profiles.registry().entries() {
            for object in registered.authority().profile.schema.objects.values() {
                observed.insert(
                    dependency_identity(profile_name, object.identity.as_str()),
                    schema_object_fingerprint(object)
                        .map_err(|error| SqlEditorRuntimeError::Contract(error.to_string()))?,
                );
            }
        }
        Ok(observed)
    }
}

fn render_provider_diagnostic(
    diagnostic: &sifr_compiler_component::EmbeddedDiagnostic,
) -> RenderedDiagnostic {
    let severity = match diagnostic.severity {
        sifr_compiler_component::DiagnosticSeverity::Error => sifr_diagnostics::Severity::Error,
        sifr_compiler_component::DiagnosticSeverity::Warning => sifr_diagnostics::Severity::Warning,
        sifr_compiler_component::DiagnosticSeverity::Note => sifr_diagnostics::Severity::Note,
    };
    let message = diagnostic.message.clone();
    let mut spans = Vec::with_capacity(1 + diagnostic.related.len());
    spans.push(render_provider_span(&diagnostic.primary, true));
    spans.extend(
        diagnostic
            .related
            .iter()
            .map(|span| render_provider_span(span, false)),
    );
    RenderedDiagnostic {
        code: diagnostic.code.clone(),
        severity,
        message: message.clone(),
        message_template: "{message}".to_string(),
        args: BTreeMap::from([(
            "message".to_string(),
            sifr_diagnostics::DiagnosticArg::String(message),
        )]),
        url: format!("https://docs.sifr-lang.org/errors/{}", diagnostic.code),
        spans,
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

fn render_provider_span(
    span: &sifr_compiler_component::SourceSpan,
    is_primary: bool,
) -> sifr_diagnostics::DiagnosticSpan {
    sifr_diagnostics::DiagnosticSpan {
        file: Some(span.document.clone()),
        byte_start: span.start,
        byte_end: span.end,
        line: None,
        column: None,
        end_line: None,
        end_column: None,
        is_primary,
        label: (!is_primary).then(|| "related SQL location".to_string()),
        lines: Vec::new(),
    }
}

fn dependency_identity(profile: &str, identity: &str) -> String {
    format!("{profile}::{identity}")
}

fn dependency_scoped_cache_request(
    request: &EmbeddedAnalysisRequest,
    dependencies: Option<&[sifr_compiler_component::DependencyDescriptor]>,
) -> Result<EmbeddedAnalysisRequest, SqlEditorRuntimeError> {
    let mut key_request = request.clone();
    key_request.context.schema_fingerprint = None;
    key_request.context.artifacts.clear();
    if let Some(dependencies) = dependencies {
        key_request.context.semantic_profile.insert(
            "schema-slice".to_string(),
            serde_json::to_string(dependencies)
                .map_err(|error| SqlEditorRuntimeError::Contract(error.to_string()))?,
        );
    }
    Ok(key_request)
}

fn request_for_document(
    document: &SqlEditorDocumentView,
    source_document: &str,
    profile_identity: &str,
    schema_fingerprint: &str,
    profile: &sifr_sql_contract::SchemaProfile,
    registration: sifr_compiler_component::ComponentRegistration,
    context_artifact: sifr_compiler_component::ContextArtifact,
) -> Option<EmbeddedAnalysisRequest> {
    let parts = template_parts(document, source_document)?;
    let holes = document
        .parameter_protocol_types
        .iter()
        .enumerate()
        .map(|(index, ty)| {
            Some(HoleDescriptor {
                index: u32::try_from(index).ok()?,
                ty: ty.clone()?,
                fragment_identity: None,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let semantic_profile = BTreeMap::from([
        (
            "server-version".to_string(),
            profile.schema.dialect.server_version.clone(),
        ),
        (
            "modes".to_string(),
            serde_json::to_string(&profile.schema.dialect.modes).ok()?,
        ),
        (
            "features".to_string(),
            serde_json::to_string(&profile.schema.dialect.features).ok()?,
        ),
        (
            "strictness".to_string(),
            format!("{:?}", profile.strictness),
        ),
        (
            "session".to_string(),
            serde_json::to_string(&profile.session).ok()?,
        ),
    ]);
    Some(EmbeddedAnalysisRequest {
        protocol_major: COMPONENT_PROTOCOL_MAJOR,
        component: registration.identity,
        provider_diagnostics: registration.diagnostics,
        compiler_semantic_version: env!("CARGO_PKG_VERSION").to_string(),
        parts,
        holes,
        context: AnalysisContext {
            schema_profile: Some(profile_identity.to_string()),
            schema_fingerprint: Some(schema_fingerprint.to_string()),
            semantic_profile,
            imported_signatures: Vec::new(),
            artifacts: vec![context_artifact],
        },
        plan_kind: PlanKind::Expression,
    })
}

fn template_parts(
    document: &SqlEditorDocumentView,
    source_document: &str,
) -> Option<Vec<TemplatePart>> {
    let mut mappings = document.template.mappings.clone();
    mappings.sort_by_key(|mapping| mapping.virtual_range.start());
    mappings
        .into_iter()
        .map(|mapping| {
            let span = SourceSpan {
                document: source_document.to_string(),
                start: mapping.source_range.start().to_u32(),
                end: mapping.source_range.end().to_u32(),
            };
            match mapping.kind {
                TemplateSourceMapKind::Static => Some(TemplatePart::Static {
                    text: virtual_text(&document.template.source, mapping.virtual_range)?
                        .to_string(),
                    span,
                }),
                TemplateSourceMapKind::Interpolation { index } => Some(TemplatePart::Hole {
                    index: u32::try_from(index).ok()?,
                    span,
                }),
            }
        })
        .collect()
}

fn virtual_text(source: &str, range: ruff_text_size::TextRange) -> Option<&str> {
    source.get(
        usize::try_from(range.start().to_u32()).ok()?
            ..usize::try_from(range.end().to_u32()).ok()?,
    )
}

#[derive(Debug)]
pub(super) enum SqlEditorRuntimeError {
    Cache(String),
    Component(ComponentError),
    Contract(String),
    Operation(EmbeddedProviderOperationError<ComponentError>),
}

impl std::fmt::Display for SqlEditorRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cache(message) | Self::Contract(message) => formatter.write_str(message),
            Self::Component(error) => write!(formatter, "{error}"),
            Self::Operation(EmbeddedProviderOperationError::Provider(error)) => {
                write!(formatter, "{error}")
            }
            Self::Operation(EmbeddedProviderOperationError::Cancelled(cancelled)) => {
                write!(
                    formatter,
                    "SQL editor analysis cancelled at {:?}",
                    cancelled.checkpoint
                )
            }
        }
    }
}
