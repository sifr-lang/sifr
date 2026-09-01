use crate::protocol::{
    ClosedType, EmbeddedAnalysisRequest, EmbeddedAnalysisResponse, RuntimeLowering,
    SemanticOperation, SourceSpan, TemplatePart,
};
use crate::{
    ComponentError, ComponentErrorKind, ComponentHostLimits, DiagnosticRegistry,
    compute_plan_fingerprint,
};
use std::collections::BTreeSet;

pub fn validate_request(
    request: &EmbeddedAnalysisRequest,
    limits: &ComponentHostLimits,
) -> Result<(), ComponentError> {
    if request.protocol_major != crate::COMPONENT_PROTOCOL_MAJOR {
        return Err(version_error(request.protocol_major));
    }
    if request.component.package.is_empty()
        || request.component.processor.is_empty()
        || request.compiler_semantic_version.is_empty()
    {
        return Err(envelope_error("request identity fields must not be empty"));
    }
    if semver::Version::parse(&request.compiler_semantic_version).is_err() {
        return Err(envelope_error(
            "compiler semantic version must be one exact semantic version",
        ));
    }
    if !valid_fingerprint(&request.component.sha256) {
        return Err(envelope_error(
            "request component hash must be a SHA-256 hex value",
        ));
    }
    DiagnosticRegistry::compiler()
        .validate_with(std::slice::from_ref(&request.provider_diagnostics))?;
    if request.parts.len() > limits.max_template_parts
        || request.holes.len() > limits.max_holes
        || request.context.artifacts.len() > limits.max_context_artifacts
        || request.context.imported_signatures.len() > limits.max_dependencies
    {
        return Err(limit_error("request collection limit exceeded"));
    }
    let mut hole_indices = BTreeSet::new();
    let mut previous_hole = None;
    for hole in &request.holes {
        if previous_hole.is_some_and(|index| index >= hole.index) {
            return Err(envelope_error(
                "hole descriptors must use unique ascending indices",
            ));
        }
        previous_hole = Some(hole.index);
        if !hole_indices.insert(hole.index) {
            return Err(envelope_error("hole indices must be unique"));
        }
        if hole
            .fragment_identity
            .as_ref()
            .is_some_and(String::is_empty)
        {
            return Err(envelope_error("fragment identity must not be empty"));
        }
        validate_type(&hole.ty, 0, limits)?;
    }
    let mut referenced_holes = BTreeSet::new();
    let mut previous_span: Option<(&str, u32)> = None;
    for part in &request.parts {
        let span = match part {
            TemplatePart::Static { text, span } => {
                if text.len() > limits.max_static_segment_bytes {
                    return Err(limit_error("static template segment is too large"));
                }
                validate_span(span)?;
                span
            }
            TemplatePart::Hole { index, span } => {
                if !hole_indices.contains(index) {
                    return Err(envelope_error("template references an unknown hole"));
                }
                if !referenced_holes.insert(*index) {
                    return Err(envelope_error(
                        "template references one hole more than once",
                    ));
                }
                validate_span(span)?;
                span
            }
        };
        if previous_span
            .is_some_and(|(document, end)| document != span.document.as_str() || end > span.start)
        {
            return Err(envelope_error(
                "template source spans must use one document and canonical order",
            ));
        }
        previous_span = Some((&span.document, span.end));
    }
    if referenced_holes != hole_indices {
        return Err(envelope_error(
            "each typed hole must occur exactly once in the template",
        ));
    }
    if request
        .context
        .schema_profile
        .as_ref()
        .is_some_and(String::is_empty)
    {
        return Err(envelope_error("schema profile identity must not be empty"));
    }
    if request
        .context
        .schema_fingerprint
        .as_ref()
        .is_some_and(|value| !valid_fingerprint(value))
    {
        return Err(envelope_error(
            "schema fingerprint must be a SHA-256 hex value",
        ));
    }
    if request
        .context
        .imported_signatures
        .windows(2)
        .any(|items| items[0] >= items[1])
        || request
            .context
            .imported_signatures
            .iter()
            .any(String::is_empty)
    {
        return Err(envelope_error(
            "imported signatures must be non-empty, unique, and sorted",
        ));
    }
    let mut previous_artifact = None;
    for artifact in &request.context.artifacts {
        if artifact.kind.is_empty()
            || artifact.identity.is_empty()
            || artifact.format_version == 0
            || !valid_fingerprint(&artifact.fingerprint)
            || artifact.payload.is_empty()
        {
            return Err(envelope_error(
                "context artifacts require a kind, identity, format version, fingerprint, and payload",
            ));
        }
        if artifact.payload.len() > limits.max_context_artifact_bytes {
            return Err(limit_error("context artifact exceeds its byte limit"));
        }
        let key = (&artifact.kind, &artifact.identity);
        if previous_artifact.is_some_and(|previous| previous >= key) {
            return Err(envelope_error(
                "context artifacts must be non-empty, unique, and sorted by kind and identity",
            ));
        }
        previous_artifact = Some(key);
    }
    Ok(())
}

