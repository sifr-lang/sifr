use crate::registration::{hex_digest, verify_component_hash};
use crate::*;
use semver::Version;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn compiler_owned_wit_has_one_closed_export_and_no_imports() {
    assert!(COMPILER_COMPONENT_WIT.contains("export analyze: func(request: list<u8>) -> list<u8>"));
    assert!(!COMPILER_COMPONENT_WIT.contains("import "));
    assert!(!COMPILER_COMPONENT_WIT.contains("wasi:"));
}

#[test]
fn exact_registration_rejects_hash_drift_duplicates_and_protocol_downgrade() {
    let bytes = wat::parse_str("(component)").expect("minimal component should parse");
    let identity = identity(&bytes);
    let registration = registration(identity.clone());
    let resolved = resolve_component(
        &ComponentRequirement {
            identity: identity.clone(),
            protocol_major: 1,
        },
        [ResolvedComponent {
            registration: registration.clone(),
            bytes: bytes.clone(),
        }],
    )
    .expect("exact component should resolve");
    assert_eq!(resolved.registration, registration);

    let duplicate = resolve_component(
        &ComponentRequirement {
            identity: identity.clone(),
            protocol_major: 1,
        },
        [
            ResolvedComponent {
                registration: registration.clone(),
                bytes: bytes.clone(),
            },
            ResolvedComponent {
                registration: registration.clone(),
                bytes: bytes.clone(),
            },
        ],
    )
    .expect_err("duplicate exact identities must fail");
    assert_eq!(duplicate.kind, ComponentErrorKind::Registration);

    let downgrade = resolve_component(
        &ComponentRequirement {
            identity,
            protocol_major: 2,
        },
        [ResolvedComponent {
            registration,
            bytes,
        }],
    )
    .expect_err("protocol downgrade must fail");
    assert_eq!(downgrade.kind, ComponentErrorKind::ProtocolVersion);
}

#[test]
fn diagnostic_registries_are_stable_disjoint_and_lifecycle_tagged() {
    let compiler = DiagnosticRegistry::compiler();
    let provider = DiagnosticRegistry {
        owner: DiagnosticRegistryOwner::Provider {
            namespace: "FIXTURE".to_string(),
        },
        declarations: vec![DiagnosticCodeDeclaration {
            code: "SIFR-FIXTURE-0001".to_string(),
            lifecycle: DiagnosticLifecycle::Deprecated,
        }],
    };
    compiler
        .validate_with(&[provider])
        .expect("disjoint registries should pass");

    let stolen = DiagnosticRegistry {
        owner: DiagnosticRegistryOwner::Provider {
            namespace: "COMPONENT".to_string(),
        },
        declarations: Vec::new(),
    };
    assert_eq!(
        stolen
            .validate_with(&[])
            .expect_err("namespace is sealed")
            .kind,
        ComponentErrorKind::DiagnosticRegistry
    );

    let stolen = DiagnosticRegistry {
        owner: DiagnosticRegistryOwner::Provider {
            namespace: "IMPORT".to_string(),
        },
        declarations: Vec::new(),
    };
    assert_eq!(
        stolen
            .validate_with(&[])
            .expect_err("every canonical compiler namespace is sealed")
            .kind,
        ComponentErrorKind::DiagnosticRegistry
    );

    let canonical_codes = ComponentErrorKind::ALL.map(ComponentErrorKind::code);
    assert_eq!(
        compiler
            .declarations
            .iter()
            .map(|declaration| declaration.code.as_str())
            .collect::<Vec<_>>(),
        canonical_codes
    );
    for code in canonical_codes {
        assert!(
            sifr_diagnostics::registry_entry(code).is_some(),
            "component error code {code} must be in the canonical registry"
        );
    }
}

