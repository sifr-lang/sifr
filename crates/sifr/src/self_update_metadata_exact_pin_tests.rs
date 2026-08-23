use super::{
    ChannelMetadata, PreviewChannel, PreviewVersion, TargetRequest, UpdateAction,
    resolve_update_plan,
    tests::{active_metadata, digest_evidence, metadata_payload, release},
};
use serde_json::json;

fn preview_metadata_with_stable_record() -> ChannelMetadata {
    let mut payload = metadata_payload();
    payload["releases"]["0.1.0"] = release(
        "stable",
        json!({
            "aarch64-apple-darwin": digest_evidence(),
            "x86_64-apple-darwin": digest_evidence(),
            "aarch64-unknown-linux-gnu": digest_evidence(),
            "x86_64-unknown-linux-gnu": digest_evidence()
        }),
    );
    ChannelMetadata::parse(&serde_json::to_string(&payload).expect("serialize metadata"))
        .expect("preview metadata with historical stable record parses")
}

#[test]
fn exact_cross_channel_version_requires_force() {
    let stable = TargetRequest::Version(PreviewVersion::parse("0.1.0").expect("stable"));
    let error = resolve_update_plan(
        "0.1.0-beta.2",
        "beta",
        stable.clone(),
        false,
        Some(&active_metadata()),
    )
    .expect_err("exact cross-channel pin requires force");
    assert!(error.message.contains("requires --force"));

    let plan = resolve_update_plan(
        "0.1.0-beta.2",
        "beta",
        stable,
        true,
        Some(&active_metadata()),
    )
    .expect("forced exact cross-channel pin resolves");
    assert_eq!(plan.action, UpdateAction::ChannelSwitch);
    assert_eq!(plan.requested_channel, None);
    assert_eq!(plan.resolved_channel, PreviewChannel::Stable);
}

#[test]
fn preview_metadata_rejects_exact_stable_record() {
    let stable = TargetRequest::Version(PreviewVersion::parse("0.1.0").expect("stable"));
    let error = resolve_update_plan(
        "0.1.0-beta.2",
        "beta",
        stable,
        true,
        Some(&preview_metadata_with_stable_record()),
    )
    .expect_err("preview metadata must reject exact stable selection");
    assert!(error.message.contains("require active GA metadata"));
}
