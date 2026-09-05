# Architecture Review: adhoc-seamless-package-dx.md (Round 2)

**Reviewer:** agent architecture review
**Date:** 2026-05-20
**Document:** `issues/adhoc-seamless-package-dx.md`
**Implementation state:** Phase 37 complete, adhoc phase spec exists, implementation not started
**Review scope:** Round 2 — verification of Round 1 blocker resolution and implementation readiness

---

## Executive Summary

**STATUS: READY — All 24 Round 1 blockers resolved. No remaining blockers.**

The spec is implementation-ready across all tracked dimensions: DX, source layout, API model, dependency projection, Cargo failure boundary, workspaces, migration, compiler/codegen, diagnostics, security, testing, demos, and guardrails. Round 1 was thorough; every concrete required change appears in the updated spec.

---

## Round 1 Blocker Resolution

### 1. User Experience and Command Semantics

**Blocker 1 (init on existing directory):** RESOLVED.
- Lines 208-213 define explicit `--force` behavior, non-interactive requirement, and diagnostics path for existing manifests.
- `sifr init` fails if target directory contains files without `--force`. `--force` creates missing Sifr-owned files but must not overwrite user files.

**Blocker 2 (sifr fix --check ambiguity):** RESOLVED.
- Line 510-514: `sifr fix --check` reports failures without writing. `sifr fix` attempts repair unless `--frozen` is active.

**Blocker 3 (sifr --explain for non-package diagnostics):** RESOLVED.
- Lines 548-555: `--explain` includes package-manager-specific recovery only for `SIFR-PACKAGE-*` codes.

**Blocker 4 (projection drift + network):** RESOLVED.
- Line 601: "Projection drift is checked before Cargo command execution." Line 510: "Projection drift always fails package-aware commands before invoking Cargo."

### 2. Package Layout and `__init__.sifr` API Model

**Blocker 5 (pure marker lifecycle):** RESOLVED.
- Lines 131-138: Lifecycle states, validation trigger ("every package-aware command"), diagnostic (`SIFR-PACKAGE-0501`), and explicit non-silent-conversion rule.
- `SIFR-PACKAGE-0709` (line 504) covers the case where the pure marker is missing but cannot be regenerated because user-owned Rust target exists.

**Blocker 6 (non-re-export in __init__.sifr):** RESOLVED.
- Line 316: "Definitions written directly in `__init__.sifr` are public names of that namespace when their names do not start with `_`."
- Lines 362-370: Complete semantic spec for class/def/type definitions and import forms.

**Blocker 7 (parse_init_sifr_reexports grammar):** RESOLVED.
- Lines 324-342: Full input/output data structure with `NamespaceApi` containing `public_symbols: BTreeMap<String, PublicSymbolOrigin>` and `public_child_namespaces`.
- Lines 344-370: Complete grammar and semantic spec for all supported import forms.

**Blocker 8 (filesystem vs. namespace graph):** RESOLVED.
- Line 318-319: "Filesystem presence confirms possible module locations; the namespace API graph determines what is public." Line 319: "A directory without `__init__.sifr` is not a public namespace across package boundaries."

### 3. Dependency Schema and Projection

**Blocker 9 (dependency schema underspecified):** RESOLVED.
- Lines 376-417: Complete schema with all field meanings, backward compatibility rules, and `sifr add` write target explicitly set to `sifr.toml`.

**Blocker 10 (0702 drift diagnostic):** RESOLVED.
- Lines 500-504: Separate codes for different drift cases. SIFR-PACKAGE-0702 covers dependency/alias drift; 0703 covers metadata pointer; 0704 covers include patterns; 0705 covers invalid source root.

**Blocker 11 (Sifr writes Cargo.toml):** RESOLVED.
- Lines 460-468: Explicitly states "Sifr-owned Cargo sections are marked and guarded" and "Sifr never rewrites user-owned Cargo sections except through explicit migration/fix commands."

**Blocker 12 (three-way compatibility matrix):** RESOLVED.
- Lines 903-909: Full compatibility matrix table covering legacy Phase 37, migrating, and new packages.

**Blocker 13 (sifr add --alias for Phase 37 packages):** RESOLVED.
- Lines 411-413: For legacy Phase 37 packages, the commands preserve the legacy model. Migration is explicit.

**Blocker 14 (migration + guardrails overlap):** RESOLVED.
- Lines 919-922: In-tree fixtures used for milestone-level testing. Published demo repos updated only after compiler integration is complete.
- Line 1054: Guardrails explicitly accept legacy `sifr/` fixtures during migration window.

**Blocker 15 (tar rollback):** RESOLVED.
- Lines 896-901: JSON migration descriptor with SHA-256 checksums, checksum-addressed file storage, conflict detection, and explicit rejection of tar approach.

### 4. Session Architecture

**Blocker 16 (OperationPlan minimal):** RESOLVED.
- Lines 576-594: Full `OperationPlan` schema including `CargoCommandPlan` with `command`, `current_dir`, `targets`, `lock_mode`, `features`, and `args`.

**Blocker 17 (credential redaction GitHub prefixes):** RESOLVED.
- Line 795: Full list including `gh_`, `gho_`, `ghp_`, `ghs_`, `ghr_`, `ghu_`.

