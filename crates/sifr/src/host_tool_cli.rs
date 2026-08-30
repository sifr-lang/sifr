use super::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_INTERNAL_COMPILER_FAILURE, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG,
    EXIT_USER_DIAGNOSTIC, diagnostic_with_code, package_diagnostic,
};
use super::diagnostic_rendering_and_run::render_diagnostics;
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::DiskSourceProvider;
use std::io::Write as _;
use std::process::Command;

pub(super) fn cmd_host_tool(words: &[String], diagnostic_format: DiagnosticFormat) -> i32 {
    let Some((namespace, forwarded)) = words.split_first() else {
        render_tool_error("missing tool namespace", diagnostic_format);
        return EXIT_USAGE_OR_CONFIG;
    };
    let current_dir = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            render_tool_error(
                &format!("cannot read the current directory: {error}"),
                diagnostic_format,
            );
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let mut provider = DiskSourceProvider::new();
    let snapshot = match sifr_package::load_package_graph_snapshot(
        &current_dir,
        sifr_package::CargoLockMode::Frozen,
        &mut provider,
    ) {
        Ok(snapshot) => snapshot,
        Err(failure) => {
            let diagnostics = failure
                .into_diagnostics()
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>();
            render_diagnostics(&diagnostics, diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let graph = match sifr_package::resolve_host_tool_graph(&snapshot, &mut provider) {
        Ok(graph) => graph,
        Err(errors) => {
            let diagnostics = errors
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>();
            render_diagnostics(&diagnostics, diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    if let Err(error) = sifr_package::verify_host_tool_graph(&graph, &mut provider) {
        render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
        return EXIT_USAGE_OR_CONFIG;
    }
    let plan = match graph.command_plan(namespace, env!("SIFR_BUILD_TARGET"), forwarded) {
        Ok(plan) => plan,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let provision_profile = match provision_profile(namespace, forwarded) {
        Ok(profile) => profile,
        Err(message) => {
            render_tool_error(&message, diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let capabilities = plan
        .capabilities
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(",");
    let output = match Command::new(&plan.program)
        .args(&plan.args)
        .current_dir(&plan.current_dir)
        .env("SIFR_TOOL_NAMESPACE", &plan.namespace)
        .env("SIFR_TOOL_CAPABILITIES", capabilities)
        .env("SIFR_TOOL_PACKAGE_CHECKSUM", &plan.package_checksum)
        .env(
            "SIFR_TOOL_LOCKFILE_FINGERPRINT",
            &graph.lockfile_fingerprint,
        )
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            render_tool_error(
                &format!("cannot execute tool namespace '{namespace}': {error}"),
                diagnostic_format,
            );
            return EXIT_INTERNAL_COMPILER_FAILURE;
        }
    };
    if !output.stderr.is_empty() {
        let _ = std::io::stderr().write_all(&output.stderr);
    }
    if !output.status.success() {
        if !output.stdout.is_empty() {
            let _ = std::io::stdout().write_all(&output.stdout);
        }
        return output.status.code().unwrap_or(EXIT_USER_DIAGNOSTIC);
    }
    if let Some(profile) = provision_profile {
        return render_connection_manifest(&output.stdout, &profile, namespace, diagnostic_format);
    }
    if std::io::stdout().write_all(&output.stdout).is_err() {
        return EXIT_INTERNAL_COMPILER_FAILURE;
    }
    EXIT_SUCCESS
}

fn provision_profile(namespace: &str, forwarded: &[String]) -> Result<Option<String>, String> {
    if namespace != "sql"
        || forwarded.first().map(String::as_str) != Some("test")
        || forwarded.get(1).map(String::as_str) != Some("provision")
    {
        return Ok(None);
    }
    let profiles = forwarded
        .windows(2)
        .filter(|pair| pair[0] == "--profile")
        .map(|pair| pair[1].clone())
        .collect::<Vec<_>>();
    if profiles.len() != 1 || profiles[0].is_empty() {
        return Err(
            "sifr sql test provision requires exactly one --profile <name> argument".to_string(),
        );
    }
    Ok(profiles.into_iter().next())
}

fn render_connection_manifest(
    bytes: &[u8],
    profile: &str,
    namespace: &str,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let source = match std::str::from_utf8(bytes) {
        Ok(source) => source,
        Err(error) => {
            render_tool_error(
                &format!("SQL test provision returned non-UTF-8 output: {error}"),
                diagnostic_format,
            );
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    let manifest = match sifr_sql_contract::TestConnectionManifest::from_json(source.trim()) {
        Ok(manifest) => manifest,
        Err(error) => {
            render_tool_error(&error.to_string(), diagnostic_format);
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    if manifest.profile != profile {
        render_tool_error(
            &format!(
                "SQL test provision returned profile '{}' for requested profile '{profile}'",
                manifest.profile
            ),
            diagnostic_format,
        );
        return EXIT_USER_DIAGNOSTIC;
    }
    if manifest.cleanup.tool_namespace != namespace {
        render_tool_error(
            "SQL test provision returned a cleanup namespace that differs from the selected tool",
            diagnostic_format,
        );
        return EXIT_USER_DIAGNOSTIC;
    }
    let canonical = match manifest.to_canonical_json() {
        Ok(canonical) => canonical,
        Err(error) => {
            render_tool_error(&error.to_string(), diagnostic_format);
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    let mut stdout = std::io::stdout();
    if writeln!(stdout, "{canonical}").is_err() {
        return EXIT_INTERNAL_COMPILER_FAILURE;
    }
    EXIT_SUCCESS
}

fn render_tool_error(message: &str, diagnostic_format: DiagnosticFormat) {
    render_diagnostics(
        &[diagnostic_with_code(
            message,
            DiagnosticCode::PACKAGE_METADATA_PARSE,
        )],
        diagnostic_format,
    );
}