pub fn validate_response(
    request: &EmbeddedAnalysisRequest,
    response: &EmbeddedAnalysisResponse,
    limits: &ComponentHostLimits,
    provider_diagnostics: &DiagnosticRegistry,
) -> Result<(), ComponentError> {
    if response.protocol_major != request.protocol_major
        || response.plan.protocol_major != request.protocol_major
    {
        return Err(version_error(response.protocol_major));
    }
    if response.plan.plan_kind != request.plan_kind {
        return Err(envelope_error("component changed the requested plan kind"));
    }
    if response.plan.provider_identity != request.component.processor {
        return Err(envelope_error(
            "component response identity does not match the request",
        ));
    }
    if response.plan.diagnostics.len() > limits.max_diagnostics
        || response.plan.dependencies.len() > limits.max_dependencies
        || response.plan.source_map.len() > limits.max_source_map_entries
        || response.plan.operations.len() > limits.max_operations
    {
        return Err(limit_error("response collection limit exceeded"));
    }
    validate_type(&response.plan.result_type, 0, limits)?;
    let known_holes = request
        .holes
        .iter()
        .map(|hole| hole.index)
        .collect::<BTreeSet<_>>();
    let request_documents = request
        .parts
        .iter()
        .map(|part| match part {
            TemplatePart::Static { span, .. } | TemplatePart::Hole { span, .. } => {
                span.document.as_str()
            }
        })
        .collect::<BTreeSet<_>>();
    let mut operation_count = 0;
    for operation in &response.plan.operations {
        validate_operation(operation, 0, &mut operation_count, &known_holes, limits)?;
    }
    for diagnostic in &response.plan.diagnostics {
        if diagnostic.message.len() > limits.max_diagnostic_bytes {
            return Err(limit_error("diagnostic message is too large"));
        }
        validate_span(&diagnostic.primary)?;
        validate_response_document(&diagnostic.primary, &request_documents)?;
        for span in &diagnostic.related {
            validate_span(span)?;
            validate_response_document(span, &request_documents)?;
        }
        if provider_diagnostics.lifecycle_for(&diagnostic.code) != Some(diagnostic.lifecycle) {
            return Err(ComponentError::new(
                ComponentErrorKind::DiagnosticRegistry,
                format!(
                    "provider diagnostic '{}' is undeclared or has different lifecycle metadata",
                    diagnostic.code
                ),
            ));
        }
    }
    let mut previous_provider_end = None;
    for mapping in &response.plan.source_map {
        if mapping.provider_start > mapping.provider_end {
            return Err(envelope_error("provider source-map range is reversed"));
        }
        if previous_provider_end.is_some_and(|end| end > mapping.provider_start) {
            return Err(envelope_error(
                "provider source-map entries must be sorted and non-overlapping",
            ));
        }
        previous_provider_end = Some(mapping.provider_end);
        validate_span(&mapping.source)?;
        validate_response_document(&mapping.source, &request_documents)?;
    }
    validate_dependencies(response)?;
    validate_runtime(&response.plan.runtime, &known_holes)?;
    if !valid_fingerprint(&response.plan.stable_fingerprint) {
        return Err(envelope_error(
            "plan fingerprint must be a SHA-256 hex value",
        ));
    }
    if compute_plan_fingerprint(&response.plan)? != response.plan.stable_fingerprint {
        return Err(envelope_error(
            "plan fingerprint does not match the canonical plan contents",
        ));
    }
    Ok(())
}

fn validate_response_document(
    span: &SourceSpan,
    request_documents: &BTreeSet<&str>,
) -> Result<(), ComponentError> {
    if !request_documents.contains(span.document.as_str()) {
        return Err(envelope_error(
            "component response span names a document outside the request",
        ));
    }
    Ok(())
}

