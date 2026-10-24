//! Independent Rust probe crates share one same-configuration Cargo invocation.
use super::cargo_invocation_trace::record_cargo_invocation;
use super::cargo_resolution::prepare_cargo_resolution;
use super::rust_interop_probe::{
    PendingRustBridgeProbe, ProbeExecutionFailure, probe_io_failure, probe_source,
};
use super::rust_interop_probe_cache::{
    ProbeInputSnapshot, mark_probe_cache_hit, probe_cache_file, probe_cache_key,
};
use super::rust_interop_probe_diagnostics::{classify_probe_failure, probe_resolution_diagnostics};
use super::rust_interop_probe_features::dependency_features;
use super::rust_interop_probe_manifest::{
    bind_probe_package_identity, probe_cargo_toml, probe_cargo_vendor_args,
};
use super::rust_interop_sqlx_offline::{
    configure_hermetic_build_environment, validate_probe_sqlx_offline_metadata,
};
use sifr_identity::IdentityEncoder;
use std::path::PathBuf;

#[path = "rust_interop_probe_batch_protocol.rs"]
mod protocol;
#[path = "rust_interop_probe_batch_workspace.rs"]
mod workspace;

struct PlannedProbe<'a> {
    index: usize,
    probe: &'a PendingRustBridgeProbe,
    backend_root: PathBuf,
    manifest: String,
    source: String,
}

