# Review: milestone_diag_2a — Diagnostic Registry Skeleton (Pass 1)

Branch: `codex/semantic-diagnostics-diag-2a`
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Validation evidence reported: `cargo test -p sifr_diagnostics`, `python3 scripts/check_diagnostic_schema_sync.py`, `python3 scripts/check_diagnostic_docs_sync.py`, `cargo fmt --check`, `cargo check --workspace`, `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings`, `scripts/run_all_tests.sh --profile quick` (signature `e1bf653aaa770517`).

## Scope reviewed

- `crates/sifr_diagnostics/src/codes.rs` (replacing `crates/sifr_diagnostics/src/codes/mod.rs`)
- `crates/sifr_diagnostics/src/bin/gen-error-docs.rs` (new generator binary)
- `crates/sifr_diagnostics/src/model/mod.rs` and `crates/sifr_diagnostics/src/render/mod.rs` (test-only `DiagnosticCode` updates)
- `docs/errors/diagnostic-codes.md`, `internal_docs/diagnostic_codes.md` (generated outputs)
- `scripts/check_diagnostic_docs_sync.py`, `scripts/run_all_tests.sh` (drift validation wiring)
- `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md` (phase tracker)

## Verdict

The skeleton meets the spirit of `milestone_diag_2a`: it is structurally correct, vacuously consistent with zero active codes, and ships a working docs generator + drift check wired into local validation. There are **no blocking correctness defects**. A small number of DoD facets are partially implemented because they are vacuous at zero active codes; those should be filled out before any active code lands in `milestone_diag_2b`. There is also one piece of repository hygiene to clean up.

## Definition-of-done coverage

| DoD bullet | Status | Evidence / notes |
| --- | --- | --- |
| Registry skeleton with families, per-family `0000` numbering, state machine, reserved family bases | ✅ | `DIAGNOSTIC_FAMILIES` + 17 `reserved_family_base(...)` entries in [codes.rs:117-222](../crates/sifr_diagnostics/src/codes.rs:117). |
| `SIFR-INTERNAL-0001` reserved (Error, panic boundary), `SIFR-INTERNAL-0002` reserved (Note, recovery cap) | ✅ | [codes.rs:223-250](../crates/sifr_diagnostics/src/codes.rs:223). |
| Registry and code constants cannot silently diverge | ✅ | `registry_skeleton_is_internally_consistent` enforces `ACTIVE_DIAGNOSTIC_CODES` ⇔ `Active` entries set equality, plus per-code severity equality ([codes.rs:361-379](../crates/sifr_diagnostics/src/codes.rs:361)). |
| Registry record shape (id, family, summary, state, docs path, fixture, template, owner, declared args, dedupe args, tooling) | ✅ | `DiagnosticRegistryEntry` carries all required fields ([codes.rs:101-115](../crates/sifr_diagnostics/src/codes.rs:101)). |
| `DiagnosticCode::code()` is the only accessor used for JSON, docs URLs, sorting, registry checks | ✅ | `docs_url` derives from `code()` ([codes.rs:36-38](../crates/sifr_diagnostics/src/codes.rs:36)); `docs_slug` field has been removed entirely. |
| Tooling metadata defaults documented; no LSP/code-action validation in this phase | ✅ | `DiagnosticTooling::DEFAULT` ([codes.rs:88-91](../crates/sifr_diagnostics/src/codes.rs:88)); explicit "tooling metadata defaults" prose in both generated docs. |
| Docs generator writes `docs/errors/<CODE>.md`, `docs/errors/diagnostic-codes.md`, `internal_docs/diagnostic_codes.md` | ✅ | `generated_documents()` produces all three ([gen-error-docs.rs:45-65](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:45)); generated outputs match. |
| Build-time validation test checks template placeholders against declared args, JSON-only arg declarations, docs-page presence for active codes, constant/registry sync, canonical code forms, registry state validity, registry-declared severity constraints | ⚠️ partial | Five of seven facets implemented (see _Findings → F1_). |
| Skeleton validation must pass with zero active codes; active-code checks become non-vacuous in 2b | ✅ | `ACTIVE_DIAGNOSTIC_CODES = &[]` ([codes.rs:253](../crates/sifr_diagnostics/src/codes.rs:253)); test passes. |
| Non-test emission-presence checks activate per family later | ✅ | Not in scope; deferred per DoD. |
| CI/local validation runs the generator and fails on drift (DoD says "with `git diff --exit-code`") | ⚠️ deviation | Uses in-memory `--check` comparison rather than `git diff --exit-code` (see _Findings → F2_). Functional, but a known deviation. |

