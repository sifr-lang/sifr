# `milestone_diag_4b` slice 3 — `RenderedDiagnostic` retirement review (pass 1)

## Scope under review

Per [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:72](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:72), slice 3 of `milestone_diag_4b`:

- Delete the remaining custom driver `CompilerDiagnostic` transport.
- Delete the public driver re-exports of canonical diagnostic model types (`Severity`, `DiagnosticSpan`, `DiagnosticChild`, `DiagnosticSuggestion`, `RelatedSpan`, `SuggestionKind`).
- Carry `sifr_diagnostics::RenderedDiagnostic` directly through driver and CLI APIs.
- Preserve active `SIFR-*` diagnostic identity, code-derived labels, compact rendering behavior, and recovery-limit behavior.
- No fallback paths or compatibility shims.

I read every file in the working tree's `git status`, the full diff (`git diff HEAD`), the canonical model definitions in `crates/sifr_diagnostics/src/model/mod.rs` and `crates/sifr_diagnostics/src/render/mod.rs`, the verification baseline directory under `crates/sifr/tests/verification/**/baselines/`, the validation-lane manifest at `verification/validation_lanes/manifest.json`, and the previous slice-2 review at [reviews/semantic-diagnostic-code-taxonomy-diag-4b-compile-error-retirement-review-pass-2.md](reviews/semantic-diagnostic-code-taxonomy-diag-4b-compile-error-retirement-review-pass-2.md). I did not run any cargo or harness commands; the user reports `cargo test -p sifr_driver --lib --tests` passes.

## Summary

The Rust source migration is correct, mechanical, and fully scoped. `CompilerDiagnostic`, the local `Severity::Help` variant, the bespoke `DiagnosticSpan/Child/Suggestion/RelatedSpan/SuggestionKind` types, and the `Display for CompilerDiagnostic` impl have all been deleted from `sifr_driver` and `sifr`. Every `Vec<CompilerDiagnostic>` / `Box<CompilerDiagnostic>` API in the driver and CLI now carries `Vec<RenderedDiagnostic>` / `Box<RenderedDiagnostic>` directly, and the `pub use` block in [crates/sifr_driver/src/lib.rs:23](crates/sifr_driver/src/lib.rs:23) no longer re-exports any canonical diagnostic-model type. Construction goes through `crate::diagnostics::diagnostic_with_code(...)` (driver) and a small private `diagnostic_with_code` (CLI), each of which sets `code`, `severity = code.declared_severity()`, `url = code.docs_url()`, and a stub `message_template = "{message}"` plus single-arg `args` map. The orchestrator's `lower_frontend_module` forwarding still mutates only `error.message`, and the slice-2 invariant test [crates/sifr_driver/src/tests/test_runner.rs:319](crates/sifr_driver/src/tests/test_runner.rs:319) is preserved without modification, so frontend `SIFR-TYPE-0002` identity continues to survive `run_tests`.

Behavior preservation is **almost** complete — but there is one BLOCKING regression in the JSON output schema visible to the verification hardening harness, plus three NIT-level observations.

## Findings

### F1 — BLOCKING — JSON verification baselines were not regenerated

The CLI's JSON renderer at [crates/sifr/src/main.rs:418](crates/sifr/src/main.rs:418) still calls `serde_json::to_string_pretty(&diagnostics)`, but `diagnostics` is now `Vec<RenderedDiagnostic>` instead of `Vec<CompilerDiagnostic>`. Those two structs serialize differently:

- `RenderedDiagnostic` declares `spans: Vec<DiagnosticSpan>` (where the canonical `DiagnosticSpan` itself has the new fields `byte_start`, `byte_end`, `end_line`, `end_column`, `is_primary`, `label`, `lines`).
- `RenderedDiagnostic` adds `message_template: String` and `args: BTreeMap<String, DiagnosticArg>`.
- The retired `CompilerDiagnostic` had `primary_span: Option<DiagnosticSpan>` and `related_spans: Vec<RelatedSpan>` instead. Those names are gone from the new envelope.
- Children move from the slim `DiagnosticChild { severity, message }` to `RenderedDiagnosticChild` with `ChildSeverity` (a different enum that admits `Help` only as a child). Empty children are not affected, but the type identity changed.