#[test]
fn non_sql_component_parses_typed_requests_and_is_cacheless_deterministic() {
    let first_request = request();
    let first_response = response_for_request(&first_request);
    let mut second_request = first_request.clone();
    let TemplatePart::Static { text, .. } = &mut second_request.parts[0] else {
        panic!("fixture must start with static text");
    };
    *text = "world ".to_string();
    second_request.holes[0].ty = ClosedType::Int;
    let second_response = response_for_request(&second_request);
    let bytes = routed_fixture_component(&[
        (first_request.clone(), first_response.clone()),
        (second_request.clone(), second_response.clone()),
    ]);
    let identity = identity(&bytes);
    let mut first_request = first_request;
    first_request.component = identity.clone();
    let mut second_request = second_request;
    second_request.component = identity.clone();
    let registration = registration(identity.clone());

    for (request, expected) in [
        (&first_request, &first_response),
        (&second_request, &second_response),
    ] {
        let mut first_host = ComponentHost::new(ComponentHostLimits::default(), None)
            .expect("component host should initialize");
        let mut second_host = ComponentHost::new(ComponentHostLimits::default(), None)
            .expect("component host should initialize");
        let first = first_host
            .analyze(&registration, &bytes, request)
            .expect("fixture component should parse the request");
        let second = second_host
            .analyze(&registration, &bytes, request)
            .expect("a fresh cacheless host should reproduce the plan");
        assert_eq!(&first.response, expected);
        assert_eq!(first.response, second.response);
        assert!(!first.cache_hit);
        assert!(!second.cache_hit);
    }
    assert_ne!(
        first_response.plan.operations,
        second_response.plan.operations
    );
    assert_ne!(
        first_response.plan.result_type,
        second_response.plan.result_type
    );
    assert_ne!(
        first_response.plan.dependencies,
        second_response.plan.dependencies
    );

    let root = temp_root("round_trip");
    let cache = AnalysisCache::open(&root, 8 * 1024 * 1024).expect("cache should open");
    let mut host = ComponentHost::new(ComponentHostLimits::default(), Some(cache))
        .expect("component host should initialize");
    let first = host
        .analyze(&registration, &bytes, &first_request)
        .expect("fixture component should run");
    let second = host
        .analyze(&registration, &bytes, &first_request)
        .expect("fixture component should use cache");
    assert_eq!(first.response, first_response);
    assert!(!first.cache_hit);
    assert!(second.cache_hit);
    assert_eq!(first.cache_key, second.cache_key);
    std::fs::remove_dir_all(root).expect("fixture cache should be removable");
}

#[test]
fn malformed_output_and_missing_wit_export_are_structured_errors() {
    let malformed = fixture_component(b"not-json", false);
    let malformed_identity = identity(&malformed);
    let mut malformed_request = request();
    malformed_request.component = malformed_identity.clone();
    let mut host = ComponentHost::new(ComponentHostLimits::default(), None)
        .expect("component host should initialize");
    let error = host
        .analyze(
            &registration(malformed_identity),
            &malformed,
            &malformed_request,
        )
        .expect_err("malformed response must fail");
    assert_eq!(error.kind, ComponentErrorKind::ProtocolEnvelope);

    let missing = wat::parse_str("(component)").expect("minimal component should parse");
    let missing_identity = identity(&missing);
    let mut missing_request = request();
    missing_request.component = missing_identity.clone();
    let error = host
        .analyze(&registration(missing_identity), &missing, &missing_request)
        .expect_err("missing analyze export must fail");
    assert_eq!(error.kind, ComponentErrorKind::ProtocolEnvelope);
}

#[test]
fn undeclared_provider_diagnostic_and_forged_plan_fingerprint_are_rejected() {
    let mut undeclared = response();
    undeclared.plan.diagnostics[0].code = "SIFR-FIXTURE-9999".to_string();
    undeclared.plan.stable_fingerprint =
        compute_plan_fingerprint(&undeclared.plan).expect("fixture plan should fingerprint");
    let output = serde_json::to_vec(&undeclared).expect("fixture response should serialize");
    let bytes = fixture_component(&output, false);
    let undeclared_identity = identity(&bytes);
    let mut request = request();
    request.component = undeclared_identity.clone();
    let mut host = ComponentHost::new(ComponentHostLimits::default(), None)
        .expect("component host should initialize");
    let error = host
        .analyze(&registration(undeclared_identity), &bytes, &request)
        .expect_err("undeclared provider diagnostic must fail");
    assert_eq!(error.kind, ComponentErrorKind::DiagnosticRegistry);

    let mut forged = response();
    forged.plan.stable_fingerprint = "0".repeat(64);
    let output = serde_json::to_vec(&forged).expect("fixture response should serialize");
    let bytes = fixture_component(&output, false);
    let identity = identity(&bytes);
    request.component = identity.clone();
    let error = host
        .analyze(&registration(identity), &bytes, &request)
        .expect_err("forged plan fingerprint must fail");
    assert_eq!(error.kind, ComponentErrorKind::ProtocolEnvelope);
}

