use super::*;

fn fixture(bytes: &[u8]) -> (ComponentRegistration, EmbeddedAnalysisRequest) {
    let mut r = request();
    r.component = identity(bytes);
    (registration(r.component.clone()), r)
}

fn owner_host(owner: &ComponentCompilation, limits: ComponentHostLimits) -> ComponentHost {
    ComponentHost::with_compilation(owner.clone(), limits, None).expect("compatible owner")
}

#[test]
fn compiled_reuse_executes_changed_requests_and_context_without_response_hits() {
    let bytes = include_bytes!("../../fixtures/words_component/words_component.wasm");
    let (registration, first) = fixture(bytes);
    let limits = ComponentHostLimits::default();
    let owner = ComponentCompilation::new(&limits).expect("owner");
    let mut second = first.clone();
    let TemplatePart::Static { text, .. } = &mut second.parts[0] else {
        panic!("static fixture")
    };
    *text = "world ".into();
    second.holes[0].ty = ClosedType::Int;
    second
        .context
        .semantic_profile
        .insert("mode".into(), "changed".into());
    for request in [&first, &second, &first] {
        // A different short-lived host must reuse the same compiled owner.
        let mut host = owner_host(&owner, limits.clone());
        let run = host
            .analyze(&registration, bytes, request)
            .expect("execute");
        assert_eq!(run.response, response_for_request(request));
        assert!(!run.cache_hit);
    }
    assert_eq!(owner.stats().0, 1);
    assert_eq!(owner.stats().1, 2);
}

#[test]
fn compiled_hits_revalidate_hash_registration_protocol_and_request_limits() {
    let bytes = fixture_component(&serde_json::to_vec(&response()).unwrap(), false);
    let (registration, request) = fixture(&bytes);
    let owner = ComponentCompilation::new(&ComponentHostLimits::default()).unwrap();
    let mut host = owner_host(&owner, ComponentHostLimits::default());
    host.analyze(&registration, &bytes, &request).unwrap();
    let stats = owner.stats();
    let mut corrupt = bytes.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(host.analyze(&registration, &corrupt, &request).is_err());
    let mut altered = registration.clone();
    altered.identity.processor.push_str(".different");
    assert_eq!(
        host.analyze(&altered, &bytes, &request).unwrap_err().kind,
        ComponentErrorKind::Registration
    );
    let mut wrong_protocol = request.clone();
    wrong_protocol.protocol_major += 1;
    assert_eq!(
        host.analyze(&registration, &bytes, &wrong_protocol)
            .unwrap_err()
            .kind,
        ComponentErrorKind::ProtocolVersion
    );
    let mut limited = owner_host(
        &owner,
        ComponentHostLimits {
            max_input_bytes: 1,
            ..ComponentHostLimits::default()
        },
    );
    assert_eq!(
        limited
            .analyze(&registration, &bytes, &request)
            .unwrap_err()
            .kind,
        ComponentErrorKind::ResourceLimit
    );
    assert_eq!(
        owner.stats(),
        stats,
        "all failures precede the compiled lookup"
    );
    host.analyze(&registration, &bytes, &request).unwrap();
    assert_eq!(owner.stats().1, 1);
}

#[test]
fn changed_artifacts_lru_eviction_and_mapped_image_budget_are_bounded() {
    let a = fixture_component(&serde_json::to_vec(&response()).unwrap(), false);
    let b = fixture_component(b"invalid output b", false);
    let c = fixture_component(b"invalid output c", false);
    let limits = ComponentHostLimits::default();
    let owner = ComponentCompilation::new(&limits).unwrap();
    owner.set_test_capacity(2, usize::MAX);
    let mut host = owner_host(&owner, limits.clone());
    for bytes in [&a, &b, &a, &c, &a, &b] {
        let (reg, req) = fixture(bytes);
        let _ = host.analyze(&reg, bytes, &req);
        assert!(owner.stats().2 <= 2);
    }
    assert_eq!((owner.stats().0, owner.stats().1), (4, 2));
    let sample_owner = ComponentCompilation::new(&limits).unwrap();
    let mut sample_host = owner_host(&sample_owner, limits.clone());
    let (reg, req) = fixture(&a);
    sample_host.analyze(&reg, &a, &req).unwrap();
    let image_bytes = sample_owner.stats().3;
    assert!(image_bytes > 1);
    for budget in [0, image_bytes - 1, image_bytes] {
        let bounded = ComponentCompilation::new(&limits).unwrap();
        bounded.set_test_capacity(8, budget);
        let mut host = owner_host(&bounded, limits.clone());
        for _ in 0..2 {
            host.analyze(&reg, &a, &req).unwrap();
        }
        let (compilations, hits, entries, retained) = bounded.stats();
        assert!(retained <= budget);
        if budget < image_bytes {
            assert_eq!((compilations, hits, entries), (2, 0, 0));
        } else {
            assert_eq!((compilations, hits, entries), (1, 1, 1));
        }
    }
}

