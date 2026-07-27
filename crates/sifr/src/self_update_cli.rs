use crate::cli_model_and_entrypoint::{
    diagnostic_with_code, DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG,
    EXIT_USER_DIAGNOSTIC,
};
use crate::diagnostic_rendering_and_run::render_diagnostics;
use crate::self_update_metadata::{
    fetch_channel_metadata, parse_channel, resolve_update_plan, PreviewChannel, PreviewVersion,
    TargetRequest, UpdateAction, UpdatePlan,
};
use crate::self_update_receipt::{
    discover_install_receipt, DiscoveredReceipt, ReceiptDiscoveryEnv,
};
use crate::self_update_runner::SelfUpdateRunner;
use clap::{Args, Subcommand, ValueEnum};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use std::io::{self, Write as _};

const SIFR_BUILD_VERSION: &str = env!("SIFR_BUILD_VERSION");

#[derive(Debug, Args)]
pub(crate) struct SelfArgs {
    #[command(subcommand)]
    command: SelfCommands,
}

#[derive(Debug, Subcommand)]
enum SelfCommands {
    /// Update a standalone Sifr installation
    Update(SelfUpdateArgs),
    /// Show standalone self-update receipt information
    Version(SelfVersionArgs),
}

#[derive(Debug, Args)]
struct SelfUpdateArgs {
    /// Resolve the latest version for a preview channel
    #[arg(long)]
    channel: Option<String>,
    /// Resolve one immutable preview version
    #[arg(long)]
    version: Option<String>,
    /// Print the plan without running the installer
    #[arg(long)]
    dry_run: bool,
    /// Format dry-run output
    #[arg(long, value_enum, default_value_t = SelfOutputFormat::Text)]
    format: SelfOutputFormat,
    /// Allow reinstall, downgrade, or channel switch
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Args)]
struct SelfVersionArgs {
    /// Print only the current executable version in text mode
    #[arg(long)]
    short: bool,
    /// Format version output
    #[arg(long, value_enum, default_value_t = SelfOutputFormat::Text)]
    format: SelfOutputFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum SelfOutputFormat {
    Text,
    Json,
}

pub(crate) fn cmd_self(args: &SelfArgs, diagnostic_format: DiagnosticFormat) -> i32 {
    match &args.command {
        SelfCommands::Update(update) => cmd_update(update, diagnostic_format),
        SelfCommands::Version(version) => cmd_version(version, diagnostic_format),
    }
}

fn cmd_update(args: &SelfUpdateArgs, diagnostic_format: DiagnosticFormat) -> i32 {
    if let Some(diagnostic) = update_args_diagnostic(args) {
        return render_usage_diagnostic(diagnostic, diagnostic_format);
    }

    let request = match target_request(args, diagnostic_format) {
        Ok(request) => request,
        Err(exit_code) => return exit_code,
    };
    let discovered = match discover_production_receipt(diagnostic_format) {
        Ok(discovered) => discovered,
        Err(exit_code) => return exit_code,
    };
    let metadata = match fetch_channel_metadata() {
        Ok(metadata) => metadata,
        Err(diagnostic) => {
            return render_user_error(diagnostic, diagnostic_format);
        }
    };
    let plan = match resolve_update_plan(
        &discovered.receipt.version,
        &discovered.receipt.channel,
        request,
        args.force,
        Some(&metadata),
    ) {
        Ok(plan) => plan,
        Err(diagnostic) => return render_user_error(diagnostic, diagnostic_format),
    };

    if args.dry_run {
        let output = render_dry_run(&plan, &discovered, args.format);
        let _ = writeln!(io::stdout(), "{output}");
        return EXIT_SUCCESS;
    }

    if plan.action == UpdateAction::NoOp {
        let _ = writeln!(
            io::stdout(),
            "Sifr {} is already installed at {}",
            plan.current_version.text,
            discovered.receipt.binary_path
        );
        return EXIT_SUCCESS;
    }

    match SelfUpdateRunner::production().run(&plan, &discovered) {
        Ok(exit_code) => exit_code,
        Err(error) => render_user_error_with_exit(
            error.diagnostic.as_ref(),
            diagnostic_format,
            error.exit_code,
        ),
    }
}

fn cmd_version(args: &SelfVersionArgs, diagnostic_format: DiagnosticFormat) -> i32 {
    if let Some(diagnostic) = version_args_diagnostic(args) {
        return render_usage_diagnostic(diagnostic, diagnostic_format);
    }
    let discovered = match discover_production_receipt(diagnostic_format) {
        Ok(discovered) => discovered,
        Err(exit_code) => return exit_code,
    };
    let output = render_version(&discovered, args.short, args.format);
    let _ = writeln!(io::stdout(), "{output}");
    EXIT_SUCCESS
}

fn update_args_diagnostic(args: &SelfUpdateArgs) -> Option<Box<RenderedDiagnostic>> {
    if args.channel.is_some() && args.version.is_some() {
        return Some(self_update_diagnostic(
            "--channel cannot be combined with --version for self-update",
        ));
    }
    if !args.dry_run && args.format != SelfOutputFormat::Text {
        return Some(self_update_diagnostic(
            "--format is accepted only with sifr self update --dry-run",
        ));
    }
    None
}

fn version_args_diagnostic(args: &SelfVersionArgs) -> Option<Box<RenderedDiagnostic>> {
    if args.short && args.format == SelfOutputFormat::Json {
        return Some(self_update_diagnostic(
            "sifr self version --short cannot be combined with --format json",
        ));
    }
    None
}

fn target_request(
    args: &SelfUpdateArgs,
    diagnostic_format: DiagnosticFormat,
) -> Result<TargetRequest, i32> {
    if let Some(channel) = &args.channel {
        return parse_channel(channel)
            .map(TargetRequest::Channel)
            .map_err(|diagnostic| render_user_error(diagnostic, diagnostic_format));
    }
    if let Some(version) = &args.version {
        return PreviewVersion::parse(version)
            .map(TargetRequest::Version)
            .map_err(|diagnostic| render_user_error(diagnostic, diagnostic_format));
    }
    Ok(TargetRequest::ReceiptChannel)
}

fn discover_production_receipt(
    diagnostic_format: DiagnosticFormat,
) -> Result<DiscoveredReceipt, i32> {
    let env = ReceiptDiscoveryEnv::production()
        .map_err(|diagnostic| render_user_error(diagnostic, diagnostic_format))?;
    discover_install_receipt(&env)
        .map_err(|diagnostic| render_user_error(diagnostic, diagnostic_format))
}

fn render_dry_run(
    plan: &UpdatePlan,
    discovered: &DiscoveredReceipt,
    format: SelfOutputFormat,
) -> String {
    match format {
        SelfOutputFormat::Text => render_dry_run_text(plan, discovered),
        SelfOutputFormat::Json => render_dry_run_json(plan, discovered),
    }
}

fn render_dry_run_text(plan: &UpdatePlan, discovered: &DiscoveredReceipt) -> String {
    let requested_channel = format_requested_channel(plan.requested_channel);
    [
        format!("current_version: {}", plan.current_version.text),
        format!("target_version: {}", plan.target_version.text),
        format!("receipt_channel: {}", plan.receipt_channel.as_str()),
        format!("requested_channel: {requested_channel}"),
        format!("resolved_channel: {}", plan.resolved_channel.as_str()),
        format!("install_dir: {}", discovered.receipt.install_dir),
        format!("binary_path: {}", discovered.receipt.binary_path),
        format!("sysroot_path: {}", discovered.receipt.sysroot_path),
        format!("installer_url: {}", plan.target_version.installer_url()),
        format!("action: {}", plan.action.as_str()),
        format!("force: {}", plan.force),
        format!("would_run_installer: {}", plan.action.would_run_installer()),
        "warnings: []".to_owned(),
    ]
    .join("\n")
}

fn render_dry_run_json(plan: &UpdatePlan, discovered: &DiscoveredReceipt) -> String {
    let requested_channel = json_optional_channel(plan.requested_channel);
    format!(
        "{{\n  \"schema_version\": 2,\n  \"current_version\": {},\n  \"target_version\": {},\n  \"receipt_channel\": {},\n  \"requested_channel\": {},\n  \"resolved_channel\": {},\n  \"install_dir\": {},\n  \"binary_path\": {},\n  \"sysroot_path\": {},\n  \"installer_url\": {},\n  \"action\": {},\n  \"force\": {},\n  \"would_run_installer\": {},\n  \"warnings\": []\n}}",
        json_string(&plan.current_version.text),
        json_string(&plan.target_version.text),
        json_string(plan.receipt_channel.as_str()),
        requested_channel,
        json_string(plan.resolved_channel.as_str()),
        json_string(&discovered.receipt.install_dir),
        json_string(&discovered.receipt.binary_path),
        json_string(&discovered.receipt.sysroot_path),
        json_string(&plan.target_version.installer_url()),
        json_string(plan.action.as_str()),
        plan.force,
        plan.action.would_run_installer(),
    )
}

fn render_version(discovered: &DiscoveredReceipt, short: bool, format: SelfOutputFormat) -> String {
    match format {
        SelfOutputFormat::Text if short => SIFR_BUILD_VERSION.to_owned(),
        SelfOutputFormat::Text => render_version_text(discovered),
        SelfOutputFormat::Json => render_version_json(discovered),
    }
}

fn render_version_text(discovered: &DiscoveredReceipt) -> String {
    [
        format!(
            "current_executable: {}",
            discovered.current_executable.display()
        ),
        format!("current_version: {SIFR_BUILD_VERSION}"),
        format!("receipt_version: {}", discovered.receipt.version),
        format!("receipt_path: {}", discovered.receipt_path.display()),
        format!("install_dir: {}", discovered.receipt.install_dir),
        format!("binary_path: {}", discovered.receipt.binary_path),
        format!("sysroot_path: {}", discovered.receipt.sysroot_path),
        format!(
            "sysroot_schema_version: {}",
            discovered.receipt.sysroot_schema_version
        ),
        format!(
            "sysroot_sifr_version: {}",
            discovered.receipt.sysroot_sifr_version
        ),
        format!(
            "sysroot_target_triple: {}",
            discovered.receipt.sysroot_target_triple
        ),
        format!("channel: {}", discovered.receipt.channel),
        format!("target: {}", discovered.receipt.target),
        format!("matches_receipt: {}", discovered.matches_receipt),
        "warnings: []".to_owned(),
    ]
    .join("\n")
}

fn render_version_json(discovered: &DiscoveredReceipt) -> String {
    format!(
        "{{\n  \"schema_version\": 2,\n  \"current_executable\": {},\n  \"current_version\": {},\n  \"receipt_version\": {},\n  \"install_dir\": {},\n  \"binary_path\": {},\n  \"sysroot_path\": {},\n  \"sysroot_schema_version\": {},\n  \"sysroot_sifr_version\": {},\n  \"sysroot_target_triple\": {},\n  \"channel\": {},\n  \"target\": {},\n  \"matches_receipt\": {},\n  \"warnings\": []\n}}",
        json_string(&discovered.current_executable.display().to_string()),
        json_string(SIFR_BUILD_VERSION),
        json_string(&discovered.receipt.version),
        json_string(&discovered.receipt.install_dir),
        json_string(&discovered.receipt.binary_path),
        json_string(&discovered.receipt.sysroot_path),
        discovered.receipt.sysroot_schema_version,
        json_string(&discovered.receipt.sysroot_sifr_version),
        json_string(&discovered.receipt.sysroot_target_triple),
        json_string(&discovered.receipt.channel),
        json_string(&discovered.receipt.target),
        discovered.matches_receipt,
    )
}

fn format_requested_channel(channel: Option<PreviewChannel>) -> &'static str {
    channel.map_or("null", PreviewChannel::as_str)
}