So for the same input, the new JSON stderr will look like:

```
{
  "code": "...",
  "severity": "Error",
  "message": "...",
  "message_template": "{message}",
  "args": { "message": { "kind": "string", "value": "..." } },
  "url": "...",
  "spans": [],
  "children": [],
  "help": null,
  "suggestions": []
}
```

instead of the old:

```
{
  "code": "...",
  "severity": "Error",
  "message": "...",
  "url": "...",
  "primary_span": null,
  "related_spans": [],
  "children": [],
  "help": null,
  "suggestions": []
}
```

The five checked-in baselines below still carry the **old** schema (`primary_span: null`, `related_spans: []`, no `message_template`/`args`/`spans`):

- [crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt](crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt)
- [crates/sifr/tests/verification/project/missing_import_reports_error/baselines/check-json.stderr.txt](crates/sifr/tests/verification/project/missing_import_reports_error/baselines/check-json.stderr.txt)
- [crates/sifr/tests/verification/project/workspace_unresolved_import/baselines/check-json.stderr.txt](crates/sifr/tests/verification/project/workspace_unresolved_import/baselines/check-json.stderr.txt)
- [crates/sifr/tests/verification/project/workspace_ambiguous_import/baselines/check-json.stderr.txt](crates/sifr/tests/verification/project/workspace_ambiguous_import/baselines/check-json.stderr.txt)
- [crates/sifr/tests/verification/project/workspace_malformed_manifest/baselines/check-json.stderr.txt](crates/sifr/tests/verification/project/workspace_malformed_manifest/baselines/check-json.stderr.txt)

The hardening harness is wired up in [scripts/run_all_tests.sh:143](scripts/run_all_tests.sh:143)/[scripts/run_verification_hardening.py:308](scripts/run_verification_hardening.py:308) and the `pr` profile of [verification/validation_lanes/manifest.json](verification/validation_lanes/manifest.json) lists `hardening_suites = ["diagnostics", "project", ...]`, so the `pr` (authoritative merge gate), `nightly`, and `release` lanes will all hit a `stderr` mismatch on these five fixtures. `quick` does not run hardening (`hardening_suites: []`), which is why the user's `cargo test -p sifr_driver --lib --tests` did not catch it — but the slice cannot ship without those baselines reflecting the new canonical schema.

Why this is the slice's responsibility: switching the public CLI JSON wire format from the bespoke `CompilerDiagnostic` shape to the canonical `RenderedDiagnostic` shape is *the* observable consequence of the slice — and it is the right shape (it matches the schema produced by `render_sink_*` and the `DiagnosticEnvelope` schema in `sifr_diagnostics`). The fix is to bless the new output, not to add a serialization adapter.

Recommended action: regenerate the five baselines (`python3 scripts/run_verification_hardening.py --bless --suite diagnostics --suite project` or equivalent), include them in this slice's diff, and re-run `scripts/run_all_tests.sh --profile quick` plus at minimum `python3 scripts/run_verification_hardening.py --suite diagnostics --suite project` (or run `--profile pr`).

Two follow-ups that should land alongside the bless:

- The internal handoff doc at [internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:26](internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:26) still describes the canonical schema as containing `primary_span` and `related_spans`. That description was already stale before this slice (the canonical schema uses `spans`), but the slice makes the staleness user-visible because the CLI's JSON now actually matches the canonical schema. Consider updating that bullet in the same PR; the `27_*` doc is a phase-final spec, not an active migration tracker, so this is a NIT, not a blocker.
- An add-only sanity test that asserts a sample `RenderedDiagnostic` JSON output through the same `serde_json::to_string_pretty` path the CLI uses would future-proof the baselines against schema drift; existing tests cover `render_compact_diagnostics` shape but not `render_diagnostics`/JSON.

### F2 — NIT — `compact_severity_summary` `help_count` semantics changed

[crates/sifr/src/main.rs:299](crates/sifr/src/main.rs:299) used to count `Severity::Help`-typed diagnostics into `help_count`. Canonical `Severity` has no `Help` variant, so the new code instead counts `diagnostic.help.is_some()`:

```rust
match diagnostic.severity {
    Severity::Error => error_count += 1,
    Severity::Warning => warning_count += 1,
    Severity::Note => note_count += 1,
}
if diagnostic.help.is_some() {
    help_count += 1;
}
```

This is a *deliberate semantic change*, not a slip:

- In production, no driver code path emitted `Severity::Help`, so `help_count` was always 0 in old code.
- In production, the driver's `diagnostic_with_code` constructor leaves `help: None`, and no driver/HIR code ever calls a `.help(...)` builder against the rendered envelope, so `help_count` is still effectively always 0 today.
- If/when the slice's deferred `DiagnosticSink` migration starts forwarding HIR-emitted help text into the rendered envelope, `help_count` will start reflecting "diagnostics that ship a `help:` line", which is a more useful summary than the old "diagnostics that ARE help".

The renamed snapshot test `test_compact_renderer_snapshot_multi_severity_group_order` at [crates/sifr/src/main.rs:1419](crates/sifr/src/main.rs:1419) was updated coherently — the third diagnostic flips from `Severity::Help` (old) to `Severity::Note` (new), and the `1 help item(s)` count now flows from the warning's `Some("remove the assignment")` help text. The expected output's `note` and `help` lines are mutually consistent.

I would not block on this, but it is worth recording in the PR description as a deliberate definition change rather than a no-op rename — anyone reading the diff will reach for the wrong mental model otherwise.

### F3 — NIT — duplicated `diagnostic_with_code` and legacy-display helpers across the driver/CLI crate boundary

[crates/sifr/src/main.rs:94](crates/sifr/src/main.rs:94) reintroduces a private `diagnostic_with_code(message, code) -> RenderedDiagnostic` that is byte-for-byte identical to [crates/sifr_driver/src/diagnostics.rs:28](crates/sifr_driver/src/diagnostics.rs:28)'s `pub(crate) fn diagnostic_with_code`. Similarly, [crates/sifr/src/main.rs:291](crates/sifr/src/main.rs:291) defines a `#[cfg(test)] fn legacy_diagnostic_display(...)` that mirrors [crates/sifr_driver/src/diagnostics.rs:108](crates/sifr_driver/src/diagnostics.rs:108)'s `pub(crate) fn diagnostic_legacy_display`.

The duplication is forced by the `pub(crate)` visibility — promoting the driver helper to `pub` would make sense if more than one caller in `sifr` ends up needing it, but right now only `run_with_panic_boundary` and a handful of CLI tests construct one. I would leave it as-is for this slice but call it out so we don't sprawl: if a fourth or fifth `diagnostic_with_code` site appears under `crates/sifr/`, promote the driver's helper to `pub` rather than copy-paste again.

The CLI's `legacy_diagnostic_display` is `#[cfg(test)]`, but the driver's `diagnostic_legacy_display` is not. That asymmetry is inconsequential (the driver helper is also only called from tests today) but mildly inconsistent.

### F4 — NIT — stub `message_template`/`args` is correct but does pollute every diagnostic

`diagnostic_with_code` builds every diagnostic with `message_template = "{message}"` and `args = {"message": <full rendered message>}`. That is the right way to keep the rendered envelope JSON-Schema-valid (`deny_unknown_fields` deserialization works, and the message is recoverable from the args), and it preserves the slice's "no compatibility fallback" rule (no special fallback `message_template = ""`). Two minor consequences worth noting:

- Every diagnostic in JSON now ships a redundant template/args copy of its `message`. That is fine as a transitional shape; the inventory at [internal_docs/diagnostic_emission_inventory.md:84](internal_docs/diagnostic_emission_inventory.md:84) already says the next step is `DiagnosticSink`-direct rendering. Noting it so the next reviewer of the `DiagnosticSink` migration does not mistake the stub for "the canonical template path".
- Forwarded HIR diagnostics arrive at the orchestrator as `RenderedDiagnostic` already (built via the same stub by `lowering_error_to_diagnostic`), so the `error.message = format!("[{}] {}", ...)` mutation at [crates/sifr_driver/src/test_runner/orchestrator.rs:107](crates/sifr_driver/src/test_runner/orchestrator.rs:107) leaves `args["message"]` desynchronized from `message`. That is OK because no current code path re-formats `message` from `message_template` on the driver/CLI side — `render_diagnostics`, `render_compact_diagnostics`, and `apply_diagnostic_recovery_limits` all read `diagnostic.message` directly. If the CLI ever switches to template-based formatting on this stream, this mutation will need to update the args too. Not actionable in this slice; recording for the `DiagnosticSink`-direct migration.