pub(super) fn execute(probes: &[PendingRustBridgeProbe]) -> Vec<(usize, ProbeExecutionFailure)> {
    let mut groups: Vec<(String, Vec<PlannedProbe<'_>>)> = Vec::new();
    let mut failures = Vec::new();
    for (index, probe) in probes.iter().enumerate() {
        match plan(index, probe) {
            Ok((key, planned)) => {
                if let Some((_, group)) = groups.iter_mut().find(|(existing, _)| *existing == key) {
                    group.push(planned);
                } else {
                    groups.push((key, vec![planned]));
                }
            }
            Err(failure) => failures.push((index, failure)),
        }
    }
    for (_, group) in groups {
        match execute_group(&group) {
            Ok(results) => {
                for (planned, result) in group.iter().zip(results) {
                    if let Err(stderr) = result {
                        failures.push((
                            planned.index,
                            classify_probe_failure(planned.probe, &stderr),
                        ));
                    }
                }
            }
            Err(failure) => {
                failures.extend(group.iter().map(|planned| (planned.index, failure.clone())));
            }
        }
    }
    failures.sort_by_key(|(index, _)| *index);
    failures
}

fn plan(
    index: usize,
    probe: &PendingRustBridgeProbe,
) -> Result<(String, PlannedProbe<'_>), ProbeExecutionFailure> {
    if !probe.backend.cargo_manifest_path.is_file() {
        return Err(probe_io_failure(format!(
            "Rust probe Cargo manifest '{}' is missing or unreadable",
            probe.backend.cargo_manifest_path.display()
        )));
    }
    let backend_root = probe
        .backend
        .cargo_manifest_path
        .parent()
        .ok_or_else(|| probe_io_failure("Rust probe Cargo manifest has no parent".into()))?
        .to_path_buf();
    let features = dependency_features(&probe.backend.dependency_name, &backend_root, &probe.path);
    let structural = probe
        .signature
        .as_ref()
        .is_some_and(|s| !s.structural_type_params.is_empty())
        || sifr_ir::rust_opaque_structural_mapping(std::slice::from_ref(
            &probe.declaration.declaration,
        ))
        .is_some();
    let manifest = probe_cargo_toml(
        &probe.backend.dependency_name,
        &probe.backend.cargo_package_name,
        &backend_root,
        &probe.sysroot_runtime_crate,
        &features,
        structural,
    );
    let key = grouping_key(probe, &manifest)?;
    Ok((
        key,
        PlannedProbe {
            index,
            probe,
            backend_root,
            manifest,
            source: probe_source(probe),
        },
    ))
}

fn grouping_key(
    probe: &PendingRustBridgeProbe,
    manifest: &str,
) -> Result<String, ProbeExecutionFailure> {
    let policy = &probe.cargo_resolution;
    let tools = policy
        .native_toolchain
        .as_ref()
        .map_err(|e| probe_io_failure(e.clone()))?;
    let mut key = IdentityEncoder::new("rust-probe-configuration-v1");
    for (name, value) in [
        ("manifest", manifest),
        ("package-id", probe.backend.cargo_package_id.0.as_str()),
        ("package-version", probe.backend.cargo_version.as_str()),
        ("native-toolchain", tools.identity()),
        ("profile", policy.application_profile.policy_identity()),
        ("lock-mode", policy.lock_mode.as_str()),
    ] {
        key.field(name, value.as_bytes());
    }
    key.field(
        "cargo-source",
        format!("{:?}", probe.backend.cargo_source).as_bytes(),
    );
    key.field(
        "vendor-mode",
        format!("{:?}", policy.cargo_vendor_mode).as_bytes(),
    );
    key.field(
        "runtime-root",
        probe.sysroot_runtime_crate.as_os_str().as_encoded_bytes(),
    );
    key.field(
        "vendor-present",
        &[u8::from(probe.sysroot_vendor_dir.is_some())],
    );
    if let Some(path) = &probe.sysroot_vendor_dir {
        key.field("vendor", path.as_os_str().as_encoded_bytes());
    }
    key.field("trusted-sysroot", &[u8::from(probe.trusted_sysroot)]);
    for path in &policy.authoritative_locks {
        key.field("authority", path.as_os_str().as_encoded_bytes());
    }
    for path in &policy.trusted_vendor_dirs {
        key.field("trusted-vendor", path.as_os_str().as_encoded_bytes());
    }
    Ok(key.finish())
}

fn execute_group(
    group: &[PlannedProbe<'_>],
) -> Result<Vec<Result<(), String>>, ProbeExecutionFailure> {
    let first = group
        .first()
        .ok_or_else(|| probe_io_failure("empty Rust probe batch".into()))?;
    let probe = first.probe;
    let policy = &probe.cargo_resolution;
    let inputs =
        ProbeInputSnapshot::capture(probe, &first.backend_root).map_err(probe_io_failure)?;
    validate_probe_sqlx_offline_metadata(probe, &first.backend_root)?;
    let mut batch = IdentityEncoder::new("rust-probe-batch-v1");
    let mut keys = Vec::new();
    for planned in group {
        let manifest = bind_probe_package_identity(&planned.manifest, &planned.source);
        let key = probe_cache_key(planned.probe, &manifest, &planned.source, &inputs)
            .map_err(probe_io_failure)?;
        batch.field("probe", key.as_bytes());
        keys.push(key);
    }
    let vendor = policy
        .uses_sysroot_vendor()
        .then(|| {
            probe
                .sysroot_vendor_dir
                .as_deref()
                .or_else(|| policy.trusted_vendor_dirs.first().map(PathBuf::as_path))
        })
        .flatten();
    let prefix = probe_cargo_vendor_args(vendor);
    let family = policy
        .native_family(
            &probe.sysroot_runtime_crate,
            policy.cargo_vendor_mode,
            &prefix,
            "",
        )
        .map_err(|e| probe_io_failure(e.to_string()))?;
    let root = family
        .project(&first.backend_root, &batch.finish())
        .map_err(|e| probe_io_failure(e.to_string()))?;
    let members = workspace::write(
        &root,
        &group
            .iter()
            .map(|p| (p.manifest.as_str(), p.source.as_str()))
            .collect::<Vec<_>>(),
    )
    .map_err(probe_io_failure)?;
    let owned = policy
        .with_lease(family.lease())
        .map_err(|e| probe_resolution_diagnostics(&e))?;
    let prepared = prepare_cargo_resolution(&root, &owned, &prefix)
        .map_err(|e| probe_resolution_diagnostics(&e))?;
    let mut command = prepared
        .command_policy(&owned)
        .map_err(|e| probe_resolution_diagnostics(&e))?
        .cargo_command()
        .map_err(|e| probe_resolution_diagnostics(&e))?;
    command
        .args(&prefix)
        .args([
            "check",
            "--workspace",
            "--keep-going",
            "--quiet",
            "--message-format=json",
        ])
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"));
    policy.application_profile.configure(&mut command);
    if let Some(argument) = policy.lock_mode.cargo_arg() {
        command.arg(argument);
    }
    if !matches!(
        policy.lock_mode,
        sifr_package::CargoLockMode::Normal | sifr_package::CargoLockMode::Frozen
    ) {
        command.arg("--frozen");
    }
    // All declaration owners survive the grouped child, including when a caller
    // supplies distinct lifetime leases for otherwise identical configurations.
    for planned in group {
        planned
            .probe
            .cargo_resolution
            .child_leases
            .bind(&mut command)
            .map_err(|e| probe_io_failure(e.to_string()))?;
    }
    let target = family.target();
    crate::cache_storage::directory(&target).map_err(|e| probe_io_failure(e.to_string()))?;
    command.env("CARGO_TARGET_DIR", target);
    configure_hermetic_build_environment(&mut command);
    record_cargo_invocation("rust-probe", policy.lock_mode, &command);
    let output = crate::process_execution::output(&mut command)
        .map_err(|e| probe_io_failure(e.to_string()))?;
    prepared
        .assert_unchanged()
        .map_err(|e| probe_resolution_diagnostics(&e))?;
    let current =
        ProbeInputSnapshot::capture(probe, &first.backend_root).map_err(probe_io_failure)?;
    if current != inputs {
        return Err(probe_io_failure(
            "Rust probe inputs changed while Cargo was running".into(),
        ));
    }
    let results = protocol::outcomes(
        &output.stdout,
        &output.stderr,
        output.status.success(),
        &members,
    )
    .map_err(probe_io_failure)?;
    if results.len() != group.len() {
        return Err(probe_io_failure(
            "Rust probe batch has incomplete result accounting".into(),
        ));
    }
    let cwd = std::env::current_dir().map_err(|e| probe_io_failure(e.to_string()))?;
    for (key, result) in keys.iter().zip(&results) {
        if result.is_ok() {
            mark_probe_cache_hit(&probe_cache_file(key, &cwd));
        }
    }
    Ok(results)
}

#[cfg(test)]
#[path = "rust_interop_probe_batch_tests.rs"]
mod tests;