fn json_optional_channel(channel: Option<PreviewChannel>) -> String {
    channel.map_or_else(
        || "null".to_owned(),
        |channel| json_string(channel.as_str()),
    )
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_owned())
}

fn render_usage_diagnostic(
    diagnostic: Box<RenderedDiagnostic>,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let diagnostic = *diagnostic;
    let _ = render_diagnostics(&[diagnostic], diagnostic_format);
    EXIT_USAGE_OR_CONFIG
}

fn render_user_error(
    diagnostic: impl Into<Box<RenderedDiagnostic>>,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let diagnostic = diagnostic.into();
    let _ = render_diagnostics(std::slice::from_ref(diagnostic.as_ref()), diagnostic_format);
    EXIT_USER_DIAGNOSTIC
}

fn render_user_error_with_exit(
    diagnostic: &RenderedDiagnostic,
    diagnostic_format: DiagnosticFormat,
    exit_code: i32,
) -> i32 {
    let _ = render_diagnostics(std::slice::from_ref(diagnostic), diagnostic_format);
    exit_code
}

fn self_update_diagnostic(message: impl Into<String>) -> Box<RenderedDiagnostic> {
    Box::new(diagnostic_with_code(
        message,
        DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT,
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        render_dry_run_json, render_dry_run_text, render_version_json, render_version_text,
        update_args_diagnostic, version_args_diagnostic, SelfOutputFormat, SelfUpdateArgs,
        SelfVersionArgs,
    };
    use crate::self_update_metadata::{PreviewChannel, PreviewVersion, UpdateAction, UpdatePlan};
    use crate::self_update_receipt::{DiscoveredReceipt, InstallReceipt};
    use std::path::PathBuf;

    fn discovered() -> DiscoveredReceipt {
        DiscoveredReceipt {
            receipt: InstallReceipt {
                name: "sifr".to_owned(),
                version: "0.1.0-beta.1".to_owned(),
                channel: "beta".to_owned(),
                target: "aarch64-apple-darwin".to_owned(),
                install_dir: "/Users/example/.sifr/bin".to_owned(),
                binary_path: "/Users/example/.sifr/bin/sifr".to_owned(),
                sysroot_path: "/Users/example/.sifr".to_owned(),
                sysroot_schema_version: 1,
                sysroot_sifr_version: "0.1.0-beta.1".to_owned(),
                sysroot_target_triple: "aarch64-apple-darwin".to_owned(),
                sysroot_content_sha256:
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_owned(),
                artifact: "sifr-0.1.0-beta.1-aarch64-apple-darwin.tar.gz".to_owned(),
                modify_path: true,
            },
            receipt_path: PathBuf::from("/Users/example/.sifr/install.json"),
            current_executable: PathBuf::from("/Users/example/.sifr/bin/sifr"),
            matches_receipt: true,
        }
    }

    fn update_plan() -> UpdatePlan {
        UpdatePlan {
            current_version: PreviewVersion::parse("0.1.0-beta.1").unwrap(),
            target_version: PreviewVersion::parse("0.1.0-beta.2").unwrap(),
            receipt_channel: PreviewChannel::Beta,
            requested_channel: None,
            resolved_channel: PreviewChannel::Beta,
            action: UpdateAction::Update,
            force: false,
            installer_sha256: "d".repeat(64),
        }
    }

    fn plan_with_action(action: UpdateAction) -> UpdatePlan {
        let requested_channel =
            (action == UpdateAction::ChannelSwitch).then_some(PreviewChannel::Alpha);
        let resolved_channel = requested_channel.unwrap_or(PreviewChannel::Beta);
        let target_version = if action == UpdateAction::ChannelSwitch {
            PreviewVersion::parse("0.1.0-alpha.2").unwrap()
        } else {
            PreviewVersion::parse("0.1.0-beta.1").unwrap()
        };
        UpdatePlan {
            current_version: PreviewVersion::parse("0.1.0-beta.1").unwrap(),
            target_version,
            receipt_channel: PreviewChannel::Beta,
            requested_channel,
            resolved_channel,
            action,
            force: action != UpdateAction::NoOp,
            installer_sha256: "d".repeat(64),
        }
    }

    #[test]
    fn dry_run_json_is_deterministic() {
        let output = render_dry_run_json(&update_plan(), &discovered());
        assert_eq!(
            output,
            r#"{
  "schema_version": 2,
  "current_version": "0.1.0-beta.1",
  "target_version": "0.1.0-beta.2",
  "receipt_channel": "beta",
  "requested_channel": null,
  "resolved_channel": "beta",
  "install_dir": "/Users/example/.sifr/bin",
  "binary_path": "/Users/example/.sifr/bin/sifr",
  "sysroot_path": "/Users/example/.sifr",
  "installer_url": "https://github.com/sifr-lang/sifr/releases/download/0.1.0-beta.2/sifr-installer-0.1.0-beta.2",
  "action": "update",
  "force": false,
  "would_run_installer": true,
  "warnings": []
}"#
        );
    }

    #[test]
    fn dry_run_text_is_deterministic() {
        let output = render_dry_run_text(&update_plan(), &discovered());
        assert!(output.contains("current_version: 0.1.0-beta.1"));
        assert!(output.contains("requested_channel: null"));
        assert!(output.contains("would_run_installer: true"));
    }

    #[test]
    fn dry_run_json_no_op_has_false_installer_flag() {
        let output = render_dry_run_json(&plan_with_action(UpdateAction::NoOp), &discovered());
        assert!(output.contains("  \"action\": \"no_op\","));
        assert!(output.contains("  \"force\": false,"));
        assert!(output.contains("  \"would_run_installer\": false,"));
        assert!(output.contains("  \"requested_channel\": null,"));
    }

    #[test]
    fn dry_run_json_channel_switch_renders_requested_channel() {
        let output = render_dry_run_json(
            &plan_with_action(UpdateAction::ChannelSwitch),
            &discovered(),
        );
        assert!(output.contains("  \"target_version\": \"0.1.0-alpha.2\","));
        assert!(output.contains("  \"requested_channel\": \"alpha\","));
        assert!(output.contains("  \"resolved_channel\": \"alpha\","));
        assert!(output.contains("  \"action\": \"channel_switch\","));
        assert!(output.contains("  \"force\": true,"));
        assert!(output.contains("  \"would_run_installer\": true,"));
    }

    #[test]
    fn self_version_json_is_deterministic() {
        let output = render_version_json(&discovered());
        assert!(output.starts_with("{\n  \"schema_version\": 2,"));
        assert!(output.contains("  \"receipt_version\": \"0.1.0-beta.1\","));
        assert!(output.ends_with("  \"warnings\": []\n}"));
    }

    #[test]
    fn self_version_text_includes_receipt_match() {
        let output = render_version_text(&discovered());
        assert!(output.contains("matches_receipt: true"));
        assert!(output.contains("channel: beta"));
    }

    #[test]
    fn update_rejects_channel_with_version_before_receipt_discovery() {
        let diagnostic = update_args_diagnostic(&SelfUpdateArgs {
            channel: Some("beta".to_owned()),
            version: Some("0.1.0-beta.2".to_owned()),
            dry_run: true,
            format: SelfOutputFormat::Text,
            force: false,
        });

        assert!(diagnostic
            .expect("diagnostic")
            .message
            .contains("--channel"));
    }

    #[test]
    fn update_rejects_json_format_without_dry_run_before_receipt_discovery() {
        let diagnostic = update_args_diagnostic(&SelfUpdateArgs {
            channel: None,
            version: Some("0.1.0-beta.2".to_owned()),
            dry_run: false,
            format: SelfOutputFormat::Json,
            force: false,
        });

        assert!(diagnostic
            .expect("diagnostic")
            .message
            .contains("--dry-run"));
    }

    #[test]
    fn version_rejects_short_json_before_receipt_discovery() {
        let diagnostic = version_args_diagnostic(&SelfVersionArgs {
            short: true,
            format: SelfOutputFormat::Json,
        });

        assert!(diagnostic.expect("diagnostic").message.contains("--short"));
    }
}
