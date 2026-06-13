# Review: Semantic Diagnostic Code Taxonomy — milestone_diag_1 (pass 3)

**Scope reviewed:** uncommitted working tree at `branch=main` against (a) the `milestone_diag_1` definition in [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) and (b) the must-fix list in [reviews/semantic-diagnostic-code-taxonomy-diag-1-review-pass-2.md §H](reviews/semantic-diagnostic-code-taxonomy-diag-1-review-pass-2.md). This pass is narrowly scoped to verifying the pass-2 blockers; should-fix items are re-stated only when status changes.

**Files re-inspected (pass-3 deltas only):**

- [crates/sifr_diagnostics/src/lib.rs](crates/sifr_diagnostics/src/lib.rs) (clippy carve-out)
- [crates/sifr_diagnostics/src/render/mod.rs](crates/sifr_diagnostics/src/render/mod.rs) (`#[schemars(required)]` annotations + CRLF comment)
- [docs/schemas/diagnostics.schema.json](docs/schemas/diagnostics.schema.json) (regenerated)
- [internal_docs/architecture.md](internal_docs/architecture.md) (diagnostic-object bullet wording)
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) (review artifact references)
- [reviews/](reviews/) directory listing (confirm `…-review-blocked.md` removal)

**Validation re-run by reviewer (this pass):**

- `cargo test -p sifr_diagnostics` → 25 passed, 0 failed.
- `python3 scripts/check_diagnostic_schema_sync.py` → OK (exit 0).
- `cargo check --workspace` → clean.
- `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings` → **clean (no errors, no warnings)**.
- `cargo run -q -p sifr_diagnostics --bin gen-diagnostic-schema | jq` → required arrays for `RenderedDiagnostic` and `DiagnosticSpan` match the on-disk schema.