#[test]
fn ambient_wasi_import_is_denied_by_the_empty_linker() {
    let bytes = wat::parse_str(
        r#"(component
            (type $clock (instance
                (type $now (func (result u64)))
                (export "now" (func (type $now)))))
            (import "wasi:clocks/wall-clock@0.2.0" (instance $clock (type $clock))))"#,
    )
    .expect("ambient import fixture should parse");
    let identity = identity(&bytes);
    let mut request = request();
    request.component = identity.clone();
    let mut host = ComponentHost::new(ComponentHostLimits::default(), None)
        .expect("component host should initialize");
    let error = host
        .analyze(&registration(identity), &bytes, &request)
        .expect_err("ambient capability must not be linked");
    assert_eq!(error.kind, ComponentErrorKind::Capability, "{error}");
}

#[test]
fn shared_memory_is_denied_before_component_execution() {
    let bytes = wat::parse_str("(component (core module (memory 1 1 shared)))")
        .expect("shared-memory fixture should parse");
    let identity = identity(&bytes);
    let mut request = request();
    request.component = identity.clone();
    let mut host = ComponentHost::new(ComponentHostLimits::default(), None)
        .expect("component host should initialize");
    let error = host
        .analyze(&registration(identity), &bytes, &request)
        .expect_err("shared memory must not compile");
    assert_eq!(error.kind, ComponentErrorKind::Capability, "{error}");
}

#[test]
fn fuel_and_output_bounds_fail_without_panics() {
    let infinite = fixture_component(&[], true);
    let infinite_identity = identity(&infinite);
    let mut infinite_request = request();
    infinite_request.component = infinite_identity.clone();
    let mut limits = ComponentHostLimits::default();
    limits.fuel = 10_000;
    let mut host = ComponentHost::new(limits, None).expect("host should initialize");
    let error = host
        .analyze(
            &registration(infinite_identity),
            &infinite,
            &infinite_request,
        )
        .expect_err("infinite component must exhaust fuel");
    assert_eq!(error.kind, ComponentErrorKind::ResourceLimit, "{error}");

    let response_bytes = serde_json::to_vec(&response()).expect("response should serialize");
    let memory_bounded = fixture_component(&response_bytes, false);
    let memory_identity = identity(&memory_bounded);
    let mut memory_request = request();
    memory_request.component = memory_identity.clone();
    let mut limits = ComponentHostLimits::default();
    limits.max_memory_bytes = 1024;
    let mut host = ComponentHost::new(limits, None).expect("host should initialize");
    let error = host
        .analyze(
            &registration(memory_identity),
            &memory_bounded,
            &memory_request,
        )
        .expect_err("component memory minimum must respect the host limit");
    assert_eq!(error.kind, ComponentErrorKind::ResourceLimit, "{error}");

    let response = serde_json::to_vec(&response()).expect("response should serialize");
    let bounded = fixture_component(&response, false);
    let bounded_identity = identity(&bounded);
    let mut bounded_request = request();
    bounded_request.component = bounded_identity.clone();
    let mut limits = ComponentHostLimits::default();
    limits.max_output_bytes = 8;
    let mut host = ComponentHost::new(limits, None).expect("host should initialize");
    let error = host
        .analyze(
            &registration(bounded_identity.clone()),
            &bounded,
            &bounded_request,
        )
        .expect_err("oversized output must fail");
    assert_eq!(error.kind, ComponentErrorKind::ResourceLimit);

    let mut limits = ComponentHostLimits::default();
    limits.max_component_bytes = bounded.len().saturating_sub(1);
    let mut host = ComponentHost::new(limits, None).expect("host should initialize");
    let error = host
        .analyze(&registration(bounded_identity), &bounded, &bounded_request)
        .expect_err("oversized component artifact must fail before compilation");
    assert_eq!(error.kind, ComponentErrorKind::ResourceLimit);
}

