# Final-Loop Review #1: Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics

This review evaluates the current state of [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) after the source-map and phase-wide TS/Rust patches have been folded in. It cross-checks against:

- [reviews/source-map-diagnostics-ts-rust-assessment.md](source-map-diagnostics-ts-rust-assessment.md) (F1–F10)
- [reviews/source-map-diagnostics-ts-rust-assessment-review.md](source-map-diagnostics-ts-rust-assessment-review.md)
- [reviews/phase-wide-diagnostics-ts-rust-assessment.md](phase-wide-diagnostics-ts-rust-assessment.md) (Lessons 1–9)
- [reviews/phase-wide-diagnostics-ts-rust-assessment-review.md](phase-wide-diagnostics-ts-rust-assessment-review.md) (R1–R12)

User constraints applied: pre-production, no fallback compatibility, no historical compatibility, no global numeric error-code allocation, family-local `SIFR-<FAMILY>-0000..9999`, elegant compiler/language diagnostic architecture.

## Verdict

**Not yet satisfied.** The proposal has integrated almost all prior findings — F1–F10 from the source-map loop and R1–R12 from the phase-wide loop are visibly woven in (must-use builders, `ErrorEmitted`, ordering policy, pending domain objects, domain-local helpers, baseline hygiene, build-time registry validation, retired-constant deletion, no chain diagnostics, etc.). The shape of the canonical model, registry, source map, and milestone graph is now defensibly close to a clean target architecture.

But four genuine blocking gaps remain, plus a handful of medium-severity under-specifications that will surface as model/schema/sequencing churn after `milestone_diag_1` locks the surface. The most serious is an internal contradiction between the Diagnostic Ordering Policy and the recovery-cap rule — left unresolved, it will produce non-deterministic or surprising user output the moment a fixture exceeds 50 diagnostics.

Block `milestone_diag_1` until B1–B4 are resolved. M1–M7 are cheap to land in the same edit pass.

## Blocking gaps

### B1 — Ordering policy contradicts the recovery-cap rule (High)

`§"Diagnostic Ordering Policy"` (proposal lines 526–548) defines the canonical sort key:

```
(primary display path, primary byte_start, primary byte_end, severity_rank,
 diagnostic_kind_rank, code, message_template, args, insertion_order)
```

— path-first, then position, then severity. This is the TypeScript-style key R1 endorsed.

But `§"Non-Error Diagnostics"` (line 1092) states:

> The 50 top-level recovery cap applies to all top-level diagnostics after severity ordering, while the existing user-error exit behavior remains based on whether any top-level diagnostic has `Severity::Error`.

"After severity ordering" is severity-first; the canonical sort is path-first. With path-first ordering and a 50-cap by truncation, you can fill the cap with `Severity::Note` (`reveal_type(...)`) entries from an early file and silently omit `Severity::Error`s from a later file. With severity-first ordering, you violate the canonical sort key that all three renderers and fixture baselines consume.

The two clauses cannot both be true.

The right fix — consistent with how rustc and TypeScript both behave in practice and with the proposal's own commitment that errors are the load-bearing diagnostic — is to keep the path-first canonical sort as the single source of truth and project a severity-aware view only for cap application:

1. Sort the canonical stream by the path-first key from line 533.
2. Apply the cap by walking the sorted stream and admitting up to 50 diagnostics, but partition the cap so errors are never displaced by warnings/notes. Concretely: fill from `Severity::Error` first in canonical order, then `Severity::Warning`, then `Severity::Note`, until 50 are admitted; emit a separate omission-count summary per severity bucket (the proposal already requires "the summary must say how many explicit reveal results were omitted").
3. Render the admitted set in canonical order. Renderers do not re-sort.

Other resolutions are acceptable, but the proposal must pick one. The current text is internally contradictory and will block deterministic fixture baselines from existing.