**Verdict:** all five pass-2 must-fix items are resolved. No new regressions or correctness issues introduced. The milestone is **mergeable** once one residual documentation lag is addressed (§E.1 below — single-line edit in the issue's Execution Status block to reflect pass-3 status).

---

## §A. Pass-2 must-fix resolution

### A.1 — pass-2 §C.1 (clippy `--all-targets`) → ✅ Resolved

[crates/sifr_diagnostics/src/lib.rs:1](crates/sifr_diagnostics/src/lib.rs:1) now carries:

```rust
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]
```

This is a strict drop-in port of the pattern at [crates/sifr_hir/src/lib.rs:7](crates/sifr_hir/src/lib.rs:7), exactly as pass-2 §C.1 specified. Re-running the gate locally:

```
$ cargo clippy -p sifr_diagnostics --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
```

Zero errors, zero warnings — the 12 `unwrap_used` / `unwrap_err` errors flagged in pass-2 are gone. The carve-out is correctly scoped to `cfg(test)` so production code still observes the workspace-warn lint.

The pass-2 §D.3 follow-up about `scripts/run_all_tests.sh` not running clippy at any profile is unchanged (script unmodified). That remains a should-fix follow-up: the gate is now enforceable, but only by manual invocation. Not a blocker for `milestone_diag_1`.

### A.2 — pass-2 §C.4 (schema-vs-writer Option-field asymmetry) → ✅ Resolved

The pass-3 author took option (1) from pass-2 — tighten the schema to match the writer's explicit-null contract — using the documented `schemars` v1 idiom of `#[schemars(required)]` on the relevant `Option<T>` fields:

- [crates/sifr_diagnostics/src/render/mod.rs:26](crates/sifr_diagnostics/src/render/mod.rs:26) — `RenderedDiagnostic.help`
- [crates/sifr_diagnostics/src/render/mod.rs:56,60,62,64,66,69](crates/sifr_diagnostics/src/render/mod.rs:56) — `DiagnosticSpan.{file,line,column,end_line,end_column,label}`

The regenerated [docs/schemas/diagnostics.schema.json](docs/schemas/diagnostics.schema.json) reflects this:

- `RenderedDiagnostic.required` ([schema:71-82](docs/schemas/diagnostics.schema.json:71)) now contains `code, severity, message, message_template, args, url, spans, children, help, suggestions` — `help` is present.
- `DiagnosticSpan.required` ([schema:230-241](docs/schemas/diagnostics.schema.json:230)) now contains `file, byte_start, byte_end, line, column, end_line, end_column, is_primary, label, lines` — all six previously-missing Option fields are present.

Cross-check (live regenerator output matches on-disk schema):

```
$ cargo run -q -p sifr_diagnostics --bin gen-diagnostic-schema | …
RenderedDiagnostic.required: ['code', 'severity', 'message', 'message_template', 'args',
                              'url', 'spans', 'children', 'help', 'suggestions']
DiagnosticSpan.required: ['file', 'byte_start', 'byte_end', 'line', 'column',
                          'end_line', 'end_column', 'is_primary', 'label', 'lines']
```

The `python3 scripts/check_diagnostic_schema_sync.py` gate confirms the on-disk schema matches the generator output bit-for-bit. The pass-2 round-trip test (`rendered_diagnostic_json_round_trips_without_losing_arg_kinds_or_nulls`) at [crates/sifr_diagnostics/src/render/mod.rs:497](crates/sifr_diagnostics/src/render/mod.rs:497) still asserts `"help":null` and `"label":null` are present in writer output, so the writer's explicit-null contract continues to be locked. Schema and writer now agree.

This is the right path forward: keeps the milestone DoD's "explicit `null` fields where applicable" wording at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:721](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:721) honest, requires no test deletions, and gives external schema consumers a strict contract. No correctness regressions detected — the round-trip test still passes and `deny_unknown_fields` is unchanged on every consumed payload.

### A.3 — pass-2 §D.1 (architecture wording) → ✅ Resolved

[internal_docs/architecture.md:720](internal_docs/architecture.md:720) now reads:

> Canonical diagnostic object: target migrated parser, lowering, type checking, borrow checking, and codegen paths must emit `SifrDiagnostic` values from `sifr_diagnostics`. …

The phrase "target migrated … paths must emit" makes the prescriptive nature explicit: this is the end-state contract, not a description of present emission state. `milestone_diag_1` introduces the model; emission migration begins in `milestone_diag_4a`. The sentence no longer overstates current state and correctly aligns with the rest of the section (which talks about codegen/rustc mapping in the next bullet at [internal_docs/architecture.md:722](internal_docs/architecture.md:722)). Reads naturally.

### A.4 — pass-2 §D.2 (review-blocked file disposition) → ✅ Resolved

`reviews/semantic-diagnostic-code-taxonomy-diag-1-review-blocked.md` is no longer in the tree. `ls reviews/` confirms only `…-review-pass-1.md` and `…-review-pass-2.md` for this milestone. References:

- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:18](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:18) now points at `reviews/semantic-diagnostic-code-taxonomy-diag-1-review-pass-1.md` and `reviews/semantic-diagnostic-code-taxonomy-diag-1-review-pass-2.md` only — no stale link to the deleted file.
- [internal_docs/architecture.md](internal_docs/architecture.md) — no references (verified by `grep -n "review-blocked"`).
- The remaining `review-blocked` matches in the tree are inside the pass-1 / pass-2 review files themselves — i.e., historical record of the audit trail. Those should not be edited; they're frozen artifacts, and the deletion of the actual file is what was required.

The chain of evidence for `milestone_diag_1` is now: pass-1 (initial audit, blockers identified) → pass-2 (must-fix list) → pass-3 (this file, blockers verified resolved). A future reader following the issue references will land on the correct, current review.

### A.5 — pass-2 §B.5 (CRLF human-renderer trap comment) → ✅ Resolved

[crates/sifr_diagnostics/src/render/mod.rs:277-278](crates/sifr_diagnostics/src/render/mod.rs:277) now carries:

```rust
// CRLF sources retain `\r` in serialized line text. Human renderers should
// normalize or strip it before printing snippets to a terminal.
fn span_lines(...) { ... }
```

