use super::{InteropBuildPlan, RustBridgeSourceDigest, RustInteropCargoInputs};

fn fixture() -> RustInteropCargoInputs {
    RustInteropCargoInputs {
        package_id: "package".into(),
        cargo_metadata_digest: Some("metadata".into()),
        sqlx_offline_metadata_digest: Some("sqlx".into()),
        package_graph_digest: Some("graph".into()),
        package_source_map_digest: Some("source".into()),
        cargo_lock_digest: Some("lock".into()),
        target_triple: Some("target".into()),
        target_features: vec!["a".into()],
        cargo_profile: "release".into(),
        panic_strategy: Some("abort".into()),
        profile_codegen_settings: vec![("opt".into(), "3".into())],
        cargo_version: Some("cargo".into()),
        rustc_version: Some("rustc".into()),
        trust_policy_digest: "trust".into(),
        declared_build_env: vec!["ENV".into()],
    }
}

fn fragment(cargo: RustInteropCargoInputs) -> String {
    let mut plan = InteropBuildPlan::default();
    plan.rust.cargo_inputs = Some(cargo);
    plan.cache_key_fragment()
}

#[test]
fn cargo_fragment_binds_every_semantic_field() {
    let original = fixture();
    let baseline = fragment(original.clone());
    let mut cases = Vec::new();
    macro_rules! mutate {
        ($field:ident, $value:expr) => {{
            let mut changed = original.clone();
            changed.$field = $value;
            cases.push((stringify!($field), changed));
        }};
    }
    mutate!(package_id, "other".into());
    mutate!(cargo_metadata_digest, Some("other".into()));
    mutate!(sqlx_offline_metadata_digest, Some("other".into()));
    mutate!(package_graph_digest, Some("other".into()));
    mutate!(package_source_map_digest, Some("other".into()));
    mutate!(cargo_lock_digest, Some("other".into()));
    mutate!(target_triple, Some("other".into()));
    mutate!(target_features, vec!["other".into()]);
    mutate!(cargo_profile, "other".into());
    mutate!(panic_strategy, Some("other".into()));
    mutate!(profile_codegen_settings, vec![("other".into(), "3".into())]);
    mutate!(cargo_version, Some("other".into()));
    mutate!(rustc_version, Some("other".into()));
    mutate!(trust_policy_digest, "other".into());
    mutate!(declared_build_env, vec!["other".into()]);
    for (name, changed) in cases {
        assert_ne!(baseline, fragment(changed), "{name}");
    }

    let mut absent = original.clone();
    absent.cargo_metadata_digest = None;
    let none = fragment(absent.clone());
    absent.cargo_metadata_digest = Some(String::new());
    assert_ne!(none, fragment(absent));
    let mut comma = original.clone();
    comma.target_features = vec!["a,b".into()];
    let mut split = original;
    split.target_features = vec!["a".into(), "b".into()];
    assert_ne!(fragment(comma), fragment(split));
}

#[test]
fn bridge_fragment_binds_package_root_and_digest() {
    let mut plan = InteropBuildPlan::default();
    plan.rust.bridge_sources.push(RustBridgeSourceDigest {
        package_id: "package".into(),
        bridge_root: "bridge".into(),
        digest: "digest".into(),
    });
    let baseline = plan.cache_key_fragment();
    for field in 0..3 {
        let mut changed = plan.clone();
        let source = &mut changed.rust.bridge_sources[0];
        match field {
            0 => source.package_id.push('x'),
            1 => source.bridge_root.push('x'),
            _ => source.digest.push('x'),
        }
        assert_ne!(baseline, changed.cache_key_fragment());
    }
}
