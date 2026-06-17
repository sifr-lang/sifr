use crate::cli_model_and_entrypoint::diagnostic_with_code;
use serde_json::Value;
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::process::Command;

pub(crate) const INSTALL_BASE_URL: &str = "https://sifr.sh/install";
const CHANNEL_METADATA_URL: &str = "https://sifr.sh/install/metadata/channels.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreviewChannel {
    Alpha,
    Beta,
}

impl PreviewChannel {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Beta => "beta",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreviewVersion {
    pub(crate) text: String,
    pub(crate) channel: PreviewChannel,
    major: u64,
    minor: u64,
    patch: u64,
    prerelease_number: u64,
}

impl PreviewVersion {
    pub(crate) fn parse(input: &str) -> Result<Self, Box<RenderedDiagnostic>> {
        if input.contains("-rc.") {
            return Err(self_update_diagnostic(format!(
                "release-candidate version pins are disabled while stable release channels are disabled: {input}; use --channel alpha|beta or a supported preview version"
            )));
        }
        if is_stable_version(input) {
            return Err(self_update_diagnostic(format!(
                "stable-looking versions are disabled while stable release channels are disabled: {input}; use --channel alpha|beta or --version <preview>"
            )));
        }
        let (core, prerelease) = input.split_once('-').ok_or_else(|| {
            self_update_diagnostic(format!(
                "version must be an alpha or beta semver prerelease: {input}"
            ))
        })?;
        let mut core_parts = core.split('.');
        let major = parse_number(core_parts.next(), input)?;
        let minor = parse_number(core_parts.next(), input)?;
        let patch = parse_number(core_parts.next(), input)?;
        if core_parts.next().is_some() {
            return Err(invalid_preview_version(input));
        }
        let (label, number) = prerelease
            .split_once('.')
            .ok_or_else(|| invalid_preview_version(input))?;
        let channel = match label {
            "alpha" => PreviewChannel::Alpha,
            "beta" => PreviewChannel::Beta,
            "rc" => {
                return Err(self_update_diagnostic(format!(
                    "release-candidate version pins are disabled while stable release channels are disabled: {input}; use --channel alpha|beta or a supported preview version"
                )));
            }
            _ => return Err(invalid_preview_version(input)),
        };
        let prerelease_number = number
            .parse::<u64>()
            .map_err(|_| invalid_preview_version(input))?;
        Ok(Self {
            text: input.to_owned(),
            channel,
            major,
            minor,
            patch,
            prerelease_number,
        })
    }

    pub(crate) fn installer_url(&self) -> String {
        format!("{INSTALL_BASE_URL}/versions/{}", self.text)
    }

