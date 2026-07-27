use super::{
    resolve_update_plan, tests::active_metadata, PreviewChannel, PreviewVersion, TargetRequest,
    UpdateAction,
};

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