Placed directly above `span_lines`, which is the function that hands `lines[i].text` (still containing the trailing `\r` for CRLF sources, as locked by `crlf_source_has_stable_line_and_column_positions` at [render/mod.rs:452](crates/sifr_diagnostics/src/render/mod.rs:452)) to whatever consumer renders it. This is the load-bearing site for the trap — anyone writing the eventual human renderer in `milestone_diag_4a`-and-after will see this before reaching for `print!("{}", line.text)`. The comment is appropriately surgical: two lines, no exposition, no `unsafe_code`-style scare-quoting.

(Pass-2 had also suggested an alternative placement near `line_starts(...)` in `source_map/mod.rs`. Either site works; the chosen site is closer to where the value is actually emitted to consumers, which is the more useful warning location.)

---

## §B. Pass-2 should-fix items (re-confirmed status)

These were not in scope for pass 3 and remain explicitly tracked as follow-ups, not blockers:

| Pass-2 ref | Item | Status after pass 3 |
|---|---|---|
| §B.1 | `model/mod.rs` decomposition (now ~756 lines + drop hook) | Still open. Should land before `milestone_diag_2a` registry growth. |
| §B.3 | `severity_rank` redundant with derived `Ord` | Still open. Cosmetic; one-line static-assert or direct enum reuse. |
| §A.4 noted gap | `#[cfg(not(debug_assertions))]` test for the release drop counter | Still open. Pre-existing parallel-test race risk also remains latent. |
| §B.2 residual | `kind_rank` overlap with `path_rank` | Still open. Cosmetic. |
| §A.5 follow-up | TODO comment near `render_sink`'s `Result` for `SourceMapError → SIFR-INTERNAL-*` conversion | Still open. Defer to `milestone_diag_4a`. |
| §C.2 follow-up | Generator file-path argument | Still open. Cosmetic. |
| §C.4 (writer-side `unwrap_or_default` on canonical bytes for `Float(NaN)`) | `DiagnosticArg::canonical_json_bytes` still uses `unwrap_or_default()` for non-finite floats | Still open. Public-API trap, no caller exists. Defer to `milestone_diag_2b`. |

None of these are introduced or worsened by pass 3. The `model/mod.rs` size has not grown further (the pass-3 deltas all live in `lib.rs`, `render/mod.rs`, the schema JSON, the architecture doc, and the issue artifact references — `model/mod.rs` is unchanged). Decomposition urgency is unchanged from pass 2.

---

## §C. New issues / regressions introduced in pass 3

None observed. The pass-3 deltas are:

1. `#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]` in `lib.rs` — cfg-gated, scoped to `test`, mirrors the existing `sifr_hir` pattern. No effect on production code.
2. Six `#[schemars(required)]` annotations on `Option<T>` fields — schemars v1 attribute, no runtime effect, only changes the generated schema's `required` array. Verified: writer output unchanged, round-trip test still passes, `deny_unknown_fields` still enforced.
3. Schema JSON regenerated to match — verified bit-identical to fresh generator output via `check_diagnostic_schema_sync.py`.
4. Architecture wording reword — pure docs.
5. Deletion of `…-review-blocked.md` and update of issue references — pure docs/process.
6. CRLF comment near `span_lines` — pure docs.

Risk surfaces I checked for and didn't find:

- **`#[schemars(required)]` interfering with serialization.** It's a schema-only attribute on `schemars` v1; `serde` ser/de behavior is unchanged. Verified by the round-trip test still asserting `"help":null` survives serialization.
- **Schema-required fields breaking external consumers.** Anyone validating against the *previous* schema would have been silently accepting payloads missing these fields — now they'll reject them. This is the desired behavior (the writer was already always emitting them as `null`), and the milestone is pre-production so no real consumer exists.
- **clippy carve-out leaking outside `cfg(test)`.** `cfg_attr(test, …)` only applies during `cargo test` builds; production `cargo build` / `cargo check` / non-test clippy runs continue to enforce the workspace-warn `unwrap_used` / `expect_used` lints on library code.
- **Architecture doc drift between sections.** The diagnostic-object bullet at line 720 now agrees with the bullet two below at line 722 ("Codegen/rustc diagnostics use … unmapped compiler failures use `SIFR-INTERNAL-*`") — both speak to target shape, not current emission state.
- **Issue-reference dangling links.** Verified — only `reviews/…-review-pass-1.md` and `reviews/…-review-pass-2.md` are referenced from the issue's Execution Status; no stale link to the deleted file.