#[test]
fn protocol_recursion_input_and_diagnostic_bounds_are_structured() {
    let mut recursive_request = request();
    recursive_request.holes[0].ty = ClosedType::Optional {
        item: Box::new(ClosedType::Optional {
            item: Box::new(ClosedType::Int),
        }),
    };
    let mut limits = ComponentHostLimits::default();
    limits.max_type_depth = 1;
    assert_eq!(
        validate_request(&recursive_request, &limits)
            .expect_err("nested type must respect the recursion limit")
            .kind,
        ComponentErrorKind::ResourceLimit
    );

    let request = request();
    let response = response();
    let mut limits = ComponentHostLimits::default();
    limits.max_diagnostics = 0;
    assert_eq!(
        validate_response(&request, &response, &limits, &provider_registry())
            .expect_err("diagnostic count must respect the limit")
            .kind,
        ComponentErrorKind::ResourceLimit
    );

    let output = serde_json::to_vec(&response).expect("response should serialize");
    let bytes = fixture_component(&output, false);
    let identity = identity(&bytes);
    let mut request = request;
    request.component = identity.clone();
    let mut limits = ComponentHostLimits::default();
    limits.max_input_bytes = 1;
    let mut host = ComponentHost::new(limits, None).expect("host should initialize");
    assert_eq!(
        host.analyze(&registration(identity), &bytes, &request)
            .expect_err("serialized request must respect the input limit")
            .kind,
        ComponentErrorKind::ResourceLimit
    );
}

#[test]
fn cached_responses_obey_the_current_host_output_limit() {
    let response_bytes = serde_json::to_vec(&response()).expect("response should serialize");
    let bytes = fixture_component(&response_bytes, false);
    let identity = identity(&bytes);
    let registration = registration(identity.clone());
    let mut request = request();
    request.component = identity;
    let root = temp_root("cache_output_limit");

    let cache = AnalysisCache::open(&root, 8 * 1024 * 1024).expect("cache should open");
    let mut host = ComponentHost::new(ComponentHostLimits::default(), Some(cache))
        .expect("component host should initialize");
    host.analyze(&registration, &bytes, &request)
        .expect("fixture response should populate the cache");
    drop(host);

    let cache = AnalysisCache::open(&root, 8 * 1024 * 1024).expect("cache should reopen");
    let mut limits = ComponentHostLimits::default();
    limits.max_output_bytes = 8;
    let mut host = ComponentHost::new(limits, Some(cache)).expect("host should initialize");
    let error = host
        .analyze(&registration, &bytes, &request)
        .expect_err("cache hits must enforce the current output limit");
    assert_eq!(error.kind, ComponentErrorKind::ResourceLimit);
    std::fs::remove_dir_all(root).expect("fixture cache should be removable");
}

#[test]
fn cache_identity_changes_for_every_semantic_input_family() {
    let base = request();
    let base_key = CacheKey::for_request(&base).expect("key should derive");
    let mut variants = Vec::new();
    let mut changed = base.clone();
    changed.parts.push(TemplatePart::Static {
        text: "tail".to_string(),
        span: span(9, 13),
    });
    variants.push(changed);
    let mut changed = base.clone();
    changed.holes[0].ty = ClosedType::Int;
    variants.push(changed);
    let mut changed = base.clone();
    changed.context.schema_fingerprint = Some("b".repeat(64));
    variants.push(changed);
    let mut changed = base.clone();
    changed.component.processor = "fixture.other".to_string();
    variants.push(changed);
    let mut changed = base.clone();
    changed.provider_diagnostics.declarations[0].lifecycle = DiagnosticLifecycle::Deprecated;
    variants.push(changed);
    let mut changed = base;
    changed.compiler_semantic_version = "0.0.1".to_string();
    variants.push(changed);
    for variant in variants {
        assert_ne!(
            CacheKey::for_request(&variant).expect("key should derive"),
            base_key
        );
    }
}