    pub(crate) fn cmp_version(&self, other: &Self) -> std::cmp::Ordering {
        (
            self.major,
            self.minor,
            self.patch,
            channel_rank(self.channel),
            self.prerelease_number,
        )
            .cmp(&(
                other.major,
                other.minor,
                other.patch,
                channel_rank(other.channel),
                other.prerelease_number,
            ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UpdateAction {
    NoOp,
    Update,
    Reinstall,
    Downgrade,
    ChannelSwitch,
}

impl UpdateAction {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NoOp => "no_op",
            Self::Update => "update",
            Self::Reinstall => "reinstall",
            Self::Downgrade => "downgrade",
            Self::ChannelSwitch => "channel_switch",
        }
    }

    pub(crate) fn would_run_installer(self) -> bool {
        self != Self::NoOp
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UpdatePlan {
    pub(crate) current_version: PreviewVersion,
    pub(crate) target_version: PreviewVersion,
    pub(crate) receipt_channel: PreviewChannel,
    pub(crate) requested_channel: Option<PreviewChannel>,
    pub(crate) resolved_channel: PreviewChannel,
    pub(crate) action: UpdateAction,
    pub(crate) force: bool,
}

#[derive(Clone, Debug)]
pub(crate) enum TargetRequest {
    ReceiptChannel,
    Channel(PreviewChannel),
    Version(PreviewVersion),
}

#[derive(Debug, Clone)]
pub(crate) struct ChannelMetadata {
    channels: BTreeMap<String, PreviewVersion>,
}

impl ChannelMetadata {
    pub(crate) fn parse(input: &str) -> Result<Self, Box<RenderedDiagnostic>> {
        let value = serde_json::from_str::<Value>(input).map_err(|error| {
            self_update_diagnostic(format!(
                "self-update channel metadata is malformed: {error}"
            ))
        })?;
        let object = value.as_object().ok_or_else(|| {
            self_update_diagnostic("self-update channel metadata must be a JSON object")
        })?;
        if object.len() != 2
            || !object.contains_key("schema_version")
            || !object.contains_key("channels")
        {
            return Err(self_update_diagnostic(
                "self-update channel metadata contains unsupported fields",
            ));
        }
        if object.get("schema_version").and_then(Value::as_u64) != Some(1) {
            return Err(self_update_diagnostic(
                "self-update channel metadata schema_version must be 1",
            ));
        }
        let channels = object
            .get("channels")
            .and_then(Value::as_object)
            .ok_or_else(|| self_update_diagnostic("self-update channels must be an object"))?;
        let mut parsed = BTreeMap::new();
        for (name, version_value) in channels {
            if name == "stable" {
                return Err(self_update_diagnostic(
                    "stable channel metadata is disabled while stable release channels are disabled",
                ));
            }
            if name == "rc" {
                return Err(self_update_diagnostic(
                    "release-candidate channel metadata is disabled while stable release channels are disabled",
                ));
            }
            if name != "alpha" && name != "beta" {
                return Err(self_update_diagnostic(format!(
                    "unknown self-update channel in metadata: {name}"
                )));
            }
            let version = version_value.as_str().ok_or_else(|| {
                self_update_diagnostic(format!("metadata channel {name} must map to a version"))
            })?;
            let parsed_version = PreviewVersion::parse(version)?;
            if parsed_version.channel.as_str() != name {
                return Err(self_update_diagnostic(format!(
                    "metadata channel {name} points at {}",
                    parsed_version.text
                )));
            }
            parsed.insert(name.clone(), parsed_version);
        }
        if !parsed.contains_key("alpha") || !parsed.contains_key("beta") {
            return Err(self_update_diagnostic(
                "self-update metadata must contain alpha and beta channels",
            ));
        }
        Ok(Self { channels: parsed })
    }

    pub(crate) fn resolve_channel(
        &self,
        channel: PreviewChannel,
    ) -> Result<PreviewVersion, Box<RenderedDiagnostic>> {
        self.channels
            .get(channel.as_str())
            .cloned()
            .ok_or_else(|| self_update_diagnostic("requested channel is missing from metadata"))
    }
}

pub(crate) fn parse_channel(input: &str) -> Result<PreviewChannel, Box<RenderedDiagnostic>> {
    match input {
        "alpha" => Ok(PreviewChannel::Alpha),
        "beta" => Ok(PreviewChannel::Beta),
        "stable" => Err(self_update_diagnostic(
            "stable channel self-update is disabled while stable release channels are disabled; use --channel alpha|beta",
        )),
        "rc" => Err(self_update_diagnostic(
            "release-candidate channel self-update is disabled while stable release channels are disabled; use --channel alpha|beta",
        )),
        other => Err(self_update_diagnostic(format!(
            "unknown self-update channel: {other}; use --channel alpha|beta"
        ))),
    }
}

pub(crate) fn resolve_update_plan(
    current_version: &str,
    receipt_channel: &str,
    request: TargetRequest,
    force: bool,
    metadata: Option<&ChannelMetadata>,
) -> Result<UpdatePlan, Box<RenderedDiagnostic>> {
    let current_version = PreviewVersion::parse(current_version)?;
    let receipt_channel = parse_channel(receipt_channel)?;
    let (target_version, requested_channel, resolved_channel) = match request {
        TargetRequest::ReceiptChannel => {
            let metadata = metadata.ok_or_else(|| {
                self_update_diagnostic(
                    "self-update channel metadata is required for latest resolution",
                )
            })?;
            let target_version = metadata.resolve_channel(receipt_channel)?;
            (target_version, None, receipt_channel)
        }
        TargetRequest::Channel(channel) => {
            let metadata = metadata.ok_or_else(|| {
                self_update_diagnostic(
                    "self-update channel metadata is required for channel resolution",
                )
            })?;
            let target_version = metadata.resolve_channel(channel)?;
            (target_version, Some(channel), channel)
        }
        TargetRequest::Version(version) => {
            let resolved_channel = version.channel;
            (version, None, resolved_channel)
        }
    };

    if let Some(requested_channel) = requested_channel {
        if requested_channel != receipt_channel && !force {
            return Err(self_update_diagnostic(format!(
                "switching self-update channel from {} to {} requires --force",
                receipt_channel.as_str(),
                requested_channel.as_str()
            )));
        }
    }

    let action = if requested_channel.is_some_and(|channel| channel != receipt_channel) {
        UpdateAction::ChannelSwitch
    } else {
        match target_version.cmp_version(&current_version) {
            Ordering::Equal if force => UpdateAction::Reinstall,
            Ordering::Equal => UpdateAction::NoOp,
            Ordering::Greater => UpdateAction::Update,
            Ordering::Less if force => UpdateAction::Downgrade,
            Ordering::Less => {
                return Err(self_update_diagnostic(format!(
                    "downgrading self-update from {} to {} requires --force",
                    current_version.text, target_version.text
                )));
            }
        }
    };

    Ok(UpdatePlan {
        current_version,
        target_version,
        receipt_channel,
        requested_channel,
        resolved_channel,
        action,
        force,
    })
}

pub(crate) fn fetch_channel_metadata() -> Result<ChannelMetadata, Box<RenderedDiagnostic>> {
    let output = Command::new("curl")
        .args(["-fsSL", CHANNEL_METADATA_URL])
        .output()
        .map_err(|error| {
            self_update_diagnostic(format!(
                "could not run curl to fetch self-update metadata: {error}"
            ))
        })?;
    if !output.status.success() {
        return Err(self_update_diagnostic(format!(
            "self-update metadata unavailable at {CHANNEL_METADATA_URL}"
        )));
    }
    let text = String::from_utf8(output.stdout).map_err(|error| {
        self_update_diagnostic(format!("self-update metadata was not UTF-8: {error}"))
    })?;
    ChannelMetadata::parse(&text)
}

pub(crate) fn self_update_diagnostic(message: impl Into<String>) -> Box<RenderedDiagnostic> {
    Box::new(diagnostic_with_code(
        message,
        DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT,
    ))
}

fn parse_number(value: Option<&str>, original: &str) -> Result<u64, Box<RenderedDiagnostic>> {
    value
        .ok_or_else(|| invalid_preview_version(original))?
        .parse::<u64>()
        .map_err(|_| invalid_preview_version(original))
}

fn invalid_preview_version(input: &str) -> Box<RenderedDiagnostic> {
    self_update_diagnostic(format!(
        "version must be an alpha or beta semver prerelease: {input}"
    ))
}

fn is_stable_version(input: &str) -> bool {
    let mut parts = input.split('.');
    let Some(major) = parts.next() else {
        return false;
    };
    let Some(minor) = parts.next() else {
        return false;
    };
    let Some(patch) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && major.chars().all(|ch| ch.is_ascii_digit())
        && minor.chars().all(|ch| ch.is_ascii_digit())
        && patch.chars().all(|ch| ch.is_ascii_digit())
}

fn channel_rank(channel: PreviewChannel) -> u8 {
    match channel {
        PreviewChannel::Alpha => 1,
        PreviewChannel::Beta => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_channel, resolve_update_plan, ChannelMetadata, PreviewChannel, PreviewVersion,
        TargetRequest, UpdateAction,
    };

    fn metadata() -> ChannelMetadata {
        ChannelMetadata::parse(
            r#"{
  "schema_version": 1,
  "channels": {
    "alpha": "0.1.0-alpha.2",
    "beta": "0.1.0-beta.2"
  }
}"#,
        )
        .expect("metadata parses")
    }

    #[test]
    fn parses_channel_metadata() {
        let metadata = metadata();
        assert_eq!(
            metadata
                .resolve_channel(parse_channel("beta").expect("beta"))
                .expect("version")
                .text,
            "0.1.0-beta.2"
        );
    }

    #[test]
    fn receipt_channel_request_uses_receipt_channel_metadata() {
        let plan = resolve_update_plan(
            "0.1.0-beta.1",
            "beta",
            TargetRequest::ReceiptChannel,
            false,
            Some(&metadata()),
        )
        .expect("receipt-channel plan resolves");

        assert_eq!(plan.target_version.text, "0.1.0-beta.2");
        assert_eq!(plan.requested_channel, None);
        assert_eq!(plan.resolved_channel, PreviewChannel::Beta);
        assert_eq!(plan.action, UpdateAction::Update);
    }

    #[test]
    fn same_version_is_no_op_without_force() {
        let plan = resolve_update_plan(
            "0.1.0-beta.2",
            "beta",
            TargetRequest::Version(PreviewVersion::parse("0.1.0-beta.2").expect("version")),
            false,
            None,
        )
        .expect("same-version plan is allowed as a no-op");
        assert_eq!(plan.action, UpdateAction::NoOp);
        assert!(!plan.action.would_run_installer());
    }

    #[test]
    fn same_version_reinstall_requires_force() {
        let plan = resolve_update_plan(
            "0.1.0-beta.2",
            "beta",
            TargetRequest::Version(PreviewVersion::parse("0.1.0-beta.2").expect("version")),
            true,
            None,
        )
        .expect("forced same-version plan is a reinstall");
        assert_eq!(plan.action, UpdateAction::Reinstall);
    }

    #[test]
    fn downgrade_requires_force() {
        let error = resolve_update_plan(
            "0.1.0-beta.2",
            "beta",
            TargetRequest::Version(PreviewVersion::parse("0.1.0-beta.1").expect("version")),
            false,
            None,
        )
        .expect_err("downgrade requires force");
        assert!(error.message.contains("requires --force"));

        let plan = resolve_update_plan(
            "0.1.0-beta.2",
            "beta",
            TargetRequest::Version(PreviewVersion::parse("0.1.0-beta.1").expect("version")),
            true,
            None,
        )
        .expect("forced downgrade is allowed");
        assert_eq!(plan.action, UpdateAction::Downgrade);
    }

    #[test]
    fn channel_switch_requires_force() {
        let error = resolve_update_plan(
            "0.1.0-beta.1",
            "beta",
            TargetRequest::Channel(PreviewChannel::Alpha),
            false,
            Some(&metadata()),
        )
        .expect_err("channel switch requires force");
        assert!(error.message.contains("requires --force"));

        let plan = resolve_update_plan(
            "0.1.0-beta.1",
            "beta",
            TargetRequest::Channel(PreviewChannel::Alpha),
            true,
            Some(&metadata()),
        )
        .expect("forced channel switch is allowed");
        assert_eq!(plan.action, UpdateAction::ChannelSwitch);
    }

    #[test]
    fn rejects_stable_and_rc_versions() {
        assert!(PreviewVersion::parse("0.1.0").is_err());
        assert!(PreviewVersion::parse("0.1.0-rc.1").is_err());
    }

    #[test]
    fn rejects_rc_channel_without_stable_release_channel() {
        let error = parse_channel("rc").expect_err("rc channel is gated");
        assert!(error.message.contains("release-candidate"));
    }

    #[test]
    fn rejects_stable_metadata() {
        assert!(ChannelMetadata::parse(
            r#"{"schema_version":1,"channels":{"alpha":"0.1.0-alpha.1","beta":"0.1.0-beta.1","stable":"1.0.0"}}"#,
        )
        .is_err());
    }

    #[test]
    fn rejects_unknown_metadata_channel() {
        assert!(ChannelMetadata::parse(
            r#"{"schema_version":1,"channels":{"alpha":"0.1.0-alpha.1","beta":"0.1.0-beta.1","nightly":"0.1.0-alpha.2"}}"#,
        )
        .is_err());
    }
}
