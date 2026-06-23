## M0 Review — Pass 3 (Ad Hoc Sifr Sysroot and Stdlib Toolchain)

### Post-Pass-2 Changes — Verification

The user reported two changes: the registry field rename `deletion_milestone` → `deletion_stage`, and internal_docs text scrubbed of phase/milestone/M-label language. Both are clean.

**TOML schema rename — VERIFIED.**
- `internal_docs/stdlib_native_surface_ownership.toml:19`-`338`: every row uses `deletion_stage = "<descriptive label>"`; no row carries the old `deletion_milestone` key.
- `tomllib` parse shows the per-row key set is exactly `{id, public_modules, current_owner, final_owner, reason, certification_state, migration_blocker, can_move_before_runtime_certification, deletion_stage}` — 9/9 required fields on all 30 rows.
- The 9 distinct `deletion_stage` values (`stateless-native-leaf migration`, `fallible-data-text migration`, `filesystem-resource migration`, `process-resource migration`, `network-tls-resource migration`, `http-resource migration`, `python-resource migration`, `runtime-state migration`, `final retained allowlist`) preserve the deletion-ordering semantics that `M9`/`M10`/`M11a-f`/`M12` previously carried, without baking issue-plan labels into the registry.
- Header metadata at `internal_docs/stdlib_native_surface_ownership.toml:7` uses `created_for_stage = "baseline_inventory"` — taxonomy-compliant.

**Internal-docs taxonomy scrub — VERIFIED.**
- `grep -niE "\bm[0-9]+|milestone|phase" internal_docs/sifr_sysroot_and_stdlib_architecture.md internal_docs/stdlib_native_surface_ownership.toml` returns zero matches. Architecture prose uses descriptive substitutes: `internal_docs/sifr_sysroot_and_stdlib_architecture.md:23` ("Later implementation stages"), `:103` ("deletion stage"), `:104` ("the final intrinsic-registry cleanup stage"), `:548` ("By the final intrinsic-registry cleanup stage").
- Issue plan and roadmap (`plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md`, `plans/roadmap.md:80`) keep M0-M13 / Phase 39.1 labels — correct, since the guardrail constrains `internal_docs/` only.

### Pass 2 Items — Status Re-check

- **Coverage**: 29 distinct `_sifr.*` imports in `lib/sifr/*.sifr` (math, crypto, bytes, calendar, collections, compress, datetime, encoding, fs, html, http, i18n, json, logging, net, platform, process, python, regex, runtime, signal, sys, task, time, tls, toml, unicode, url, uuid) — every one has a registry row. Plus `generated-runtime-integer-glue` for the runtime bridge family. Total 30 rows = full coverage; no orphans.
- **Cross-doc links**: `internal_docs/architecture.md:56` and `plans/roadmap.md:80` still resolve to the architecture doc and registry with correct relative paths.
- **PR-log placeholder** at `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:16` ("M0 baseline/inventory: pending.") still awaits the URL on merge — non-blocking, same as pass 1/pass 2.

### Findings

| Severity | File:line | Finding |
| --- | --- | --- |
| Info | `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:73` | Acceptance task still says "deletion milestone" in English prose while the registry field is now `deletion_stage`. The issue plan is allowed M-label language, so this isn't a taxonomy violation, but the lexical drift between the issue's task list and the registry's actual field name is mildly confusing. Non-blocking; consider rewording to "deletion stage" or "deletion milestone (`deletion_stage` field)" on the same merge that fills the PR-log. |
| Info | `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:16` | PR-log line still reads "M0 baseline/inventory: pending." — placeholder for the PR URL, fills on merge. Not an M0 gate. |

No High / Medium / Low findings. No blockers.

### Validation Report

`target/validation_lane_reports/create-pr.latest.json`:
- `case_timings`: 125 cases, all `pass` (Counter: `{'pass': 125}`)
- `advisories: ["warm wall-time budget exceeded"]` — perf budget note, not a correctness signal
- `policy`, `lane_steps`, `hardening_summary`, `e2e` all present; no `failures`/`failed_cases` keys emitted (the schema only adds those on failure)

### Verdict

**review-satisfied** for M0. The `deletion_milestone` → `deletion_stage` rename is complete and consistent across all 30 rows, and internal_docs is free of phase/milestone/M-label language while the deletion-stage ordering semantics are preserved. Coverage of every `_sifr.*` private import and the generated runtime integer glue is intact. The acceptance bar at `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:79-88` remains met. Ready to open the M0 PR and proceed to M1.