A separate sub-issue: `§"Non-Error Diagnostics"` says "Warnings do not affect the exit code; invocations with warnings only exit `0`" (line 1090) — but does not state how the cap interacts with this. If the cap drops all errors (impossible under fix #2, possible under naive truncation), exit code is still non-zero because the unfiltered sink saw an `Error` before truncation. State that explicitly: exit code is computed on the unfiltered sink, not the cap output.

### B2 — `CompileError` final shape is left as "either A or B" (High)

`milestone_diag_4b` (line 853) says:

> Convert `CompileError` into either a structured diagnostic wrapper or an internal boundary error that already carries `SifrDiagnostic`.

This binary "either/or" is exactly the kind of late-bound design decision that becomes a fallback path. In a no-fallback world, `CompileError` either:

- (a) becomes `struct CompileError(Vec<SifrDiagnostic>)` (or `NonEmptyVec`) — a thin transport at the driver boundary that carries already-canonical diagnostics, used to short-circuit `Result` flow without recreating a separate code source, **or**
- (b) is deleted entirely; the driver returns `Result<T, ErrorEmitted>` (the typed proof from `milestone_diag_1`), and the canonical diagnostics are read from the sink at the boundary.

Pick one before `milestone_diag_4b` opens. Each has implications for the public surface of `sifr_driver`, for `sifr` (CLI), and for the test runner. (b) is more elegant and matches rustc's `ErrorGuaranteed`-returning pass functions; (a) is mechanical and easier to migrate. A "structured diagnostic wrapper" in name only — `CompileError { diagnostic: SifrDiagnostic }` — would also re-create the single-code-per-error-path coupling that the proposal explicitly rejects (`CompileError` would become a hidden code source again because `Display` impls leak), so the wrapper variant must specifically be `Vec<SifrDiagnostic>` if chosen.

The proposal's own hard rule "Do not define public diagnostic types outside `crates/sifr_diagnostics`" (line 1063) further narrows the choice — option (a) only works if `CompileError` does not implement `Display` and does not derive any code from its own type. Pin this.

### B3 — `DiagnosticBuilder` API is named but not specified (High)

`SifrDiagnostic`, `SourceDiagnostic`, `InternalDiagnostic`, `RelatedSpan`, `DiagnosticChild`, `DiagnosticSuggestion`, `SuggestionEdit`, `SuggestionApplicability`, `DiagnosticArg`, `ChildSeverity`, `SourceId`, `SourceSpan`, `DiagnosticSpan`, and `DiagnosticSpanLine` are all sketched in the model section. `DiagnosticBuilder` is referenced by name in `§"Diagnostic Builder API"` (line 469) and `milestone_diag_1` (line 633) but its surface is never written down.

Every domain helper in the proposal — `sifr_hir::name_resolution::diagnostics::undefined_variable(name, span)`, etc. — has to construct a builder. If the builder type is not pinned in `milestone_diag_1`, every domain crate writes the helper against a guessed surface and the model's `#[must_use]` / non-`Clone` / cancel discipline becomes a runtime convention rather than a typed contract.

Sketch it alongside `SifrDiagnostic` in `§"Target Architecture"`. At minimum:

```rust
#[must_use]
pub struct DiagnosticBuilder {
    // private fields
}

impl DiagnosticBuilder {
    pub fn source(code: DiagnosticCode, severity: Severity, primary_span: SourceSpan) -> Self;
    pub fn internal(code: DiagnosticCode, severity: Severity) -> Self;
    pub fn message_template(self, template: &'static str) -> Self;
    pub fn arg(self, name: &'static str, value: impl Into<DiagnosticArg>) -> Self;
    pub fn related(self, span: SourceSpan, kind: RelatedKind, label: Option<String>) -> Self;
    pub fn child(self, severity: ChildSeverity, message: impl Into<String>) -> Self;
    pub fn help(self, help: impl Into<String>) -> Self;
    pub fn suggestion(self, suggestion: DiagnosticSuggestion) -> Self;
    pub fn build(self) -> SifrDiagnostic;
    pub fn cancel(self); // explicit drop, only legal in tests/internal probes
}

// Drop impl: panics in debug, routes a SIFR-INTERNAL-* in release
// (per proposal §"Diagnostic Emission Discipline")
```

Without this in the proposal, contributors will discover the builder shape through PRs and the API will diverge from what the helpers and the must-use discipline assume.

A related sub-point: `DiagnosticBuilder::cancel(self)` is the only legal way to drop a builder without emit/build. The proposal says drop without consume is "a programmer bug" (line 480) — but does `cancel` count as consume? It must, otherwise tests cannot construct probe diagnostics. Spell that out.

### B4 — `LoweringError` is never fully retired (High)

`milestone_diag_1` (line 632):

> Add the canonical `LoweringOutcome` and `DiagnosticSink` types alongside the existing `LoweringError`. `LoweringError` becomes private transitional plumbing only and is removed from user-facing paths in `milestone_diag_4a`.

`milestone_diag_11` hard rule (line 937):

> The codebase must have no user-facing `LoweringError { message, line, col }` style path.

These cover the *user-facing* path. They do not say when `LoweringError` is fully deleted from the codebase. A "private transitional plumbing" type that survives the entire migration is exactly the kind of latent fallback the no-compatibility contract forbids — a helper that emits a `LoweringError` instead of a `SifrDiagnostic` would silently bypass the whole system.

By `milestone_diag_8` all HIR semantic emissions have been migrated to canonical diagnostics, which means `LoweringError` has no internal callers either. State explicitly that `LoweringError` is deleted as part of `milestone_diag_8` (or, at the latest, `milestone_diag_11`'s residual cleanup) so the type does not live indefinitely as "transitional." Add a `milestone_diag_11` guardrail equivalent to "the symbol `LoweringError` does not exist in the workspace."

The same lifecycle question applies to `sifr_type_system::TypeError` and `TypeErrorKind`: `milestone_diag_7` retires them as user-facing types, but the proposal should explicitly say they are *deleted* (not deprecated) by the end of `milestone_diag_7`. Currently it says "retire" and "any short-lived adapter from `TypeError` to `SifrDiagnostic` must be deleted in this milestone" (line 814) — clarify that `TypeError`/`TypeErrorKind` themselves are also deleted, not just the adapter.

## Medium-severity gaps

### M1 — JSON schema sync has no enforcement script in the validation list

`milestone_diag_1` requires the JSON schema to be checked in and generated from `schemars` (line 629). `milestone_diag_11`'s guardrails state "the JSON schema is checked in and synchronized with the Rust model" (line 942). But `§"Validation Plan"` (lines 1004–1027) lists docs sync, code coverage, and baseline hygiene scripts — and not a schema sync check. Either:

- Fold schema regeneration into `scripts/check_diagnostic_docs_sync.py`, or
- Add `scripts/check_diagnostic_schema_sync.py` and wire it into `scripts/run_all_tests.sh`.

Without an explicit checker, schema drift surfaces only when a JSON consumer breaks.

### M2 — `milestone_diag_2a` validation test must tolerate the skeleton state

`milestone_diag_2a` lands the registry skeleton with only family reservations, before `milestone_diag_3` populates active codes. The build-time validation `#[test]` (line 681) checks "template placeholders against declared args, JSON-only arg declarations, docs-page presence for active codes, constant/registry sync, canonical code forms, and registry state validity." Most of these are vacuous on an empty active-code set, but "constant/registry sync" can fire if `DiagnosticCode` constants don't exist yet.

State explicitly: in `milestone_diag_2a` the validation test passes against an empty `Active` population; the active-code rules fire only once `milestone_diag_2b` populates entries. This is a one-line addition that prevents the skeleton milestone from getting blocked on its own validator.

### M3 — Helpers for `Reserved` codes would falsely satisfy the emission check

`milestone_diag_11` requires every active code's canonical `DiagnosticCode::...` constant to appear in non-test sources outside `sifr_diagnostics` (line 924). `Reserved` codes are exempt — and that's correct.

But: a domain helper that wraps a `Reserved` code's constant *would* appear in non-test source, which means a future contributor who pre-builds a helper before activating the code can confuse the emission check. State the policy:

- A `DiagnosticCode` constant exists only for `Active` codes (already implied at line 717 — make it a hard rule).
- A domain helper exists only for an `Active` code. Pre-helper drafts live behind a `Reserved` registry entry without a constant.
- The emission check measures *constant presence*, which is decidable and matches Rust tidy's textual approach.

The R2 fix in the prior review covered three rules; add this fourth rule to make them consistent with the helper architecture from R4.

### M4 — `SourceSpan` validation timing is unspecified

`milestone_diag_1` says the source map "validates source spans" (line 627) and the DoD lists "invalid span rejection" tests (line 651). But: at construction time, at emission time, or both? In debug, in release, or always?

A reasonable rule (matching rustc) is: the source map validates a `SourceSpan` against its registered source on first conversion to `DiagnosticSpan` (the JSON/render boundary), and `SourceSpan::new(source_id, range)` panics in debug if the range exceeds the source's byte length. State that in `§"Source Mapping Architecture"` so the test list has a target.

### M5 — `DiagnosticCode` Rust-identifier naming convention is implicit

The proposal shows `DiagnosticCode::TYPE_ASSIGNMENT_MISMATCH` for `SIFR-TYPE-0002`. The canonical wire form `SIFR-FAMILY-NNNN` (with hyphens and digits) is not a legal Rust identifier, so the constant name is necessarily a different string from the canonical code value. The build-time validation rule "code constant names match the canonical `SIFR-FAMILY-NNNN` form" (R10's bullet) is then literally false unless reinterpreted.

State the mapping convention in `§"Diagnostic Identity Policy"` or `milestone_diag_2a`:

- The canonical value (e.g. `"SIFR-TYPE-0002"`) is the *string value* of the constant and the JSON wire form.
- The Rust constant name is `UPPER_SNAKE_CASE`, human-readable, and chosen by the registry author. It encodes the rule, not the number.
- The validation test asserts `DiagnosticCode::FOO_BAR.canonical() == "SIFR-FAMILY-NNNN"`, where each constant has a `canonical()` (or equivalent) accessor.

This prevents a contributor from naming a constant `T0002` or matching the wire form literally.

### M6 — Internal diagnostic provenance is not specified

`InternalDiagnostic` (lines 244–252) has no `primary_span` and no `related_spans`, only message/template/args/children/help. That's correct — internal failures may not have meaningful source mapping.

But where does the panic backtrace or compiler-side context go? The proposal allows children for internal diagnostics but doesn't say whether a stack frame, a HIR node id, or any other provenance is permitted. Pin one:

- Provenance other than `code`, message, template, args, children, and help is not part of the JSON wire format for internal diagnostics; debug-mode renderers may attach a backtrace as a `ChildSeverity::Note` child, and release-mode does not.
- Or: add a typed `provenance: Option<InternalProvenance>` field with an enum of allowed provenance kinds.

The first is simpler and consistent with one wire format. Pick one.

### M7 — `milestone_diag_4a` deletion order for message-prefix classifiers is fragile

`milestone_diag_4a` (line 730) requires removal of "workspace message-prefix code inference such as `message.starts_with(\"could not resolve import \")`" but lands *before* `milestone_diag_4b` deletes `CompilePhase` and the phase-derived public mapping (line 850). Between `4a` and `4b`, the workspace inference is gone but `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` still exists. The `milestone_diag_7` DoD says "the largest e2e fail categories no longer use `SIFR-TYPE-0001`" (line 818) — which holds if every type-check call site has migrated, but the residual `CompilePhase::TypeCheck` mapping still routes anything that did *not* migrate to the catch-all.

State explicitly that during `milestone_diag_4a` through `milestone_diag_8`, any HIR or type-system path that has not yet migrated must emit through `SifrDiagnostic` with a specific code via the inventory-assigned target — not through `CompilePhase::TypeCheck` — and that the inventory in `milestone_diag_3` is the source of truth for which sites have been migrated. Otherwise the `milestone_diag_7` DoD is vacuously true: "no longer uses `SIFR-TYPE-0001`" because the migrated sites don't, while non-migrated sites still do.

The simplest and cleanest mitigation is to delete `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` in `milestone_diag_4a` and accept that any unmigrated site fails to compile until it is migrated in `7`/`8`. This converts the silent-fallback risk into a build break, which matches the no-fallback contract. Reorder accordingly or state the explicit exemption.

## Low-severity notes

### L1 — `lowered_from: Option<Box<SourceSpan>>` is recursive owning

This is correct in shape but each desugared HIR node now owns a heap allocation. For HIR, this is fine. For tight inner loops, prefer `Option<SourceSpanRef<'_>>` or arena-interned spans later. Worth a one-line note that the current shape is correct for the diagnostic boundary but may be revisited if HIR storage profiling shows it.

### L2 — Documentation URL casing on case-insensitive filesystems

`§"Documentation URL Policy"` (line 178) says "Documentation URLs and filenames use the canonical uppercase code form" and "URL is case-sensitive; generated filenames must match canonical code casing even on case-insensitive filesystems." Good.

The build/check step on macOS APFS (case-insensitive default) cannot detect a casing-only difference via `fs::metadata` alone. State that the docs-sync check uses `read_dir` and compares exact-case filename strings, not metadata hits. This is one line and prevents a silent macOS-only regression.

### L3 — `RelatedKind::Note` vs `ChildSeverity::Note` overlap

`RelatedKind::Note` (a labeled span) and `ChildSeverity::Note` (a free-form note text) are intentionally different concepts. A future contributor will conflate them. Add a one-paragraph clarification in `§"Target Architecture"` saying:

- `RelatedSpan` with `RelatedKind::Note` carries a span and an optional label; it points at code.
- `DiagnosticChild` with `ChildSeverity::Note` carries free text without a span.
- A note that wants both — text *and* a span — uses two values: a `RelatedSpan` and a `DiagnosticChild`, or a single `RelatedSpan` with a label that conveys the text.

### L4 — `expect-error` grammar's "optional span qualifier" is unspecified

`milestone_diag_5` (line 762) writes the canonical grammar as `expect-error: SIFR-<FAMILY>-dddd` and adds:

> An optional span qualifier may be added only if the existing fixture format needs it to disambiguate multiple diagnostics at one line. Message-substring matchers are not part of the grammar.

"May be added" is a deferred decision. If multiple diagnostics on one line are possible (common: a single `f()` call producing both `SIFR-CALL-0001` and `SIFR-NAME-0001`), the qualifier is necessary, not optional. Either:

- State the qualifier syntax now (e.g. `expect-error[col=12]: SIFR-CALL-0001`), or
- State that two diagnostics on one line require two annotation lines and forbid a qualifier.

Either is fine; the current "may be added" language defers a fixture-grammar decision to fixture-author whim.

### L5 — `sifr_diagnostics` dependency surface

`§"Dependency Ownership"` (line 437) says `sifr_diagnostics` may depend on `serde` and `ruff_text_size`. This is correct — `ruff_text_size` is the right type for byte ranges given the Ruff-fork parser. State explicitly that the version is workspace-pinned to match `sifr_python_parser`'s Ruff version (currently 0.15.12 per `AGENTS.md`), so a Ruff upgrade does not silently change `TextRange` semantics inside diagnostics.

## Cross-checks against earlier patches (confirmed integrated)

For the record, these earlier findings are folded in correctly and need no further edit:

- **F1** (must-use, non-clone, drop discipline) — `§"Diagnostic Emission Discipline"` lines 472–490.
- **F2** (per-span labels and `is_primary`) — `RelatedSpan` with `kind` and `label`, JSON flat `spans` array per diagnostic (line 607).
- **F3** (JSON span text/lines snippet) — `DiagnosticSpanLine { text, highlight_start, highlight_end }` lines 578–583.
- **F4** (on-disk byte offsets, no normalization for current scope) — line 603.
- **F5** (1-based UTF-8 character columns in JSON) — line 605.
- **F6** (`SuggestionApplicability` 4-variant enum + multipart edits) — lines 274–288.
- **F7** (recovery dedup vs compact grouping keys separated) — lines 510–525, with `dedupe args` defined as a registry-declared subset.
- **F8** (`lowered_from: Option<Box<SourceSpan>>`) — line 562.
- **F9** (canonical_path + content_hash in source-map record) — lines 593–597.
- **F10** (multibyte 4-byte-emoji JSON consistency test) — `milestone_diag_11` line 932.
- **R1** (path-first canonical ordering tuple + insertion order) — `§"Diagnostic Ordering Policy"` lines 526–548. (See B1 for the unresolved interaction with the cap.)
- **R2** (decidable registry/emission/fixture rules + retired-constant deletion) — line 1031.
- **R3** (central path normalization, duplicate baseline detection, fixture-grammar contradiction rule, single-canonical-stream three-renderer test) — `milestone_diag_5`.
- **R4** (domain-local constructors; `sifr_diagnostics` is not a monolithic helper crate) — lines 217–222 and `§"Diagnostic Builder API"` lines 454–469.
- **R5** (pending domain objects, explicit alternative to stashing) — `§"Pending Domain Objects"` lines 484–490.
- **R6** (tooling routing reservation-only, no LSP/code-action validation in this phase) — `milestone_diag_2a` line 679.
- **R7** (`Help` children must not contain literal replacement text) — `milestone_diag_11` guardrail line 941.
- **R8** (no nested coded chains; `RelatedSpan` + `DiagnosticChild` only) — line 306.
- **R9** (`ErrorEmitted` typed proof in `milestone_diag_1`, not deferred to `milestone_diag_10`) — lines 372–381 and 634.
- **R10** (build-time `#[test]` validation in `sifr_diagnostics`) — line 681.
- **R11** (insertion order recorded by `DiagnosticSink`) — line 635 and 649.
- **R12** (`expect-error` grammar declared) — line 759. (See L4 for the residual span-qualifier ambiguity.)

## Implementation-order assessment

Sequencing graph (line 952–966) is sound. `diag_2a → diag_3 → diag_2b` correctly delays population until inventory exists. `diag_6` before `diag_5` correctly avoids a transitional `[Edddd]` window in the test harness. `diag_4b` after `diag_8` correctly closes the `CompilePhase` retirement after all migrations are complete — *if* B1, B2, and M7 are resolved.

The only remaining sequencing risk is M7: between `diag_4a` and `diag_8`, any unmigrated path that would have routed through `CompilePhase::TypeCheck` is in an undefined state. Either (a) delete the mapping in `4a` and accept a build break for any unmigrated site, or (b) explicitly call out the partial state in the milestone DoDs. (a) is the no-fallback choice and is recommended.

## Bottom line

This is a strong, near-final proposal. The integrated patches cover the bulk of TS/rustc lessons that survived contact with the no-fallback contract. The remaining issues are not directional — they are the last few "decide one of two" punts that, left unresolved, will produce non-deterministic output (B1), late-breaking driver redesign (B2), helpers built against an unspecified surface (B3), or a transitional type that lives forever (B4).

Resolve B1–B4 and fold M1–M7 into the next edit. After that, opening `milestone_diag_1` is appropriate.