**Blocker 18 (0204 machine-readable fields):** RESOLVED.
- Lines 651-662: Complete field list including `expected_package_instance_id`, `actual_package_instance_id`, `expected_cargo_package_id`, `actual_cargo_package_id`, `import_path`, `dependency_path`, `expected_type`, `actual_type`.

**Blocker 19 (alias conflict within scope):** RESOLVED.
- Lines 418-423: Definition of same-import-root conflict, duplicate-import-root diagnostic, and same-instance-different-import rule.

### 5. Diagnostics

**Blocker 20 (0105 retirement):** RESOLVED.
- Lines 736-740: Must-be-retired requirement with explicit rationale.
- Lines 804-810: Retirement process including doc rewrite, test migration, and guardrail for new active constants.
- Line 975: Milestone 3 includes "retire or supersede credential-specific Cargo-failure codes."

**Blocker 21 (URL credential redaction):** RESOLVED.
- Lines 797-798: Explicitly requires URL parsing for userinfo redaction. Example: `https://user:token@private.example.com/pkg` → `https://[redacted host]/pkg`. "Word-level substring replacement is not sufficient."

### 6. Implementation Order and Guardrails

**Blocker 22 (PackageSourceMap replacement):** RESOLVED.
- Line 932: "Implement `parse_init_sifr_reexports` and namespace API graph derivation in `PackageSourceMap`." This is the replacement.

**Blocker 23 (guardrails not spec'd):** RESOLVED.
- Lines 472-477: Sifr-owned section marking with stable comments/metadata.
- Line 476: "projection regeneration is idempotent" (requires idempotency tests).
- Lines 131-138: Pure marker lifecycle and validation trigger.
- Lines 1053-1056: Full guardrail extension spec for milestone 7.

**Blocker 24 (demo fixture strategy):** RESOLVED.
- Lines 919-922: In-tree fixtures under `verification/package_management/src_layout_fixtures/`. Published demo repos updated in milestone 7 only.

---

## Gaps Assessed

All 12 "must-fix" items from Round 1 are resolved. The 7 gaps from Round 1 (Phase 37 selector reference, schema consolidation plan, virtual workspace warning conditions, partial publish behavior, pure marker lifecycle, workspace diagnostics reference, same-version alias) are addressed as follows:

- **Phase 37 selector reference**: Line 686-689, explicit inheritance clause.
- **Schema consolidation**: Lines 911-916, post-phase legacy cleanup plan.
- **Virtual workspace warning**: Lines 679-684, precise warning conditions.
- **Partial publish**: Lines 719-723, explicit "no rollback unless Cargo exposes stable command" boundary.
- **Pure marker lifecycle**: Lines 131-138, explicit specification.
- **Workspace diagnostics**: Line 823, reference to `SIFR-PACKAGE-06xx` family.
- **Same-version alias**: Lines 421-422, allowed but treated as two names for one type identity.

No new gaps identified. The spec is coherent.

---

## Cross-Cutting Verification

### DX Completeness
All commands have explicit semantics. Lock/network mode interaction is specified for every operation. Offline validation reuses Phase 37 model (line 602). No ambiguity in the `sifr init/add/remove/update/fix` surface.

### Projection Model Consistency
Sifr-owned sections: marked, idempotent, guarded by `0702/0703/0704/0705/0709`. User-owned sections: preserved, validated (not conflicting), never silently rewritten. Drift always fails before Cargo. `cargo add` on Sifr-owned sections creates drift reportable by `sifr fix --check`.

### Migration Safety
Rollback uses JSON descriptor + checksums, not tar. Conflict detection prevents partial-overwrite on rollback. `--apply-partial` is explicit opt-in for manual repair. Migration validation diffs public API before/after and reports `0707` on divergence.

### Diagnostics Coherence
One stable Cargo wrapper (`0101`) with bounded redaction. Specific Sifr codes only for Sifr-owned failures. `0105` retirement process with test migration path. Redaction covers URL userinfo with structured parsing. Machine-readable fields specified for `0101` and `0204`.

### Compiler Integration
Namespace hashing from Cargo package id + version + source + Sifr package name. Type identity includes package instance id. No data-dependent unwraps in generated runtime. `PackageSourceMap` derives public API from `__init__.sifr`.

### Guardrails
Source layout, projection boundaries, and demo commands covered in milestone 7. Idempotency tests required. Sifr-owned Cargo section markers required. Pure-marker modifications caught before Cargo. Credential-code guardrail forbids new active Cargo stderr taxonomy codes.

---

## Conclusion

**The reviewer is satisfied. This plan is implementation-ready.**

All 24 Round 1 blockers have been addressed with concrete spec language. All 12 Round 1 required changes appear in the updated spec. All 7 Round 1 gaps are resolved or explicitly referenced to Phase 37.

The plan is structured for incremental, reviewable milestones with clear acceptance criteria, validation commands, and compatibility matrix. The migration path is safe and rollback is machine-checkable. The diagnostics model is coherent and maintainable.

Implementation can proceed.

---

*Round 2 of 2. Reviewer is satisfied. No blockers remain.*