#[test]
fn closed_envelopes_reject_unknown_fields_and_noncanonical_records() {
    let mut value = serde_json::to_value(request()).expect("request should serialize");
    value.as_object_mut().expect("request is an object").insert(
        "rust_source".to_string(),
        serde_json::Value::String("bad".to_string()),
    );
    assert!(serde_json::from_value::<EmbeddedAnalysisRequest>(value).is_err());

    let mut bad = request();
    bad.holes[0].ty = ClosedType::Record {
        fields: vec![
            RecordField {
                name: "z".to_string(),
                ty: ClosedType::Int,
            },
            RecordField {
                name: "a".to_string(),
                ty: ClosedType::Int,
            },
        ],
    };
    assert_eq!(
        validate_request(&bad, &ComponentHostLimits::default())
            .expect_err("record order must be canonical")
            .kind,
        ComponentErrorKind::ProtocolEnvelope
    );
}

fn request() -> EmbeddedAnalysisRequest {
    EmbeddedAnalysisRequest {
        protocol_major: COMPONENT_PROTOCOL_MAJOR,
        component: ComponentIdentity {
            package: "fixture-package".to_string(),
            processor: "fixture.words".to_string(),
            version: Version::new(1, 0, 0),
            sha256: "0".repeat(64),
        },
        provider_diagnostics: provider_registry(),
        compiler_semantic_version: "0.0.0".to_string(),
        parts: vec![
            TemplatePart::Static {
                text: "hello ".to_string(),
                span: span(0, 6),
            },
            TemplatePart::Hole {
                index: 0,
                span: span(6, 9),
            },
        ],
        holes: vec![HoleDescriptor {
            index: 0,
            ty: ClosedType::Str,
            fragment_identity: None,
        }],
        context: AnalysisContext {
            schema_profile: None,
            schema_fingerprint: None,
            semantic_profile: BTreeMap::from([("language".to_string(), "words".to_string())]),
            imported_signatures: vec!["fixture.token".to_string()],
        },
        plan_kind: PlanKind::Document,
    }
}

fn response() -> EmbeddedAnalysisResponse {
    response_for_request(&request())
}

fn response_for_request(request: &EmbeddedAnalysisRequest) -> EmbeddedAnalysisResponse {
    let TemplatePart::Static { text, span: source } = &request.parts[0] else {
        panic!("fixture must start with static text");
    };
    let token_end = u32::try_from(text.trim_end().len()).expect("fixture token length should fit");
    let mut response = EmbeddedAnalysisResponse {
        protocol_major: COMPONENT_PROTOCOL_MAJOR,
        plan: EmbeddedPlan {
            provider_identity: "fixture.words".to_string(),
            protocol_major: COMPONENT_PROTOCOL_MAJOR,
            plan_kind: PlanKind::Document,
            schema_identity: None,
            result_type: ClosedType::Record {
                fields: vec![RecordField {
                    name: "token".to_string(),
                    ty: request.holes[0].ty.clone(),
                }],
            },
            operations: vec![SemanticOperation::Sequence {
                operations: vec![
                    SemanticOperation::Literal {
                        value: text.clone(),
                    },
                    SemanticOperation::Hole { index: 0 },
                ],
            }],
            runtime: RuntimeLowering::NoRuntime,
            dependencies: vec![DependencyDescriptor {
                identity: "fixture.dictionary".to_string(),
                fingerprint: hex_digest(Sha256::digest(text.as_bytes()).as_slice()),
            }],
            diagnostics: vec![EmbeddedDiagnostic {
                code: "SIFR-FIXTURE-0001".to_string(),
                severity: DiagnosticSeverity::Note,
                lifecycle: DiagnosticLifecycle::Active,
                message: format!("recognized '{}'", text.trim_end()),
                primary: SourceSpan {
                    document: source.document.clone(),
                    start: source.start,
                    end: source.start + token_end,
                },
                related: Vec::new(),
            }],
            source_map: vec![SourceMapEntry {
                provider_start: 0,
                provider_end: token_end,
                source: SourceSpan {
                    document: source.document.clone(),
                    start: source.start,
                    end: source.start + token_end,
                },
            }],
            stable_fingerprint: String::new(),
        },
    };
    response.plan.stable_fingerprint =
        compute_plan_fingerprint(&response.plan).expect("fixture plan should fingerprint");
    response
}

