## Code review: M13b final closure (round 2)

### Verdict

READY. No blockers.

### Blocking findings

None. All four round-2 review goals are satisfied:

1. **Manifest cleanly reduced to `retained-by-design` rows only.**
   `internal_docs/stdlib_retained_compiler_intrinsics.toml` has 11 surfaces, all `retained-by-design`. The 8 `closing` rows (`_sifr.sys`, `_sifr.http::header_helpers`, `_sifr.time`, `_sifr.crypto::random`, `_sifr.net`, `_sifr.tls`, `_sifr.signal`, `_sifr.python`, `_sifr.logging`) and the `closed_surface = "_sifr.process"` record on `origin/main` are gone. `python3 scripts/check_stdlib_manifest_schema.py` reports `PASS (surfaces=11, schema_version=2, final_state=retained-by-design)`. The schema now rejects unknown top-level fields (`closed_surface`), unknown surface fields (`removal_criteria`, `previous_state`, `removed_in_pr`), and any non-`retained-by-design` state, so re-introducing transitional artifacts fails the guard.

2. **Deleted migration closure guard is safely absorbed.**
   The permanent no-regression checks that survived from `scripts/check_stdlib_migration_closure.py` — deleted ownership registry absence and stale architecture phrases — are folded into `scripts/check_stdlib_native_intrinsic_allowlist.py:41-45,137-142,207-219` (`STALE_ARCH_PHRASES` and `_permanent_file_failures`). Self-test coverage exercises both branches (`check_stdlib_native_intrinsic_allowlist.py:436-448`). Wiring at `verification/policy/guardrails.json` and `verification/runner/sifr_verify/profile_runner.py:361-364` cleanly removes the deleted script's entrypoint; no dangling references remain outside of historical archive/review docs.

3. **Module-driven third-party dep leakage is gone; retained glue keeps a narrow allowlist.**
   `crates/sifr_stdlib_manifest/src/features/dependency_plan.rs:250-290` restricts `retained_direct_dependencies` to a `required_features`-only walk over `retained_dependency_specs`, which now enumerates only 10 packages (`bigdecimal`, `metrics`, `num-bigint`, `num-traits`, `rayon`, `rust_decimal`, `serde`, `serde_json`, `tokio`, `tracing`). The manifest's `generated-feature-planning-glue.retained_direct_dependency_packages` matches those 10 exactly. Network/TLS/HTTP/ICU/Unicode/URL/hash/rand/cookie/postcard/regex direct emissions are removed. The updated `stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies` test in `crates/sifr_stdlib_manifest/src/features_tests.rs:301-357` and the reshaped snapshots in `verification/areas/stdlib_parity/data/*.json` prove that module-only selection now emits only sysroot crates. Driver TLS trust in `crates/sifr_driver/src/build/materialize.rs:309-317` correctly derives from `SifrRuntime`/`SifrStdlib` `tls`/`http` crate features (the string-scraping `matches_tls_dependency_package` helper and its test are removed; `sysroot_tls_native_link_evidence_is_explicitly_trusted:585-604` reworked to push a `SifrStdlib` crate with `tls` feature instead of a dep string).

4. **Fallback signature modules are frozen in the manifest and gated by the surviving allowlist guard.**
   `internal_docs/stdlib_retained_compiler_intrinsics.toml:157-183` adds `[[surface]] id = "retained-fallback-signature-glue"` listing all 19 `_sifr.*` bootstrap signature modules. `scripts/check_stdlib_manifest_schema.py:31,113-122,125-135` accepts and requires `fallback_signature_modules` as an owned-surface field. `scripts/check_stdlib_native_intrinsic_allowlist.py:93-97,116-131,354-360` adds it to the observed→allowed comparison and self-test rejects removal (`fallback_signature_modules missing allowlist entries: _sifr.alpha`). Regex `RETAINED_SIGNATURE_MODULE_RE` at line 36 matches all 19 modules registered in `crates/sifr_retained_intrinsics/src/lib.rs:61-85`. Guard reports `fallback_signature_modules=19` and passes.

### Non-blocking notes

