I have everything needed. The `_sifr.i` was a sed artifact — `_sifr.i18n` is the real import. Below is the pass 2 review.

## M0 Review — Pass 2 (Ad Hoc Sifr Sysroot and Stdlib Toolchain)

### Pass 1 Blockers — Status

**B1. `_sifr.uuid` row missing — RESOLVED.**
`internal_docs/stdlib_native_surface_ownership.toml:65-74` now contains a `[[surface]]` block with:
- `id = "_sifr.uuid"` matching architecture row `internal_docs/sifr_sysroot_and_stdlib_architecture.md:72`
- `public_modules = ["sifr.uuid"]` (matches the actual `from _sifr.uuid` import in `lib/sifr/uuid.sifr:2`)
- `current_owner` points at `crates/sifr_codegen/src/intrinsics/registry/uuid.rs` with rand/uuid dep planning
- `final_owner = "crates/sifr_stdlib uuid feature via stdlib/_sifr/uuid.sifr"` — parallel to `_sifr.regex` (line 58) and `_sifr.html` (line 91)
- `certification_state = "can-move-before-runtime-certification"`, `can_move_before_runtime_certification = true`, `deletion_milestone = "M9"` — all consistent with the stateless-leaf classification pass 1 required.

**B2. `_sifr.logging` row missing — RESOLVED.**
`internal_docs/stdlib_native_surface_ownership.toml:153-162` adds the row pass 1 asked for:
- `id = "_sifr.logging"`, `public_modules = ["sifr.logging"]` (matches `lib/sifr/logging.sifr:2`)
- `current_owner` explicitly names `crates/sifr_codegen/src/intrinsics/registry/logging.rs` **plus** "generated global logging state" — correctly captures the `__SIFR_GLOBAL_LOG_LEVEL` statefulness pass 1 flagged
- `certification_state = "future-owned-by-runtime-resource-certification"`, `can_move_before_runtime_certification = false` — correct posture for stateful surface
- `deletion_milestone = "M11c"` — aligns with the M11c submilestone title "Signals and runtime/logging state" at `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:586`.

### Pass 1 Non-Blocking Items — Status

- **N1 (call-site table breadth)** addressed at `internal_docs/sifr_sysroot_and_stdlib_architecture.md:85-87`: prose now says "This table groups the call sites by migration concern; the complete surface-by-surface ownership decision remains the TOML registry," redirecting reviewers to the registry as the authoritative enumeration.
- **N2 (matrix-row id enumeration)** addressed at `internal_docs/sifr_sysroot_and_stdlib_architecture.md:110-116`: all 11 future-owned row ids are listed by id (`bridge_type_matrix`, `opaque_resource_matrix`, `panic_boundary_wrapper_emission`, `async_runtime_reqwest`, `callbacks_call_scoped`, `callback_subscription_matrix`, `ecosystem_backend_certification`, `ecosystem_cli_certification`, `native_build_script`, `proc_macro_trust`, `cargo_locked_offline`). Mechanically checkable.
- **N3 / N4** were explicitly non-blocking; no change required.

### Pass 2 Cross-Checks (re-run on the updated files)

- Registry parses to 30 rows (29 `_sifr.*` + `generated-runtime-integer-glue`). The 29 `_sifr.*` ids exactly match the 29 distinct private imports found by `grep -E "^from _sifr\." lib/sifr/*.sifr` — no surface in `lib/sifr` is missing a row, and no row has gone orphan.
- Required fields (`current_owner`, `final_owner`, `reason`, `certification_state`, `migration_blocker`, `can_move_before_runtime_certification`, `deletion_milestone`) are present on every row (verified for the two new ones; pass 1 verified the other 28).
- `internal_docs/architecture.md:56` and `plans/roadmap.md:80` still link to both new artifacts with correct relative paths.
- File-size guardrail PASS per the user's run.

### Remaining M0 Blockers

None. The acceptance bar at `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:79-88` ("inventories identify every native stdlib surface", "the migration registry gives reviewers one mechanical inventory", "runtime/resource surfaces are tagged with certification status before implementation begins") is now met.

The PR-log placeholder at line 16 ("M0 baseline/inventory: pending.") still needs the PR URL on merge — same non-blocking note as pass 1, not an M0 gate.

### Verdict

**review-satisfied** for M0. B1 and B2 are resolved with rows that match the structural requirements pass 1 specified, and N1/N2 were addressed in passing. The phase is ready to proceed to M1.