fn validate_dependencies(response: &EmbeddedAnalysisResponse) -> Result<(), ComponentError> {
    let mut previous = None;
    for dependency in &response.plan.dependencies {
        if dependency.identity.is_empty() || !valid_fingerprint(&dependency.fingerprint) {
            return Err(envelope_error("component dependency descriptor is invalid"));
        }
        if previous.is_some_and(|identity: &str| identity >= dependency.identity.as_str()) {
            return Err(envelope_error(
                "component dependencies must have unique canonical identities",
            ));
        }
        previous = Some(dependency.identity.as_str());
    }
    Ok(())
}

fn validate_runtime(
    runtime: &RuntimeLowering,
    known_holes: &BTreeSet<u32>,
) -> Result<(), ComponentError> {
    let RuntimeLowering::ProviderCall {
        declaration,
        parameter_order,
        ..
    } = runtime
    else {
        return Ok(());
    };
    if declaration.is_empty() {
        return Err(envelope_error("runtime declaration must not be empty"));
    }
    let mut seen = BTreeSet::new();
    if parameter_order
        .iter()
        .any(|index| !known_holes.contains(index) || !seen.insert(*index))
    {
        return Err(envelope_error(
            "runtime parameter order must contain unique known holes",
        ));
    }
    Ok(())
}

fn validate_type(
    ty: &ClosedType,
    depth: usize,
    limits: &ComponentHostLimits,
) -> Result<(), ComponentError> {
    if depth > limits.max_type_depth {
        return Err(limit_error("type recursion limit exceeded"));
    }
    match ty {
        ClosedType::Optional { item } | ClosedType::List { item } => {
            validate_type(item, depth + 1, limits)
        }
        ClosedType::Tuple { items } => {
            if items.len() > limits.max_type_width {
                return Err(limit_error("tuple type width limit exceeded"));
            }
            for item in items {
                validate_type(item, depth + 1, limits)?;
            }
            Ok(())
        }
        ClosedType::Record { fields } => {
            if fields.len() > limits.max_type_width {
                return Err(limit_error("record type width limit exceeded"));
            }
            let mut previous = None;
            for field in fields {
                if field.name.is_empty()
                    || previous.is_some_and(|name: &str| name >= field.name.as_str())
                {
                    return Err(envelope_error(
                        "record fields must have unique canonical names",
                    ));
                }
                previous = Some(field.name.as_str());
                validate_type(&field.ty, depth + 1, limits)?;
            }
            Ok(())
        }
        ClosedType::Bool
        | ClosedType::Int
        | ClosedType::Float
        | ClosedType::Str
        | ClosedType::Bytes
        | ClosedType::None => Ok(()),
    }
}

fn validate_operation(
    operation: &SemanticOperation,
    depth: usize,
    count: &mut usize,
    known_holes: &BTreeSet<u32>,
    limits: &ComponentHostLimits,
) -> Result<(), ComponentError> {
    if depth > limits.max_operation_depth {
        return Err(limit_error("semantic plan recursion limit exceeded"));
    }
    *count = count.saturating_add(1);
    if *count > limits.max_operations {
        return Err(limit_error("semantic operation count limit exceeded"));
    }
    match operation {
        SemanticOperation::Hole { index } if !known_holes.contains(index) => {
            Err(envelope_error("semantic plan references an unknown hole"))
        }
        SemanticOperation::Sequence { operations } => {
            if operations.len() > limits.max_operations {
                return Err(limit_error("semantic sequence is too large"));
            }
            for item in operations {
                validate_operation(item, depth + 1, count, known_holes, limits)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_span(span: &SourceSpan) -> Result<(), ComponentError> {
    if span.document.is_empty() || span.start > span.end {
        return Err(envelope_error("source span is invalid"));
    }
    Ok(())
}

fn envelope_error(message: &'static str) -> ComponentError {
    ComponentError::new(ComponentErrorKind::ProtocolEnvelope, message)
}

fn limit_error(message: &'static str) -> ComponentError {
    ComponentError::new(ComponentErrorKind::ResourceLimit, message)
}

fn version_error(version: u16) -> ComponentError {
    ComponentError::new(
        ComponentErrorKind::ProtocolVersion,
        format!("unsupported component protocol major version {version}"),
    )
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