---

## §D. Risk spot-checks (re-verified end-to-end after pass-3 fixes)

These were green in pass 2; reconfirming nothing slipped:

- **No public diagnostic types defined outside `sifr_diagnostics`** — `grep -rn "use sifr_diagnostics" crates/` returns only `crates/sifr_hir/src/lowering_outcome.rs:2`, which imports `SifrDiagnostic`. ✓
- **`Severity::Help` cannot exist** — `Severity` enum unchanged; `ChildSeverity { Note, Help }` is the correct location. ✓
- **`SourceDiagnostic` requires a span** — `primary_span: SourceSpan`, not `Option`. ✓
- **`ErrorEmitted` zero-size + unforgeable** — `assert_eq_size!(ErrorEmitted, ())` static assertion in `model/mod.rs`. ✓
- **`DiagnosticBuilder` `#[must_use]` and `!Clone`** — `assert_not_impl_any!(DiagnosticBuilder: Clone)` static assertion + `#[must_use]` attribute. ✓
- **JSON envelope `{ "version": 1, "diagnostics": [...] }`** — unchanged. ✓
- **Round-trip identity locked** — `rendered_diagnostic_json_round_trips_without_losing_arg_kinds_or_nulls` passes after schema tightening. ✓
- **Renderer is the only sort site** — `render_sink` sorts via `sort_by_cached_key` at [render/mod.rs:100-102](crates/sifr_diagnostics/src/render/mod.rs:100). ✓
- **`deny_unknown_fields` on consumed payloads** — present on `DiagnosticEnvelope`, `RenderedDiagnostic`, `RenderedDiagnosticChild`, `RenderedDiagnosticSuggestion`, `RenderedSuggestionEdit`, `DiagnosticSpan`, `DiagnosticSpanLine`. ✓
- **Schema sync gate** — `check_diagnostic_schema_sync.py` exits 0 against the regenerated schema. ✓

---

## §E. Required actions before merging `milestone_diag_1`

**Must-fix (block merge):**

1. **§E.1 — Update [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) Execution Status.** Two single-line edits:
   - Line 18: extend the review references list to include `reviews/semantic-diagnostic-code-taxonomy-diag-1-review-pass-3.md` (this file) and tick the checkbox once the pass-3 author confirms no further follow-up is requested.
   - Line 27: change `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings pending after pass-2 fixes.` to `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings passed.` (the gate is now actually clean — the wording lags reality by one pass).

   These are documentation hygiene, not code, but they're the load-bearing artifacts that say "milestone_diag_1 is reviewed and validated." Without them the issue still claims clippy is "pending."

**No code-level must-fix items remain.** All five pass-2 blockers are resolved at the code/doc/process level and confirmed by re-running the validation gate.

**Should-fix (track explicitly, not block merge):** unchanged from pass 2 §H — see §B above for the consolidated list. None of these have been worsened by pass 3.

**Optional polish (carry into follow-up issues, not this PR):**

- Wire `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings` into `scripts/run_all_tests.sh --profile pr` so the gate isn't manual-only.
- Add a `scripts/check_diagnostics_maintainability_guardrails.py` modeled on `scripts/check_hir_maintainability_guardrails.py` before `milestone_diag_2a` adds the registry.
- Decompose `model/mod.rs` per the pass-1 sketch before the registry doubles its size.

---

## §F. Summary

Pass 3 cleanly addresses every pass-2 must-fix:

- Clippy `--all-targets` is now clean (cfg_attr in lib.rs).
- Schema and writer agree on Option-field presence (schemars(required) + regenerated JSON).
- Architecture wording reflects target shape, not current emission state.
- Stale `…-review-blocked.md` removed; issue references corrected.
- CRLF human-renderer trap is documented in code at the load-bearing site.

No new code regressions, no fresh correctness issues, no changes to the should-fix surface. The only residual blocker is a documentation lag in the issue's Execution Status block (§E.1) — two single-line edits.

With those two issue-file edits, `milestone_diag_1` is mergeable.

— Pass-3 review complete.
