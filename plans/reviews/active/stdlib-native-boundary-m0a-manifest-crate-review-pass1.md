## M0a Code Review — `sifr_stdlib_model` → `sifr_stdlib_manifest`

**Verdict: Satisfactory to proceed to PR.** No blockers. All M0a scope items (workspace/lock rename, downstream imports, guardrails, verification profiles, coverage classification, docs) are updated coherently and consistently. IPC helpers and retained intrinsic signatures remain in the crate as explicit transitional M0 state — documented in the new `lib.rs` module docstring — and pushing them out is correctly deferred to M0b/M0c. No fallback paths, compatibility aliases, or duplicate registries were introduced.

### Confirmed correctness

- **Workspace + Cargo.lock rename is atomic and minimal.** `Cargo.toml:9,57`, `Cargo.lock:3159,3190,3263,3380` flip together; `[package] name = "sifr_stdlib_manifest"` with no `[lib] name` legacy override and no re-export shim.
- **All active source, test, verification-JSON, profile-JSON, sanitizer-JSON, and generated-code-quality producer-fingerprint entries flip in the same diff.** `rg` across `crates`, `scripts`, `verification`, `internal_docs`, `docs`, `plans/issues/active`, `plans/phases` returns zero live references to `sifr_stdlib_model` (matches user validation).
- **Dependency-direction guardrail is updated end-to-end.** `scripts/check_source_crate_dependency_direction.py` updates `ALL_SIFR_CRATES`, three `*_FORBIDDEN_DEPENDENCIES` sets, the `RULES` entry, the `generated_dependency_spec_violations` skip clause, `seed_valid_repo` fixtures, and both self-test cases — no dangling old name.
- **Fixture binary name (`sifr-stdlib-ipc-pipe-fixture-worker`) intentionally not renamed.** Consumers in `crates/sifr_lowering/src/lower/ipc_schema_extraction.rs:211` and `crates/sifr_stdlib_manifest/tests/ipc_process_pipe_fixture.rs:57` still match; renaming now would be churn since M0b extracts IPC into `sifr_ipc` anyway.
- **New crate-level doc comment in `crates/sifr_stdlib_manifest/src/lib.rs:1-7`** explicitly calls out retained intrinsic signatures and IPC protocol helpers as transitional split state, matching M0a plan constraints.
- **Package rename in `verification/areas/coverage_matrix/data/cargo_metadata_classification.json:165-168`** correctly updates both the package `name` and the `[lib] target` while leaving the `bin`/`test` target names untouched (they were never renamed).
- **Archived plans/reviews under `plans/reviews/active/**` and `plans/issues/archive/**` are intentionally left with `sifr_stdlib_model` references** — matches the "archived plans/reviews are intentionally not rewritten" constraint.

### Non-blocking cleanup suggestions

These are doc-only rough edges left by mechanical global replace. None affect code correctness, guardrails, or the plan's acceptance criteria.

1. **`internal_docs/architecture.md:275` — self-referential sentence.**
   ```
   sifr_stdlib_manifest/   (... ; split out from current sifr_stdlib_manifest)
   ```
   The trailing clause was "split out from current `sifr_stdlib_model`" before the rename and now says the crate was split out from itself. The crate no longer needs a "was renamed from" annotation — either drop the trailing clause, or reword to describe what still remains to be split out (IPC → `sifr_ipc`, retained signatures → compiler-retained-glue, suggestion policy → frontend/diagnostics).

2. **`internal_docs/architecture.md:276` — stale phrase after rename.**
   ```
   sifr_ipc/  (... split out from the current stdlib model)
   ```
   Inconsistent with the rename; "the current `sifr_stdlib_manifest`" would match the rest of the doc.

3. **`internal_docs/sifr_sysroot_and_stdlib_architecture.md:59` — awkward row phrasing.** The "Compiler-side stdlib model" row now has both `Current owner` and `Final owner` referencing `crates/sifr_stdlib_manifest`, with the final-owner cell describing "Split into `crates/sifr_stdlib_manifest`". Content is accurate (moves IPC to `sifr_ipc`, suggestion policy out, deletes fallback signatures) but the row label still says "stdlib model" and reads as splitting the crate into itself. Consider renaming the row to "Compiler-side stdlib manifest" and phrasing the final owner as scope reduction rather than "Split into" the same crate name.

4. **`crates/sifr_stdlib_manifest/src/lib.rs:1-7` docstring** enumerates "retained intrinsic signatures" and "IPC protocol helpers" as transitional but does not mention that the legacy CPython-shaped import suggestion policy (`unsupported_legacy_stdlib_module`, `is_bare_stdlib_tail`, `cpython_stdlib_reserved_suggestion`, `LegacyStdlibModule`, `BareStdlibMatch`) is likewise transitional (M0c moves it to the frontend/diagnostics boundary). One additional sentence would make the transitional surface fully self-documented so a future reader doesn't cement suggestion policy here.

None of the above alters observable behavior or fails any guardrail. All four verification profiles, the dependency-direction script (with its self-test), coverage classification, sanitizer reproductions, generated-code-quality producer fingerprint, and the architecture ownership table are internally consistent under the new name.

**Recommendation: proceed to PR.** The three doc nits can be folded into M0e (architecture doc updates) rather than blocking M0a merge, since M0e revisits these docs anyway.
