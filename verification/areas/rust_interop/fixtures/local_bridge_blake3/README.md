# local_bridge_blake3

Current evidence is projection/probe-level. Runtime `blake3` byte hashing waits
for the bridge type and direct signature contracts.

- Positive evidence: package-local bridge roots resolve as `bridge.*`,
  generated bridge namespace projections are recorded deterministically, and
  managed projection files are generated without overwriting user-owned bridge
  files (`rust_interop_tests::package_rust_interop_resolves_bridge_root`,
  `package_projection_tests::rust_bridge_projection_repair_writes_managed_projection_without_touching_user_bridge`).
- Negative evidence: bridge module conflicts and invalid bridge filenames are
  Sifr projection diagnostics before Cargo (`package_projection_tests::rust_bridge_projection_conflict_does_not_overwrite_user_authored_mod_rs`,
  `package_projection_tests::rust_bridge_projection_rejects_keyword_bridge_module_filename`).
- Archive evidence: Rust-backed local bridges require `Cargo.toml`, managed
  projection files, and user bridge files in package archives
  (`package_rust_bridge_archive_tests::*`).
