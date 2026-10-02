//! Provider semantic failures are valid component results with physical source spans.
use crate::ProviderSemanticDiagnostic;
use sifr_compiler_component::{
    ClosedType, ComponentError, DiagnosticLifecycle, DiagnosticSeverity, EmbeddedAnalysisRequest,
    EmbeddedAnalysisResponse, EmbeddedDiagnostic, EmbeddedPlan, RuntimeLowering, SourceSpan,
    TemplatePart, compute_plan_fingerprint,
};

/// Only source locations retained by the checked catalog become diagnostic authority.
pub fn schema_diagnostic_source_artifact(
    schema: &crate::SchemaIr,
) -> Result<Option<sifr_compiler_component::ContextArtifact>, ComponentError> {
    let locations = schema
        .objects
        .values()
        .filter_map(|object| object.source.as_ref())
        .map(|source| SourceSpan {
            document: source.document.clone(),
            start: source.start,
            end: source.end,
        })
        .collect::<Vec<_>>();
    if locations.is_empty() {
        return Ok(None);
    }
    sifr_compiler_component::diagnostic_source_artifact("sifr.sql.schema-sources", &locations)
        .map(Some)
}

pub fn provider_diagnostic_response(
    request: &EmbeddedAnalysisRequest,
    diagnostic: &ProviderSemanticDiagnostic,
) -> Result<EmbeddedAnalysisResponse, ComponentError> {
    let mut response = EmbeddedAnalysisResponse {
        protocol_major: request.protocol_major,
        plan: EmbeddedPlan {
            provider_identity: request.component.processor.clone(),
            protocol_major: request.protocol_major,
            plan_kind: request.plan_kind,
            schema_identity: request.context.schema_profile.clone(),
            result_type: ClosedType::None,
            operations: Vec::new(),
            runtime: RuntimeLowering::NoRuntime,
            dependencies: Vec::new(),
            diagnostics: vec![EmbeddedDiagnostic {
                code: diagnostic.code.clone(),
                severity: DiagnosticSeverity::Error,
                lifecycle: DiagnosticLifecycle::Active,
                message: diagnostic.message.clone(),
                primary: SourceSpan {
                    document: diagnostic.primary.document.clone(),
                    start: diagnostic.primary.start,
                    end: diagnostic.primary.end,
                },
                related: diagnostic
                    .related
                    .iter()
                    .map(|span| SourceSpan {
                        document: span.document.clone(),
                        start: span.start,
                        end: span.end,
                    })
                    .collect(),
            }],
            source_map: Vec::new(),
            stable_fingerprint: String::new(),
        },
    };
    project_provider_diagnostics(request, &mut response)?;
    Ok(response)
}

/// SQL coordinates refer to provider placeholder syntax, not to the Sifr file.
pub fn project_provider_diagnostics(
    request: &EmbeddedAnalysisRequest,
    response: &mut EmbeddedAnalysisResponse,
) -> Result<(), ComponentError> {
    for diagnostic in &mut response.plan.diagnostics {
        diagnostic.primary = project_span(request, &diagnostic.primary);
        for related in &mut diagnostic.related {
            *related = project_span(request, related);
        }
    }
    response.plan.stable_fingerprint = compute_plan_fingerprint(&response.plan)?;
    Ok(())
}

fn project_span(request: &EmbeddedAnalysisRequest, location: &SourceSpan) -> SourceSpan {
    if request
        .parts
        .iter()
        .any(|part| span(part).document == location.document)
    {
        return location.clone();
    }
    let virtual_document = if request.component.processor.contains("postgresql") {
        "sifr://sql/query"
    } else {
        "query.sql"
    };
    if location.document != virtual_document {
        return location.clone();
    }
    let mut virtual_start = 0u32;
    let mut selected = Vec::new();
    for part in &request.parts {
        let source = span(part);
        let length = match part {
            TemplatePart::Static { text, .. } => u32::try_from(text.len()).unwrap_or(u32::MAX),
            TemplatePart::Hole { index, .. } => {
                if request.component.processor.contains("mysql") {
                    1
                } else {
                    1 + u32::try_from(index.saturating_add(1).to_string().len()).unwrap_or(10)
                }
            }
        };
        let virtual_end = virtual_start.saturating_add(length);
        if location.start < virtual_end && location.end > virtual_start {
            let mapped = if matches!(part, TemplatePart::Static { .. })
                && source.end.saturating_sub(source.start) == length
            {
                SourceSpan {
                    document: source.document.clone(),
                    start: source.start + location.start.saturating_sub(virtual_start).min(length),
                    end: source.start + location.end.saturating_sub(virtual_start).min(length),
                }
            } else {
                source.clone()
            };
            selected.push(mapped);
        }
        virtual_start = virtual_end;
    }
    if let (Some(first), Some(last)) = (selected.first(), selected.last()) {
        return SourceSpan {
            document: first.document.clone(),
            start: first.start,
            end: last.end,
        };
    }
    // Provider contract errors without SQL coordinates still identify the template.
    match (request.parts.first(), request.parts.last()) {
        (Some(first), Some(last)) => SourceSpan {
            document: span(first).document.clone(),
            start: span(first).start,
            end: span(last).end,
        },
        _ => location.clone(),
    }
}

fn span(part: &TemplatePart) -> &SourceSpan {
    match part {
        TemplatePart::Static { span, .. } | TemplatePart::Hole { span, .. } => span,
    }
}