**N1. Top-level module doc in `sifr_retained_intrinsics` still uses transitional wording.**
`crates/sifr_retained_intrinsics/src/lib.rs:1-4` says "Transitional compiler-retained stdlib intrinsic signatures" and "This crate hosts fallback signatures that still feed lowering and driver bootstrap while native stdlib declarations continue replacing them." Round-1 N4 was addressed for the inline dispatcher comments (lines 70–71, 79–80, 82–83 now say "Retained as compiler-owned bootstrap signature glue for sysroot source lowering and reviewed by the final retained-glue manifest"), but the module-level doc-comment above still frames the crate as transitional. Now that the manifest has frozen the 19 fallback modules as `retained-by-design`, the crate is bootstrap glue, not migration scaffolding. Reword to match. Hygiene, not correctness.

**N2. Round-1 N6 regex fragility persists.**
`scripts/check_stdlib_native_intrinsic_allowlist.py:37` — `GENERATED_DEPENDENCY_PACKAGE_RE = re.compile(r'"([A-Za-z0-9_-]+)\s+=')` — still relies on the exact `"pkg = ..."` string shape in the non-test portion of `dependency_plan.rs`. It correctly extracts the current 10 packages, but a future edit that splits a spec across lines or drops the whitespace around `=` would silently reduce the observed set, letting a stale allowlist entry escape detection. Consider bracketing the spec block with a named guardrail marker or scan the parsed `retained_dependency_specs` function body directly. Non-blocking; the manifest and self-test still catch drift on the allowlist side.

**N3. Roadmap row wording for M13a is slightly imprecise.**
`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:104` says "M13a final sys/fs boundary migrated the last `_sifr.sys` and `_sifr.fs` native leaves through sysroot declarations and removed retained compiler dispatch/fallback paths". `_sifr.fs` still retains `builtin_open`/`builtin_open_text` and its file-handle registry files as `retained-by-design` per the current manifest (lines 12-29), so "last `_sifr.fs` native leaves" reads as if all fs behavior migrated. This describes an already-merged PR and is documentation-only, but future closeout housekeeping should clarify that M13a closed the migratable fs leaves while builtin-`open` shadowing remains permanent language glue.

**N4. Standard closeout housekeeping remains.**
`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:5` still shows the top-level `## Status` as `In progress.`, and the M13 row (line 104) has only the M13a evidence cell. Expected pre-merge; when M13b lands, the row needs the M13b PR sha entry and the summary line should flip to the final wording, per the phase's per-milestone closeout rules.

### Verification performed

- `python3 scripts/check_stdlib_manifest_schema.py` — PASS (surfaces=11, schema_version=2, final_state=retained-by-design)
- `python3 scripts/check_stdlib_manifest_schema.py --self-test` — PASS (including new `_sifr.new_bad_state: new manifest rows must be retained-by-design` rejection at lines 380–398)
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py` — PASS (exact_intrinsics=27, registry_files=11, preamble_files=9, fallback_signature_modules=19, retained_direct_dependency_packages=10, direct_runtime_roots=2)
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py --self-test` — PASS (missing-fallback-module rejection at lines 353–360 confirmed)
- Regex sanity check against `dependency_plan.rs` non-test region captures exactly the 10 packages in the manifest allowlist.
- Manifest `retained-fallback-signature-glue.fallback_signature_modules` list matches the 19 `_sifr.*` modules registered in `sifr_retained_intrinsics/src/lib.rs:61-85`.
- `sysroot_trusted_native_links` in `materialize.rs` derives trust solely from `SifrRuntime`/`SifrStdlib` crate features; dead `matches_tls_dependency_package` helper and its unit test are removed; the reworked TLS trust test at `materialize.rs:585-604` uses the crate-feature path.
- Snapshot updates in `verification/areas/stdlib_parity/data/network_http_dependency_snapshots.json` and `concurrency_runtime_dependency_snapshots.json` remove all module-only third-party leakage; the http-transport snapshot still includes the retained `tokio`/`tracing` glue because those features are explicitly passed as `required_features`, matching `retained_dependency_specs`.

The 4 non-blocking notes are hygiene items; none require code changes before merging M13b.