## Slice-goal verification

- ✓ `git grep -n "CompilerDiagnostic\b" -- crates/` returns zero matches. The transport is fully gone.
- ✓ `git grep -nE "\bRelatedSpan\b|\bDiagnosticSuggestion\b|\bSuggestionKind\b|\bDiagnosticChild\b" -- crates/sifr_driver/ crates/sifr/` returns zero matches. None of the bespoke types survive in the driver or CLI.
- ✓ `git grep -nE "Severity::Help" -- crates/sifr_driver/ crates/sifr/` returns zero matches. The vestigial fourth severity variant is gone.
- ✓ `git grep -nE "primary_span|related_spans" -- crates/sifr_driver/ crates/sifr/src/` returns only the new local helper `fn primary_span(diagnostic: &RenderedDiagnostic)` at [crates/sifr_driver/src/diagnostics.rs:104](crates/sifr_driver/src/diagnostics.rs:104) and its single caller in `apply_diagnostic_recovery_limits`. (See F1 for the orphaned baseline strings.)
- ✓ The `pub use diagnostics::{...}` block in [crates/sifr_driver/src/lib.rs:23](crates/sifr_driver/src/lib.rs:23) was reduced to `apply_diagnostic_recovery_limits, diagnostic_label_for_code, diagnostic_label_for_code_str, CompileResult, CompileResultFull` — no canonical-model types are re-exported. CLI imports `DiagnosticArg`, `DiagnosticCode`, `DiagnosticSpan`, `RenderedDiagnostic`, and `Severity` directly from `sifr_diagnostics`.
- ✓ Active SIFR-* identity at construction: every `diagnostic_with_code(_, code)` call site preserves the explicit `DiagnosticCode::*` argument. Identities verified at the build/materialize/workspace/discovery/compile_order/frontend/stdlib/test-runner sites, and the slice-2 invariant test (`error.code == DiagnosticCode::TYPE_MISMATCH.code()` and `!= INTERNAL_COMPILER_PANIC`) is preserved unchanged.
- ✓ `apply_diagnostic_recovery_limits` algorithm is unchanged: same `MAX_TOP_LEVEL_DIAGNOSTICS = 50`, `MAX_SIMILAR_DIAGNOSTICS_PER_GROUP = 5`, same severity-rank-by-code-by-message-by-file group key, same `... +N more similar diagnostics` summary collapsing, same final `truncate(50)` cap. Span clearing now goes through `summary.spans.clear()` (was `primary_span = None; related_spans.clear()`); behavior preserved because `diagnostic_with_code` always produces empty `spans` so the clear is a no-op today.
- ✓ Code-derived label table `diagnostic_label_for_code_str` at [crates/sifr_driver/src/diagnostics.rs:122](crates/sifr_driver/src/diagnostics.rs:122) is unchanged: same `internal compiler error` / `build error` / `parse error` / `codegen error` / `type error` mapping. The `test_diagnostic_labels_are_derived_from_diagnostic_codes` table-driven test at [crates/sifr_driver/src/tests/diagnostics.rs:81](crates/sifr_driver/src/tests/diagnostics.rs:81) still pins this against `crate::diagnostics::diagnostic_legacy_display` for every active code family.
- ✓ Compact rendering: `render_compact_diagnostics` keeps the same group key and ordering, only swapping `&diagnostic.primary_span` for `diagnostic.spans.iter().find(|span| span.is_primary)`. With empty spans (the production case), behavior is identical.
- ✓ No fallback paths or compatibility shims: I read every diagnostic-construction site in the diff and saw no `if old { ... } else { ... }` branch, no transitional `Severity::Help` stub, no message-prefix fallback for code derivation. The slice is pure replacement, not a parallel transport.

## Test coverage