## Findings

### Blocking

None.

### Non-blocking — DoD facets that need filling out before active codes land

#### F1. Registry validation does not yet enforce JSON-only arg declarations or strong severity constraints

The DoD enumerates seven things the build-time validation test must check. Five are implemented; two are essentially placeholders:

1. **JSON-only arg declarations.** `DiagnosticArgFormat::JsonOnly` exists ([codes.rs:60-73](../crates/sifr_diagnostics/src/codes.rs:60)), but no assertion ties it to behavior. The intended invariant is that an arg declared `JsonOnly` must NOT appear as a `{name}` placeholder in the message template — otherwise "JSON-only" is meaningless. Currently `assert_template_placeholders_are_declared` only checks placeholders ⊆ declared args, not formats. Vacuous today (zero active codes, no `JsonOnly` args), but the framework is missing.

   Suggestion (when 2b lands): in `assert_template_placeholders_are_declared`, also assert that for each placeholder the matching declaration's format is `MessageAndJson`.

2. **Registry-declared severity constraints.** Today the only severity invariant is "active entries declare a severity" ([codes.rs:322-326](../crates/sifr_diagnostics/src/codes.rs:322)) plus the pairwise equality with `DiagnosticCode::declared_severity`. The DoD wording suggests stronger constraints (e.g., source diagnostics declare `Error|Warning`; structural notes declare `Note`; reserved family bases never declare severity, etc.). None of those are enforced. As above, vacuous today.

   Suggestion: spell out the invariant in the test, even if the only currently-enforceable rule is "reserved family-base entries (`SIFR-<FAMILY>-0000`) must have `declared_severity == None`." That is verifiable today and would prevent a future drift.

Both items are non-blocking because the milestone is explicit that active-code checks "become non-vacuous in `milestone_diag_2b`". Flagging here so they are not lost when 2b lands.

#### F2. Drift check uses an in-memory comparison rather than `git diff --exit-code`

DoD: "CI or local validation can run the generator and fail on drift with `git diff --exit-code`."

Implementation: `gen-error-docs --check` reads each expected path from disk and string-compares against the in-memory generator output ([gen-error-docs.rs:79-99](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:79)), invoked from [scripts/check_diagnostic_docs_sync.py](../scripts/check_diagnostic_docs_sync.py).

This is functionally equivalent for the *intended* drift surface (modifications to checked-in generator outputs) and is arguably better than `git diff --exit-code`, since it works in dirty trees and does not silently ignore untracked siblings. It does, however, have one blind spot the literal `git diff` approach also misses: **orphan files** in `docs/errors/`. If a future code is retired (and so disappears from `active_registry_entries()`), its `docs/errors/SIFR-...md` page will linger on disk and the check will not flag it.

Suggestion (non-blocking, can be filed for 2b): in `check_documents`, also enumerate `docs/errors/*.md`, build the expected set as `{ "diagnostic-codes.md" } ∪ { format!("{}.md", entry.id) for entry in active_registry_entries() }`, and report any disk file not in the expected set.

This is genuinely out of scope today (no active or retired codes), but the drift check is the only safety net here, so the gap is worth noting.

### Non-blocking — repository hygiene

#### F3. Empty leftover directory `crates/sifr_diagnostics/src/codes/`

`git status` shows `deleted: crates/sifr_diagnostics/src/codes/mod.rs`, but the now-empty parent directory is still present on disk:

```
crates/sifr_diagnostics/src/codes/
└── (empty)
```