fn span(start: u32, end: u32) -> SourceSpan {
    SourceSpan {
        document: "fixture.sifr".to_string(),
        start,
        end,
    }
}

fn identity(bytes: &[u8]) -> ComponentIdentity {
    ComponentIdentity {
        package: "fixture-package".to_string(),
        processor: "fixture.words".to_string(),
        version: Version::new(1, 0, 0),
        sha256: hex_digest(Sha256::digest(bytes).as_slice()),
    }
}

fn registration(identity: ComponentIdentity) -> ComponentRegistration {
    ComponentRegistration {
        identity,
        protocol: ProtocolRange {
            minimum: 1,
            maximum: 1,
        },
        artifact: "components/fixture.wasm".to_string(),
        diagnostics: provider_registry(),
    }
}

fn provider_registry() -> DiagnosticRegistry {
    DiagnosticRegistry {
        owner: DiagnosticRegistryOwner::Provider {
            namespace: "FIXTURE".to_string(),
        },
        declarations: vec![DiagnosticCodeDeclaration {
            code: "SIFR-FIXTURE-0001".to_string(),
            lifecycle: DiagnosticLifecycle::Active,
        }],
    }
}

fn fixture_component(output: &[u8], infinite: bool) -> Vec<u8> {
    let escaped = output
        .iter()
        .map(|byte| format!("\\{:02x}", byte))
        .collect::<String>();
    let body = if infinite {
        "(loop $forever (result i32) (br $forever))".to_string()
    } else {
        format!(
            "i32.const 0 i32.const 16 i32.store i32.const 0 i32.const {} i32.store offset=4 i32.const 0",
            output.len()
        )
    };
    let source = format!(
        r#"(component
            (type $analyze-type (func (param "request" (list u8)) (result (list u8))))
            (core module $module
                (memory (export "memory") 1)
                (data (i32.const 16) "{escaped}")
                (global $next (mut i32) (i32.const 4096))
                (func (export "cabi_realloc")
                    (param $old i32) (param $old-size i32) (param $align i32) (param $new-size i32)
                    (result i32)
                    (local $result i32)
                    global.get $next
                    local.tee $result
                    local.get $new-size
                    i32.add
                    global.set $next
                    local.get $result)
                (func (export "analyze")
                    (param $request i32) (param $request-len i32) (result i32)
                    {body}))
            (core instance $instance (instantiate $module))
            (alias core export $instance "memory" (core memory $memory))
            (alias core export $instance "cabi_realloc" (core func $realloc))
            (alias core export $instance "analyze" (core func $analyze))
            (func $lifted (type $analyze-type)
                (canon lift (core func $analyze) (memory $memory) (realloc $realloc)))
            (export "analyze" (func $lifted)))"#
    );
    wat::parse_str(source).expect("fixture component WAT should parse")
}

