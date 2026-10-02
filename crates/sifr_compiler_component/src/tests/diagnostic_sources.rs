use super::{provider_registry, request, response};
use crate::{
    ComponentErrorKind, ComponentHostLimits, SourceSpan, diagnostic_source_artifact,
    validate_request, validate_response,
};

fn location(start: u32, end: u32) -> SourceSpan {
    SourceSpan {
        document: "db/schema.sql".into(),
        start,
        end,
    }
}

#[test]
fn declared_schema_locations_authorize_diagnostics_but_never_template_maps() {
    let mut request = request();
    request
        .context
        .artifacts
        .push(diagnostic_source_artifact("schema", &[location(200, 250)]).unwrap());
    let limits = ComponentHostLimits::default();
    validate_request(&request, &limits).unwrap();
    let mut response = response();
    response.plan.diagnostics[0]
        .related
        .push(location(211, 220));
    response.plan.stable_fingerprint = crate::compute_plan_fingerprint(&response.plan).unwrap();
    validate_response(&request, &response, &limits, &provider_registry()).unwrap();
    for bad in [
        location(199, 220),
        location(220, 251),
        SourceSpan {
            document: "forged.sql".into(),
            start: 211,
            end: 220,
        },
    ] {
        response.plan.diagnostics[0].related = vec![bad];
        assert_eq!(
            validate_response(&request, &response, &limits, &provider_registry())
                .unwrap_err()
                .kind,
            ComponentErrorKind::ProtocolEnvelope
        );
    }
    response.plan.diagnostics[0].related.clear();
    response.plan.source_map[0].source = location(211, 220);
    assert_eq!(
        validate_response(&request, &response, &limits, &provider_registry())
            .unwrap_err()
            .kind,
        ComponentErrorKind::ProtocolEnvelope
    );
}

#[test]
fn declared_diagnostic_sources_require_canonical_bounded_integrity() {
    let mut request = request();
    request.context.artifacts.push(
        diagnostic_source_artifact("schema", &[location(200, 250), location(300, 350)]).unwrap(),
    );
    let limits = ComponentHostLimits::default();
    validate_request(&request, &limits).unwrap();
    let mut bounded = limits.clone();
    bounded.max_source_map_entries = 1;
    assert_eq!(
        validate_request(&request, &bounded).unwrap_err().kind,
        ComponentErrorKind::ResourceLimit
    );
    let original = request.context.artifacts[0].clone();
    for invalid in [
        {
            let mut a = original.clone();
            a.fingerprint = "0".repeat(64);
            a
        },
        {
            let mut a = original.clone();
            a.format_version = 2;
            a
        },
        diagnostic_source_artifact(
            "schema",
            &[SourceSpan {
                document: "".into(),
                start: 0,
                end: 1,
            }],
        )
        .unwrap(),
        diagnostic_source_artifact("schema", &[location(250, 200)]).unwrap(),
    ] {
        request.context.artifacts[0] = invalid;
        assert_eq!(
            validate_request(&request, &limits).unwrap_err().kind,
            ComponentErrorKind::ProtocolEnvelope
        );
    }
    request.context.artifacts[0] = original;
    let artifact = &mut request.context.artifacts[0];
    artifact.payload = serde_json::to_vec(&vec![location(300, 350), location(200, 250)]).unwrap();
    use sha2::{Digest, Sha256};
    artifact.fingerprint =
        crate::registration::hex_digest(Sha256::digest(&artifact.payload).as_slice());
    assert_eq!(
        validate_request(&request, &limits).unwrap_err().kind,
        ComponentErrorKind::ProtocolEnvelope
    );
}