It does not affect git (empty dirs aren't tracked) and it does not affect the build (the module file is `crates/sifr_diagnostics/src/codes.rs`, not `codes/mod.rs`), but it is noise that future readers may trip over. Recommend `rmdir crates/sifr_diagnostics/src/codes` before opening the PR.

#### F4. `escape_table` only escapes `|`

`escape_table` ([gen-error-docs.rs:286-288](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:286)) only escapes pipes. The fields it wraps in backticks (`message_template`, `owner_module`, `tool_actions`, `dedupe_args`, declared arg names) are emitted as inline-code spans — e.g. `` `{template}` ``. If any of those values ever contains a backtick, the inline-code span will close prematurely and break the table cell layout.

Vacuous today (no values contain backticks) but worth either (a) documenting "these registry strings must not contain backtick or pipe" and asserting that in the validation test, or (b) extending `escape_table` to escape backticks too. Either is fine; one of them should land before the first active code with a string template.

### Non-blocking — future-readiness flag

#### F5. `DiagnosticCode::new` is `#[cfg(test)]` only

Today the only constructor for `DiagnosticCode` is gated to test builds ([codes.rs:17-23](../crates/sifr_diagnostics/src/codes.rs:17)). For the skeleton phase that is consistent and even desirable: with zero active codes, no production code should be able to mint a `DiagnosticCode` value, and `ACTIVE_DIAGNOSTIC_CODES` is empty.

When `milestone_diag_2b` populates active entries, the constructor will need to be ungated (or replaced by a non-test internal constructor). Not a defect — just a load-bearing detail to remember at the 2b boundary, where it will need to flip together with the population PR rather than land separately.

## Spot checks

- **No external consumer breakage from removed constants.** `INTERNAL_COMPILER_PANIC`, `NAME_UNDEFINED_VARIABLE`, and `TYPE_ASSIGNMENT_MISMATCH` were the only previously-exposed `DiagnosticCode` constants. `grep` confirms zero references outside this crate, so removing them is safe. The two existing in-crate test sites (`model/mod.rs`, `render/mod.rs`) have been redirected to the new `TEST_*` constants.
- **JSON schema unaffected by `DiagnosticCode` shape change.** `RenderedDiagnostic.code` is a plain `string` in the schema (see [docs/schemas/diagnostics.schema.json:27-29](../docs/schemas/diagnostics.schema.json:27)), so removing the `docs_slug` field from `DiagnosticCode` (now folded into `code()`) does not perturb the schema, and `check_diagnostic_schema_sync.py` is a no-op for this milestone. ✅
- **Test-only diagnostic codes do not collide with the registry.** `SIFR-INTERNAL-9998`, `SIFR-INTERNAL-9999`, and `SIFR-NAME-9999` sit at the top of their respective family ranges, are not present in `DIAGNOSTIC_REGISTRY`, and are gated `#[cfg(test)]`. They are never iterated by registry checks. ✅
- **`SIFR-INTERNAL-0002` severity invariant.** Recovery-cap omission is explicitly `Note` ([codes.rs:248](../crates/sifr_diagnostics/src/codes.rs:248)) and remains `Reserved` until `milestone_diag_10`, matching the design principle ("Severity → Error|Warning|Note") for `INTERNAL` notes. ✅
- **Documentation prose for `Severity::Help` exclusion at top level** is preserved in the model ([model/mod.rs](../crates/sifr_diagnostics/src/model/mod.rs)); the registry never declares `Help` and the enum makes that impossible. ✅
- **`active_registry_entries()` filters are pure.** No side-effecting iterators or filesystem reads happen inside the validation tests except `docs/errors` directory enumeration, which is bounded and cheap. ✅
- **`run_all_tests.sh` wiring.** `check_diagnostic_docs_sync.py` is invoked unconditionally before `cargo test -p sifr_diagnostics`, which means a docs-drift regression fails fast in every lane (quick/pr/nightly/release). ✅
- **Phase tracker accuracy.** The new entries in `issues/...` correctly describe what landed (registry skeleton + docs validation) and explicitly cite the report signature `e1bf653aaa770517`. The remaining unchecked boxes are (a) this review and (b) PR open + merge, which is the right state for this point in the workflow. ✅

## Suggested next actions

1. (Hygiene) `rmdir crates/sifr_diagnostics/src/codes` to remove the empty leftover directory.
2. (Optional, non-blocking) Tighten one currently-trivial registry assertion — e.g. assert reserved family-base entries (`SIFR-<FAMILY>-0000`) carry `declared_severity == None` — so the severity-constraint DoD facet is not entirely vacuous before 2b.
3. Defer the JSON-only-arg-format check, the `escape_table` backtick handling, and the orphan-page detection to land alongside the first active codes in `milestone_diag_2b`. Do not bundle them here; they would expand scope without clearing a current risk.

## Conclusion

Skeleton is correct, internally consistent, and properly wired into local validation. Approve to proceed to PR after the empty `codes/` directory is removed. The deferred validation tightening (F1, F2, F4) should land at the same time as, or before, the first active code populates the registry in `milestone_diag_2b` — they are not blockers for this milestone, but they are blockers for that one.