fn routed_fixture_component(
    routes: &[(EmbeddedAnalysisRequest, EmbeddedAnalysisResponse)],
) -> Vec<u8> {
    struct RouteLayout {
        expected: usize,
        mask: usize,
        output: usize,
        input_len: usize,
        output_len: usize,
    }

    let mut cursor = 64_usize;
    let mut data_segments = String::new();
    let mut layouts = Vec::new();
    for (request, response) in routes {
        let expected = serde_json::to_vec(request).expect("fixture request should serialize");
        let output = serde_json::to_vec(response).expect("fixture response should serialize");
        let mut mask = vec![1_u8; expected.len()];
        let marker = b"\"sha256\":\"";
        let hash_start = expected
            .windows(marker.len())
            .position(|window| window == marker)
            .map(|position| position + marker.len())
            .expect("fixture request must contain the component hash");
        mask[hash_start..hash_start + 64].fill(0);

        let expected_offset = append_fixture_data(&mut data_segments, &mut cursor, &expected);
        let mask_offset = append_fixture_data(&mut data_segments, &mut cursor, &mask);
        let output_offset = append_fixture_data(&mut data_segments, &mut cursor, &output);
        layouts.push(RouteLayout {
            expected: expected_offset,
            mask: mask_offset,
            output: output_offset,
            input_len: expected.len(),
            output_len: output.len(),
        });
    }
    let fallback = append_fixture_data(&mut data_segments, &mut cursor, b"not-json");
    let heap_start = cursor.next_multiple_of(16);
    let memory_pages = heap_start.div_ceil(65_536) + 1;
    let route_body = layouts
        .iter()
        .map(|route| {
            format!(
                r#"local.get $request
                    local.get $request-len
                    i32.const {expected}
                    i32.const {mask}
                    i32.const {input_len}
                    call $matches
                    if
                        i32.const 0
                        i32.const {output}
                        i32.store
                        i32.const 0
                        i32.const {output_len}
                        i32.store offset=4
                        i32.const 0
                        return
                    end"#,
                expected = route.expected,
                mask = route.mask,
                input_len = route.input_len,
                output = route.output,
                output_len = route.output_len,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!(
        r#"(component
            (type $analyze-type (func (param "request" (list u8)) (result (list u8))))
            (core module $module
                (memory (export "memory") {memory_pages})
                {data_segments}
                (global $next (mut i32) (i32.const {heap_start}))
                (func (export "cabi_realloc")
                    (param $old i32) (param $old-size i32) (param $align i32) (param $new-size i32)
                    (result i32)
                    (local $result i32)
                    global.get $next
                    local.tee $result
                    local.get $new-size
                    i32.add
                    global.set $next
                    local.get $result)
                (func $matches
                    (param $request i32) (param $request-len i32)
                    (param $expected i32) (param $mask i32) (param $expected-len i32)
                    (result i32)
                    (local $index i32)
                    local.get $request-len
                    local.get $expected-len
                    i32.ne
                    if
                        i32.const 0
                        return
                    end
                    block $complete
                        loop $compare
                            local.get $index
                            local.get $request-len
                            i32.ge_u
                            br_if $complete
                            local.get $mask
                            local.get $index
                            i32.add
                            i32.load8_u
                            if
                                local.get $request
                                local.get $index
                                i32.add
                                i32.load8_u
                                local.get $expected
                                local.get $index
                                i32.add
                                i32.load8_u
                                i32.ne
                                if
                                    i32.const 0
                                    return
                                end
                            end
                            local.get $index
                            i32.const 1
                            i32.add
                            local.set $index
                            br $compare
                        end
                    end
                    i32.const 1)
                (func (export "analyze")
                    (param $request i32) (param $request-len i32) (result i32)
                    {route_body}
                    i32.const 0
                    i32.const {fallback}
                    i32.store
                    i32.const 0
                    i32.const 8
                    i32.store offset=4
                    i32.const 0))
            (core instance $instance (instantiate $module))
            (alias core export $instance "memory" (core memory $memory))
            (alias core export $instance "cabi_realloc" (core func $realloc))
            (alias core export $instance "analyze" (core func $analyze))
            (func $lifted (type $analyze-type)
                (canon lift (core func $analyze) (memory $memory) (realloc $realloc)))
            (export "analyze" (func $lifted)))"#
    );
    wat::parse_str(source).expect("routed fixture component WAT should parse")
}

fn append_fixture_data(output: &mut String, cursor: &mut usize, bytes: &[u8]) -> usize {
    let offset = *cursor;
    let escaped = bytes
        .iter()
        .map(|byte| format!("\\{byte:02x}"))
        .collect::<String>();
    writeln!(output, "(data (i32.const {offset}) \"{escaped}\")")
        .expect("writing fixture WAT to a string cannot fail");
    *cursor += bytes.len();
    offset
}

fn temp_root(label: &str) -> std::path::PathBuf {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "sifr-component-{label}-{}-{sequence}",
        std::process::id()
    ))
}

#[test]
fn hash_verification_is_exact() {
    let bytes = b"component";
    let hash = hex_digest(Sha256::digest(bytes).as_slice());
    verify_component_hash(&hash, bytes).expect("exact hash should pass");
    assert_eq!(
        verify_component_hash(&"0".repeat(64), bytes)
            .expect_err("hash drift must fail")
            .kind,
        ComponentErrorKind::Integrity
    );
}