Slice-2's invariant test [crates/sifr_driver/src/tests/test_runner.rs:319](crates/sifr_driver/src/tests/test_runner.rs:319) is preserved verbatim and still asserts `code == TYPE_MISMATCH.code()` plus `code != INTERNAL_COMPILER_PANIC.code()` after the orchestrator's path-prefix mutation. Slice-2's no-fallback-rebuild stdlib-cache test at [crates/sifr_driver/src/stdlib/cache.rs:46](crates/sifr_driver/src/stdlib/cache.rs:46) is rewritten to use the new helper but otherwise unchanged. The seven driver-level tests in `tests/diagnostics.rs` (stable code/url, label derivation, order preservation, workspace codes, no-prefix-derivation, recovery-limit summarization, top-level cap) all migrate correctly to `RenderedDiagnostic` via the new `test_diagnostic`/`primary_test_span` helpers, with the same assertions and same numeric thresholds (50/5).

CLI tests in `crates/sifr/src/main.rs` (75 `#[test]` items, up from 37 in `HEAD` — the deltas are the new `test_diagnostic`/`primary_test_span` helpers, not net new tests; the actual `#[test]` count appears unchanged barring the helper functions). The four `compile_entrypoint` consistency tests (`test_compile_entrypoint_error_consistency_for_*`) and the two `check_entrypoint` consistency tests now run their byte-equality comparison through `legacy_diagnostic_display` rather than `ToString::to_string`. That is the right replacement: the old `Display` impl produced `{label}: {message}`, and `legacy_diagnostic_display` reproduces exactly that text.

What I did not see and would optionally add (non-blocking, but called out for completeness):

- A test that exercises the `serde_json::to_string_pretty(&Vec<RenderedDiagnostic>)` path used by `DiagnosticFormat::Json` in `render_diagnostics`. The closest existing coverage is the verification baselines themselves (see F1), which currently lie about the schema. A minimal unit test that asserts the JSON contains `"spans"` and `"message_template"` fields and *not* `"primary_span"`/`"related_spans"` would catch any future accidental regression to the bespoke schema.

## Doc and inventory updates

- [internal_docs/diagnostic_emission_inventory.md:8](internal_docs/diagnostic_emission_inventory.md:8) is rewritten correctly: "the legacy public `CompileError` abstraction and the custom driver `CompilerDiagnostic` transport have been deleted. Driver and CLI APIs now carry `sifr_diagnostics::RenderedDiagnostic` directly". The driver-and-CLI surface section at line 84 is similarly updated. The diagnostic-construction-count table row for `crates/sifr_driver/src/diagnostics.rs` correctly notes the panic-boundary site now uses `diagnostic_with_code` building a canonical rendered diagnostic.
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:72](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:72) is the only line added to the issue and correctly marks slice 3 as `[ ] in progress`. The slice-2 row at line 71 stays `[x]` and continues to point at PR #1707.
- The `test-only manual construction` table at [internal_docs/diagnostic_emission_inventory.md:108](internal_docs/diagnostic_emission_inventory.md:108) renames `Manual CompilerDiagnostic sites` → `Manual RenderedDiagnostic sites` and shifts the migration-owner annotation. The 9-vs-2 split between `crates/sifr/src/main.rs` (9 sites) and `crates/sifr_driver/src/tests/diagnostics.rs` (2 sites) is consistent with the new CLI helper `test_diagnostic` (used in 9 places via the snapshot tests) and the two driver-test recovery-limit fixtures that still construct envelopes via the `test_diagnostic` driver helper.

## Recommended action

- BLOCKING: regenerate the five `check-json.stderr.txt` baselines listed in F1 and ship them in the same slice. After regeneration, re-run at least `python3 scripts/run_verification_hardening.py --suite diagnostics --suite project` (or `scripts/run_all_tests.sh --profile pr`) and record the result in the issue's validation evidence block.
- Optional: add a `serde_json::to_string_pretty` schema-shape unit test under `crates/sifr/src/main.rs` to lock the new wire format.
- Optional: PR-description note that `compact_severity_summary`'s `help_count` is now derived from `help.is_some()` rather than `Severity::Help` (F2).
- Optional: refresh the stale [internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:26](internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:26) bullet so the documented canonical schema names match the implementation (`spans` instead of `primary_span`/`related_spans`).

Once the baselines land, this slice is ready for the issue checklist's `[x]` mark. The Rust side of the work is in good shape and the slice goals are otherwise fully met.