#[test]
fn failed_compilation_and_forbidden_imports_are_never_retained() {
    let imports = wat::parse_str(r#"(component (import "forbidden" (func)))"#).unwrap();
    let limits = ComponentHostLimits::default();
    let owner = ComponentCompilation::new(&limits).unwrap();
    let mut host = owner_host(&owner, limits);
    for bytes in [&b"invalid wasm"[..], imports.as_slice()] {
        let (reg, req) = fixture(bytes);
        for _ in 0..2 {
            assert!(host.analyze(&reg, bytes, &req).is_err());
        }
        assert_eq!(owner.stats().2, 0);
    }
    assert_eq!((owner.stats().0, owner.stats().1), (4, 0));
}

#[test]
fn compiled_owner_rejects_incompatible_engine_policy() {
    let limits = ComponentHostLimits::default();
    let owner = ComponentCompilation::new(&limits).unwrap();
    let mut incompatible = limits.clone();
    incompatible.max_wasm_stack_bytes *= 2;
    assert!(ComponentHost::with_compilation(owner.clone(), incompatible, None).is_err());
    assert_eq!(owner.stats().0, 0);
    let _ = owner_host(&owner, limits);
}

#[test]
fn compiled_hits_keep_fresh_fuel_memory_and_output_limits() {
    let output = serde_json::to_vec(&response()).unwrap();
    let good = fixture_component(&output, false);
    let infinite = fixture_component(&[], true);
    let limits = ComponentHostLimits::default();
    let owner = ComponentCompilation::new(&limits).unwrap();
    let (reg, req) = fixture(&good);
    owner_host(&owner, limits.clone())
        .analyze(&reg, &good, &req)
        .unwrap();
    for _ in 0..2 {
        for restricted in [
            ComponentHostLimits {
                max_memory_bytes: 1024,
                ..limits.clone()
            },
            ComponentHostLimits {
                max_output_bytes: 8,
                ..limits.clone()
            },
        ] {
            assert_eq!(
                owner_host(&owner, restricted)
                    .analyze(&reg, &good, &req)
                    .unwrap_err()
                    .kind,
                ComponentErrorKind::ResourceLimit
            );
        }
    }
    let (reg, req) = fixture(&infinite);
    for _ in 0..2 {
        let mut host = owner_host(
            &owner,
            ComponentHostLimits {
                fuel: 10_000,
                ..limits.clone()
            },
        );
        assert_eq!(
            host.analyze(&reg, &infinite, &req).unwrap_err().kind,
            ComponentErrorKind::ResourceLimit
        );
    }
    assert_eq!((owner.stats().0, owner.stats().1), (2, 5));
}

#[test]
fn compiled_reuse_never_retains_mutated_guest_memory() {
    let output = serde_json::to_vec(&response()).unwrap();
    let escaped = output
        .iter()
        .map(|b| format!("\\{b:02x}"))
        .collect::<String>();
    let bytes = wat::parse_str(format!(
        r#"(component
        (core module $m
            (memory (export "memory") 1)
            (data (i32.const 32) "{escaped}")
            (func (export "realloc") (param i32 i32 i32 i32) (result i32) i32.const 8192)
            (func (export "analyze") (param i32 i32) (result i32)
                i32.const 8 i32.load if unreachable end
                i32.const 8 i32.const 1 i32.store
                i32.const 0 i32.const 32 i32.store
                i32.const 4 i32.const {} i32.store i32.const 0))
        (core instance $i (instantiate $m))
        (alias core export $i "memory" (core memory $memory))
        (alias core export $i "realloc" (core func $realloc))
        (alias core export $i "analyze" (core func $analyze))
        (func (export "analyze") (param "request" (list u8)) (result (list u8))
            (canon lift (core func $analyze) (memory $memory) (realloc $realloc))))"#,
        output.len()
    ))
    .unwrap();
    let (reg, req) = fixture(&bytes);
    let mut host = ComponentHost::new(ComponentHostLimits::default(), None).unwrap();
    for _ in 0..3 {
        let run = host
            .analyze(&reg, &bytes, &req)
            .expect("fresh guest memory");
        assert_eq!(run.response, response());
        assert!(!run.cache_hit);
    }
    assert_eq!(
        (host.compilation().stats().0, host.compilation().stats().1),
        (1, 2)
    );
}
