use super::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_INTERNAL_COMPILER_FAILURE, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG,
    EXIT_USER_DIAGNOSTIC, diagnostic_with_code, package_diagnostic,
};
use super::diagnostic_rendering_and_run::render_diagnostics;
use super::host_tool_sandbox::{SandboxedToolRequest, run_sandboxed_tool};
use sha2::{Digest, Sha256};
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::DiskSourceProvider;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn cmd_host_tools_lock(check: bool, format: DiagnosticFormat) -> i32 {
    let (graph, mut provider) = match load_graph(format) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let result = if check {
        sifr_package::load_host_tool_lock(&graph, &mut provider).map(drop)
    } else {
        sifr_package::write_host_tool_lock(&graph).map(drop)
    };
    result.map_or_else(
        |error| {
            render_diagnostics(&[package_diagnostic(error)], format);
            EXIT_USAGE_OR_CONFIG
        },
        |()| EXIT_SUCCESS,
    )
}

pub(super) fn cmd_host_tool(words: &[String], format: DiagnosticFormat) -> i32 {
    let Some((namespace, forwarded)) = words.split_first() else {
        render_tool_error("missing tool namespace", format);
        return EXIT_USAGE_OR_CONFIG;
    };
    let (graph, mut provider) = match load_graph(format) {
        Ok(value) => value,
        Err(code) => return code,
    };
    if let Err(error) = sifr_package::verify_host_tool_graph(&graph, &mut provider)
        .and_then(|()| sifr_package::load_host_tool_lock(&graph, &mut provider).map(drop))
    {
        render_diagnostics(&[package_diagnostic(error)], format);
        return EXIT_USAGE_OR_CONFIG;
    }
    let plan = match graph.build_plan(namespace, env!("SIFR_BUILD_TARGET")) {
        Ok(plan) => plan,
        Err(error) => {
            if let Some(suggestion) = builtin_typo_suggestion(namespace) {
                render_tool_error(
                    &format!("unknown command '{namespace}'; did you mean '{suggestion}'?"),
                    format,
                );
                return EXIT_USAGE_OR_CONFIG;
            }
            render_diagnostics(&[package_diagnostic(error)], format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let provision_profile = match provision_profile(namespace, forwarded) {
        Ok(profile) => profile,
        Err(message) => {
            render_tool_error(&message, format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let cargo = match resolve_program("cargo") {
        Ok(path) => path,
        Err(message) => {
            render_tool_error(&message, format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let rustc = match resolve_program("rustc") {
        Ok(path) => path,
        Err(message) => {
            render_tool_error(&message, format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let executable = match build_tool(&cargo, &rustc, &graph.target_directory, &plan) {
        Ok(path) => path,
        Err(message) => {
            render_tool_error(&message, format);
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    let executable_hash = match sha256_file(&executable) {
        Ok(hash) => hash,
        Err(message) => {
            render_tool_error(&message, format);
            return EXIT_INTERNAL_COMPILER_FAILURE;
        }
    };
    // Re-resolve mutable inputs after the build to narrow the build/execute race.
    let (observed, mut observed_provider) = match load_graph(format) {
        Ok(value) => value,
        Err(code) => return code,
    };
    if observed != graph
        || sifr_package::load_host_tool_lock(&observed, &mut observed_provider).is_err()
    {
        render_tool_error(
            "host-tool inputs changed while the executable was built; retry after reviewing `sifr tools lock`",
            format,
        );
        return EXIT_USAGE_OR_CONFIG;
    }
    let output = match run_sandboxed_tool(&SandboxedToolRequest {
        executable: &executable,
        args: forwarded,
        workspace_root: &graph.workspace_root,
        namespace,
        capabilities: &plan.capabilities,
        package_checksum: &plan.package_checksum,
        lockfile_fingerprint: &graph.lockfile_fingerprint,
        executable_hash: &executable_hash,
        stream_output: provision_profile.is_none(),
    }) {
        Ok(output) => output,
        Err(message) => {
            render_tool_error(&message, format);
            return EXIT_INTERNAL_COMPILER_FAILURE;
        }
    };
    if output.output_exceeded_limit {
        render_tool_error("host-tool output exceeded the 10 MiB limit", format);
        return EXIT_USER_DIAGNOSTIC;
    }
    if !output.streamed && !output.stderr.is_empty() {
        let _ = std::io::stderr().write_all(&output.stderr);
    }
    if !output.status.success() {
        if !output.streamed && !output.stdout.is_empty() {
            let _ = std::io::stdout().write_all(&output.stdout);
        }
        return output.status.code().unwrap_or(EXIT_USER_DIAGNOSTIC);
    }
    if let Some(profile) = provision_profile {
        return render_connection_manifest(&output.stdout, &profile, namespace, format);
    }
    if !output.streamed && std::io::stdout().write_all(&output.stdout).is_err() {
        return EXIT_INTERNAL_COMPILER_FAILURE;
    }
    EXIT_SUCCESS
}

fn load_graph(
    format: DiagnosticFormat,
) -> Result<(sifr_package::HostToolGraph, DiskSourceProvider), i32> {
    let current_dir = std::env::current_dir().map_err(|error| {
        render_tool_error(
            &format!("cannot read the current directory: {error}"),
            format,
        );
        EXIT_USAGE_OR_CONFIG
    })?;
    let mut provider = DiskSourceProvider::new();
    let snapshot = sifr_package::load_package_graph_snapshot(
        &current_dir,
        sifr_package::CargoLockMode::Frozen,
        &mut provider,
    )
    .map_err(|failure| {
        render_diagnostics(
            &failure
                .into_diagnostics()
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>(),
            format,
        );
        EXIT_USAGE_OR_CONFIG
    })?;
    let graph =
        sifr_package::resolve_host_tool_graph(&snapshot, &mut provider).map_err(|errors| {
            render_diagnostics(
                &errors
                    .into_iter()
                    .map(package_diagnostic)
                    .collect::<Vec<_>>(),
                format,
            );
            EXIT_USAGE_OR_CONFIG
        })?;
    Ok((graph, provider))
}

fn resolve_program(name: &str) -> Result<PathBuf, String> {
    let path = std::env::var_os("PATH").ok_or_else(|| "PATH is not set".to_string())?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join(name);
        if candidate.is_file() {
            return std::path::absolute(&candidate).map_err(|error| {
                format!(
                    "cannot make program '{}' absolute: {error}",
                    candidate.display()
                )
            });
        }
    }
    Err(format!("cannot find required program '{name}' on PATH"))
}

fn build_tool(
    cargo: &Path,
    rustc: &Path,
    target_directory: &Path,
    plan: &sifr_package::HostToolBuildPlan,
) -> Result<PathBuf, String> {
    let output = Command::new(cargo)
        .args(&plan.args)
        .arg("--config")
        .arg("build.rustc-wrapper=\"\"")
        .arg("--config")
        .arg("build.rustc-workspace-wrapper=\"\"")
        .arg("--config")
        .arg("build.rustflags=[]")
        .arg("--config")
        .arg(format!("target.{}.rustflags=[]", env!("SIFR_BUILD_TARGET")))
        .current_dir(&plan.current_dir)
        .env("CARGO_TARGET_DIR", target_directory)
        .env("RUSTC", rustc)
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("CARGO_BUILD_RUSTC")
        .output()
        .map_err(|error| format!("cannot build host tool with '{}': {error}", cargo.display()))?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let separator = if !stderr.is_empty() && !stdout.is_empty() {
            "\n"
        } else {
            ""
        };
        return Err(format!(
            "host-tool build failed: {}{}{}",
            stderr.trim(),
            separator,
            stdout.trim(),
        ));
    }
    let executable = target_directory
        .join(env!("SIFR_BUILD_TARGET"))
        .join("debug")
        .join(format!(
            "{}{}",
            plan.entrypoint,
            std::env::consts::EXE_SUFFIX
        ));
    std::fs::canonicalize(&executable).map_err(|error| {
        format!(
            "host-tool build did not produce '{}': {error}",
            executable.display()
        )
    })
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| {
        format!(
            "cannot hash host-tool executable '{}': {error}",
            path.display()
        )
    })?;
    Ok(Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut encoded, byte| {
            let _ = write!(encoded, "{byte:02x}");
            encoded
        }))
}

fn provision_profile(namespace: &str, forwarded: &[String]) -> Result<Option<String>, String> {
    if namespace != "sql"
        || forwarded.first().map(String::as_str) != Some("test")
        || forwarded.get(1).map(String::as_str) != Some("provision")
    {
        return Ok(None);
    }
    let mut profiles = Vec::new();
    let mut index = 2;
    while index < forwarded.len() {
        if forwarded[index] == "--" {
            break;
        }
        if forwarded[index] == "--profile" {
            profiles.push(forwarded.get(index + 1).cloned().unwrap_or_default());
            index += 2;
        } else if let Some(value) = forwarded[index].strip_prefix("--profile=") {
            profiles.push(value.to_string());
            index += 1;
        } else {
            index += 1;
        }
    }
    if profiles.len() != 1 || profiles[0].is_empty() {
        return Err(
            "sifr sql test provision requires exactly one --profile <name> argument".into(),
        );
    }
    Ok(profiles.into_iter().next())
}

fn builtin_typo_suggestion(word: &str) -> Option<&'static str> {
    sifr_package::RESERVED_TOOL_NAMESPACES
        .iter()
        .copied()
        .filter(|candidate| {
            edit_distance(word, candidate) <= 1 || adjacent_transposition(word, candidate)
        })
        .min_by_key(|candidate| edit_distance(word, candidate))
}

fn adjacent_transposition(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let differences = left
        .iter()
        .zip(right)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    differences.len() == 2
        && differences[1] == differences[0] + 1
        && left[differences[0]] == right[differences[1]]
        && left[differences[1]] == right[differences[0]]
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut row = (0..=right.len()).collect::<Vec<_>>();
    for (left_index, left_byte) in left.bytes().enumerate() {
        let mut diagonal = row[0];
        row[0] = left_index + 1;
        for (right_index, right_byte) in right.bytes().enumerate() {
            let above = row[right_index + 1];
            row[right_index + 1] = if left_byte == right_byte {
                diagonal
            } else {
                1 + diagonal.min(above).min(row[right_index])
            };
            diagonal = above;
        }
    }
    row[right.len()]
}

fn render_connection_manifest(
    bytes: &[u8],
    profile: &str,
    namespace: &str,
    format: DiagnosticFormat,
) -> i32 {
    let source = match std::str::from_utf8(bytes) {
        Ok(source) => source,
        Err(error) => {
            render_tool_error(
                &format!("SQL test provision returned non-UTF-8 output: {error}"),
                format,
            );
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    let manifest = match sifr_sql_contract::TestConnectionManifest::from_json(source.trim()) {
        Ok(manifest) => manifest,
        Err(error) => {
            render_tool_error(&error.to_string(), format);
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    if manifest.profile != profile {
        render_tool_error(
            &format!(
                "SQL test provision returned profile '{}' for requested profile '{profile}'",
                manifest.profile
            ),
            format,
        );
        return EXIT_USER_DIAGNOSTIC;
    }
    if manifest.cleanup.tool_namespace != namespace {
        render_tool_error(
            "SQL test provision returned a cleanup namespace that differs from the selected tool",
            format,
        );
        return EXIT_USER_DIAGNOSTIC;
    }
    match manifest.to_canonical_json() {
        Ok(canonical) if writeln!(std::io::stdout(), "{canonical}").is_ok() => EXIT_SUCCESS,
        Ok(_) => EXIT_INTERNAL_COMPILER_FAILURE,
        Err(error) => {
            render_tool_error(&error.to_string(), format);
            EXIT_USER_DIAGNOSTIC
        }
    }
}

fn render_tool_error(message: &str, format: DiagnosticFormat) {
    render_diagnostics(
        &[diagnostic_with_code(
            message,
            DiagnosticCode::PACKAGE_METADATA_PARSE,
        )],
        format,
    );
}

#[cfg(test)]
mod tests {
    use super::super::cli_model_and_entrypoint::Cli;
    use clap::CommandFactory as _;
    use std::collections::BTreeSet;

    #[test]
    fn reserved_tool_namespaces_equal_builtin_cli_names() {
        let mut names = Cli::command()
            .get_subcommands()
            .map(|command| command.get_name().to_string())
            .collect::<BTreeSet<_>>();
        names.extend(["help".to_string(), "version".to_string()]);
        assert_eq!(
            names,
            sifr_package::RESERVED_TOOL_NAMESPACES
                .iter()
                .map(|name| (*name).to_string())
                .collect()
        );
    }

    #[test]
    fn provision_profile_does_not_scan_operands_after_separator() {
        let arguments = [
            "test".to_string(),
            "provision".to_string(),
            "--profile".to_string(),
            "app".to_string(),
            "--".to_string(),
            "--profile".to_string(),
            "operand".to_string(),
        ];
        assert_eq!(
            super::provision_profile("sql", &arguments),
            Ok(Some("app".to_string()))
        );
    }
}
