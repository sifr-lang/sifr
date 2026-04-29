# Final-Loop Review #2: Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics

This review evaluates [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) after the round-1 patch pass against [reviews/semantic-diagnostic-code-taxonomy-final-loop-review-1.md](semantic-diagnostic-code-taxonomy-final-loop-review-1.md). It also re-checks the proposal against the upstream artifacts:

- [reviews/source-map-diagnostics-ts-rust-assessment.md](source-map-diagnostics-ts-rust-assessment.md)
- [reviews/source-map-diagnostics-ts-rust-assessment-review.md](source-map-diagnostics-ts-rust-assessment-review.md)
- [reviews/phase-wide-diagnostics-ts-rust-assessment.md](phase-wide-diagnostics-ts-rust-assessment.md)
- [reviews/phase-wide-diagnostics-ts-rust-assessment-review.md](phase-wide-diagnostics-ts-rust-assessment-review.md)

User constraints applied unchanged: pre-production, no fallback compatibility, no historical compatibility, no global numeric error-code allocation, family-local `SIFR-<FAMILY>-0000..9999`, elegant compiler/language diagnostic architecture.

## Verdict

**Not yet satisfied.** Round 1's B1–B4 and M1–M7 are all integrated in the proposal text. The severity-aware cap admission policy, the `Result<T, ErrorEmitted>` driver shape with deletion of `CompileError` as a public abstraction, the full `DiagnosticBuilder` surface, the `LoweringError`/`TypeError`/`TypeErrorKind` deletion timeline, the schema-sync script, the skeleton-state validation rule, the active-only constants/helpers rule, the construction-and-render `SourceSpan` validation contract, the `UPPER_SNAKE_CASE` constant naming with canonical-string-accessor, the no-JSON-provenance internal-diagnostic policy, the `expect-error[col=…]` qualifier, the exact-case docs check, and the `CompilePhase::TypeCheck` deletion in `milestone_diag_4a` all appear as written.

But the round-1 patches introduced two new contradictions and exposed three previously-latent under-specifications. One is a hard milestone-sequencing contradiction in `milestone_diag_2b` that makes that milestone unmergeable as written; one is a missing `DiagnosticSink::emit` surface that the proposal references in three places without ever declaring; the rest are second-order ambiguities that will surface as renderer churn or migration churn after `milestone_diag_1` locks the sink/builder API.

Resolve B1–B2 before opening `milestone_diag_1`. M1–M6 are cheap to capture in the same edit pass. L1–L3 are forward-looking.

## Round-1 verification

For the record — these are all integrated correctly:

| Round-1 finding | Where | Status |
| --- | --- | --- |
| B1 (path-first sort + severity-aware cap admission) | proposal line 1156 | ✓ |
| B1 sub (exit code on unfiltered sink) | line 1155 | ✓ |
| B2 (delete `CompileError` as public abstraction; driver returns `Result<T, ErrorEmitted>`) | lines 420–427, 909 | ✓ |
| B3 (full `DiagnosticBuilder` surface incl. `cancel`) | lines 391–414, 416 | ✓ |
| B4 (`LoweringError` deletion timeline; `TypeError`/`TypeErrorKind` deletion) | lines 678, 868, 876, 900, 995, 996 | ✓ |
| M1 (schema sync script wired into `run_all_tests.sh`) | lines 693, 1073, 1087, 1090 | ✓ |
| M2 (skeleton-state validation tolerance) | line 731 | ✓ but see B1 below |
| M3 (Active-only constants and helpers rule) | lines 94, 767–768, 1127 | ✓ |
| M4 (`SourceSpan` validation timing — construct + render) | lines 658–659 | ✓ |
| M5 (`UPPER_SNAKE_CASE` constant + canonical-string-accessor) | line 94 | ✓ |
| M6 (no internal-diagnostic JSON provenance) | line 312 | ✓ |
| M7 (delete `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` in `4a`) | lines 782, 905–906, 1027 | ✓ but see M1 below |
| L1–L5 | lines 481, 180, 310, 811–816 | ✓ |

The round-1 contract is met. Findings below are new gaps surfaced by re-reading the proposal as a whole now that those fixes have settled.

## Blocking gaps

### B1 — `milestone_diag_2b` activation timing contradicts the migration sequence (High)

`milestone_diag_2a` line 731 says:

> The `milestone_diag_2a` validation test must pass for a skeleton registry with zero active codes. Active-code fixture, docs-page, and **non-test-emission checks become non-vacuous in `milestone_diag_2b`** when active entries are populated.

But `milestone_diag_11`'s emission guardrail (line 981) says:

> Every active registry code must appear through its canonical `DiagnosticCode::...` constant in non-test compiler source outside `sifr_diagnostics` itself. Textual presence is the decidable emission-path check; codes found only in tests or only in the registry crate must be marked `Reserved` or deleted.

If the non-test-emission check becomes non-vacuous in `2b`, it begins to enforce — and it must fail, because the actual migrations that introduce constant references in non-test compiler source happen later: `4a` (parser/workspace/codegen/build/test-runner), `6` (decimal), `7` (parser/name/import/type/call), and `8` (ownership/flow/match/class/protocol/result/stdlib). Between `2b` and `8`, every active code's constant exists in `crates/sifr_diagnostics/src/codes.rs` but nowhere outside it. The check fires; CI fails; `2b` cannot merge.

The two clauses cannot both be true.

The right resolution — consistent with how rustc's tidy gate works (the check is decidable and runs in the final state, but it is not gated to land at the moment the registry first lists codes) — is to split activation into two phases:

1. **Registry-internal checks** (constant/registry sync, canonical code form, template placeholder/args correspondence, docs-page presence) become non-vacuous in `2b`. These are decidable on the registry alone without inspecting compiler emission and pass cleanly the moment `2b` lands.
2. **Emission-presence check** activates per-family at the milestone where that family migrates: decimal in `6`, parser/name/import/type/call in `7`, ownership/flow/match/class/protocol/result/stdlib in `8`, and parser/workspace/codegen/build/test-runner in `4a`. The check is enforced wholesale in `11`'s residual cleanup.

Pick one of these wordings, but the current "non-test-emission checks become non-vacuous in `milestone_diag_2b`" must be deleted or qualified. As written, `2b` blocks itself.

A related sub-issue: `milestone_diag_2b` DoD line 765 says "Every active registry code has a representative fixture; reserved codes are explicitly marked Reserved and are exempt." For active codes whose emission migration lands in `7` or `8`, what does "representative fixture" mean during the window between `2b` and the migration milestone? Either the fixture asserts a code the compiler doesn't yet emit (which `5`'s harness validation forbids: "Validate fixture-asserted codes against the registry at harness load time" — registry presence is fine, but the fixture is an *e2e* fixture, not a registry probe), or the fixture is deferred to the migration milestone (in which case the `2b` DoD is wrong). State which.

The cleanest fix: representative fixtures are added by the migration milestone, not `2b`. `2b` records the *path* where the fixture will live in the registry record's `representative fixture path` field; the file does not need to exist yet. `milestone_diag_11`'s coverage check enforces that every active code's fixture path exists and asserts the code by phase end.

### B2 — `DiagnosticSink::emit(...)` is referenced in three places but never specified (High)

The proposal defines exactly one `DiagnosticSink` method:

```rust
impl DiagnosticSink {
    pub fn emit_error(&mut self, diag: SifrDiagnostic) -> ErrorEmitted {
        // validates Severity::Error, records the diagnostic, returns the proof
    }
}
```

But the proposal references a separate `emit` in three load-bearing places:

- Line 372: `LowerCtx::emit(...)` collects diagnostics during lowering.
- Line 416: "Dropping a builder without `build`, `emit`, return, or `cancel`..."
- Line 523: "It is consumed by `DiagnosticSink::emit(...)`, converted into a returned `SifrDiagnostic`, or explicitly cancelled..."
- Lines 501–506: every helper-call example (`ctx.emit(sifr_hir::name_resolution::diagnostics::undefined_variable(name, span))`).

`emit_error` validates `Severity::Error`. What method emits `Severity::Warning` and `Severity::Note`? `reveal_type(...)` notes (line 1150) and compiler warnings (line 1151) flow through "the same JSON envelope as errors" — they have to enter the sink somehow. Today the proposal does not say how.

This is not a small omission. The sink is the only legal channel from emission sites to renderers, and the must-use builder discipline routes through it. Pin the surface in `milestone_diag_1`. At minimum:

```rust
impl DiagnosticSink {
    /// Emit any non-error diagnostic. Severity must be Warning or Note.
    pub fn emit(&mut self, diag: SifrDiagnostic);

    /// Emit an error diagnostic and return an unforgeable proof.
    /// Severity must be Error.
    pub fn emit_error(&mut self, diag: SifrDiagnostic) -> ErrorEmitted;
}
```

Helpers and `LowerCtx::emit` are the ergonomic wrappers that dispatch to the right sink method based on `SifrDiagnostic.severity`. State whether `LowerCtx::emit` returns `Option<ErrorEmitted>` (`Some` for errors), or whether HIR callers that need the proof must call a separate `LowerCtx::emit_error` directly. Today's helper examples (`ctx.emit(undefined_variable(...))`) discard the proof — that is fine for the helper's call sites that don't taint values, but cascade-suppression sites need a typed path. Spell out which.

A corollary: the cap-omission summary diagnostics ("3 additional errors omitted by recovery cap", lines 1158) are themselves `SifrDiagnostic` values — they must enter the sink through one of these methods. State which, and which severity they carry. (`Severity::Note` is implied by line 1158, but that means they emit through `emit`, not `emit_error`, even though they describe error-bucket omissions. Worth confirming.)

## Medium-severity gaps

### M1 — `milestone_diag_4a` HIR/type-system migration scope is implicit (Medium)

`milestone_diag_4a` deletes the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` mapping (line 782). The same milestone explicitly migrates "parser adapters, workspace/project discovery, codegen boundaries, build/materialization/rustc diagnostics, and test-runner diagnostics" — HIR and type-system are not listed. Yet line 782 also says:

> Any still-unmigrated TypeCheck path must use an inventory-assigned canonical code through `SifrDiagnostic` transport or fail to compile; it must not fall back to a phase bucket until `milestone_diag_7` or `milestone_diag_8`.

That requirement is impossible to satisfy without `4a` also performing the mechanical migration of every previously-`TypeCheck`-routed call site (mostly in `sifr_hir` and `sifr_type_system`) into `SifrDiagnostic` transport with inventory-assigned codes. So `4a` does in fact migrate HIR/type-system — just at the *transport* level — and `7`/`8` then refine those migrations with category-specific helpers, related spans, and fixture coverage.

This is workable but currently implicit. The DoD becomes either vacuous or false depending on interpretation:

- DoD line 793 ("`CompilePhase::TypeCheck` no longer assigns `SIFR-TYPE-0001` to any diagnostic path") is met by the deletion.
- `milestone_diag_7` DoD line 872 ("The largest e2e fail categories no longer use `SIFR-TYPE-0001`") is *vacuously* true after `4a`, because nothing emits `SIFR-TYPE-0001` after `4a`'s deletion; `7`'s DoD does not actually measure `7`'s work.

State explicitly in `milestone_diag_4a`'s scope:

> `milestone_diag_4a` performs the mechanical transport migration of every previously-`CompilePhase::TypeCheck`-routed HIR and type-system call site to inventory-assigned `SifrDiagnostic` emission. `milestone_diag_7` and `milestone_diag_8` then refine those migrations with category-specific helpers, related spans, dedupe args, and fixture coverage. No call site falls back to a phase bucket between `4a` and `7`/`8`.

And rephrase `7`'s and `8`'s DoDs around what each milestone *adds* (specific helper modules, fixture coverage, related spans), not what it removes. The current "no longer use `SIFR-TYPE-0001`" wording is true after `4a` regardless of whether `7` or `8` ever runs.

### M2 — Must-use discipline does not cover `SifrDiagnostic` itself (Medium)

The discipline at lines 519–524 applies to `DiagnosticBuilder`:

> A diagnostic builder or `Diag` value is `#[must_use]`. It is not `Clone`. It is consumed by `DiagnosticSink::emit(...)`, converted into a returned `SifrDiagnostic`, or explicitly cancelled in tests/internal probes. Dropping a constructed diagnostic without emitting, returning, or cancelling it is a programmer bug.

But `DiagnosticBuilder::build(self) -> SifrDiagnostic` (line 411) returns a `SifrDiagnostic` value, and `SifrDiagnostic` carries no must-use marker and no drop bomb. The legal sequence:

```rust
let diag: SifrDiagnostic = builder.build();
// scope ends; diag is dropped silently. Discipline bypassed.
```

…compiles, panics nothing in debug, and emits nothing. The "is consumed by `DiagnosticSink::emit(...)`" half of the discipline is unenforceable for `SifrDiagnostic`s the moment they leave the builder.

rustc avoids this by having builders that own emission (`Diag::emit(self)`) and not exposing a public "build into a value, then emit later" path. The proposal picked a different shape — `build()` returns the value, the value flows to `sink.emit_error(diag)` — which means the must-use must extend to the value:

- Either: `SifrDiagnostic` is itself `#[must_use]` with a debug-mode `Drop` that panics if dropped without going through `DiagnosticSink::emit`/`emit_error` (requires a sentinel field that the sink consumes — implementable, but a design choice the model section should pin).
- Or: `DiagnosticBuilder::build()` is renamed to a path that hands the diagnostic directly to a sink, e.g. `builder.emit_into(&mut sink) -> Option<ErrorEmitted>`, and the standalone `build() -> SifrDiagnostic` is reserved for tests/internal probes only — analogous to `cancel`.

Pick one in `milestone_diag_1`. The current shape lets a contributor drop a `SifrDiagnostic` silently and the discipline has no language defending against it.

A related sub-point: line 416 says "Dropping a builder without `build`, `emit`, return, or `cancel` follows the diagnostic emission discipline below." `build` is listed alongside `emit` and `return` as a legal terminator — but `build` produces a value that itself can be dropped. So `build` is only a legal terminator if the *resulting value* is also covered. Tighten the wording.

### M3 — Cap omission summaries have no registry code (Medium)

Line 1158 says when the cap omits diagnostics, rendering appends:

> structured `Severity::Note` summaries with omission counts per severity bucket, such as `3 additional errors omitted by recovery cap` and `10 additional reveal_type results omitted by recovery cap`.

These are `SifrDiagnostic` values (per the "structured" qualifier and the one-stream commitment). Every `SifrDiagnostic` must have a `DiagnosticCode` per the model. Which code?

Three plausible options:

1. Reserve a dedicated registry entry, e.g. `SIFR-INTERNAL-0002` for "recovery cap omission summary" with the omitted count as a scalar arg. This is consistent with the proposal's existing `SIFR-INTERNAL-0001` "stable catch-all" reservation (line 1140) and matches the pattern of a stable structured summary.
2. Emit the summary as a non-coded text artifact outside the diagnostic stream. This breaks the one-stream commitment ("Warnings and notes appear in the same JSON envelope as errors", line 1152) and reintroduces a side channel — directly against the proposal's stated direction.
3. Reuse an existing code (e.g. `SIFR-INTERNAL-0001`). Reusing the catch-all for a structured summary blurs the catch-all's semantics and makes the registry meaning sloppier.

Option (1) is the only one consistent with the rest of the architecture. Reserve the code in `milestone_diag_2a`'s skeleton (it can land as a `Reserved` entry) and activate it in `milestone_diag_10`'s recovery work, where the cap admission policy lives.

State this in §"Non-Error Diagnostics" or §"Internal code allocation policy". As written, contributors will end up writing the summary as a free-text emitter with no code, or fight over which existing code to reuse.

### M4 — Internal diagnostics are silently capped behind source errors (Medium)

The cap admission policy (line 1156) admits errors first in canonical sorted order. The canonical sort puts source diagnostics before internal diagnostics (line 582: "Internal diagnostics sort after source diagnostics"). Internal diagnostics use `Severity::Error` (line 677). So:

- 50 source errors fill the cap before any internal error is admitted.
- Internal errors emitted in the same compilation are dropped from rendering.
- Exit code is non-zero (per line 1155, computed on unfiltered sink) — but the user sees no internal diagnostic, only source errors.

This is bad because internal diagnostics represent ICEs and broken compiler invariants. A user encountering an ICE behind 50 source errors gets no signal that the compiler also imploded. They will assume the source errors caused the failure and not file an internal-bug report.

Three ways to handle:

1. **Carve internal diagnostics out of the cap.** Admit all internal diagnostics after the source-diagnostic cap is applied. Source errors are bounded by 50; internal errors are unbounded but they already represent compiler bugs. (This matches rustc's behavior: ICE output is not subject to error-cap truncation.)
2. **Always admit internal errors first.** Insert internal errors into the cap before source errors. Source errors then compete for the remaining slots. This makes ICEs the most visible thing in the output, which is the right priority but reorders error output unfamiliarly compared to source-only runs.
3. **Document the trade-off.** Accept that internal diagnostics can be capped and pin a note ("if more than 50 source errors are emitted, internal diagnostics are summarized only by count").

Pick one in §"Non-Error Diagnostics". Option 1 is the cleanest — internal diagnostics always render, source diagnostics are capped — and matches both rustc's behavior and Sifr's stated commitment that internal diagnostics describe compiler bugs that should never be silently dropped.

### M5 — Ordering tuple's args comparison is not pinned (Medium)

The Diagnostic Ordering Policy (line 587) says:

> `args` are compared by stable key order and scalar value rendering, never by map iteration order.

But "scalar value rendering" is ambiguous when the arg variants (per line 292) are `String | Signed | Unsigned | Float | Bool`:

- `Float(f64)` comparison: total order on `f64` is undefined for NaN. Use `f64::total_cmp`? Canonical-JSON `5.0` vs `5`?
- `Bool(true)` vs `Unsigned(1)` vs `Signed(1)` — three different `DiagnosticArg` variants that render to similar strings. Are they equal under "scalar value rendering"? Or compared by variant tag first?
- `String("foo")` vs `Unsigned(0xfoo)` — what happens if a `String` and a non-`String` collide?

Without a pinned canonical form, two implementations of the comparator can produce different ordering for the same canonical stream. Pin one. The phase-wide review's R1 explicitly recommended "canonical-JSON-serialized args (byte-compared)". That's still the right call — it's content-addressable, deterministic across machines, and matches the "JSON is the wire format" commitment elsewhere in the proposal.

Replace line 587 with:

> `args` are compared as canonical JSON: keys in `BTreeMap` order, scalar values serialized by their JSON form (`String → "..."`, `Signed → number`, `Unsigned → number`, `Float → number with `f64::total_cmp` total order on equal-rendered NaN`, `Bool → true/false`), and the resulting byte string is compared lexicographically.

Add a unit test in `milestone_diag_1` that constructs two diagnostics differing only in args and asserts the ordering result.

### M6 — "Source diagnostic without display path" clause is dead or contradictory (Medium)

Line 582 in the ordering policy:

> `primary display path` is the source-map display path, compared lexicographically. Source diagnostics without a display path sort after source diagnostics with a display path. Internal diagnostics sort after source diagnostics.

But:

- `SourceDiagnostic` has `primary_span: SourceSpan` (line 239), not optional.
- `SourceSpan { source_id, range, lowered_from }` (line 603) — `source_id` is mandatory.
- Source-map records register sources with `display path` (line 595) at registration time.

If every source diagnostic has a `SourceSpan` with a registered `source_id`, every source diagnostic has a display path. The "source diagnostics without a display path" branch is never reached.

Either:

- Delete the clause. Source diagnostics always have a display path.
- Document the case. If the source-map can register a source without a display path (e.g., synthetic stdin without a filename, or a path-stripped-by-display-policy case), say so and add a unit test exercising it.

The first is cleaner and matches the proposal's stronger "source diagnostics cannot be constructed without a `SourceSpan`" requirement at line 695. Keep the "internal diagnostics sort after source diagnostics" half — that's load-bearing.

## Low-severity notes

### L1 — `SourceSpan` validation should also fire at sink emit time

Line 659 says span validation runs at `SourceSpan::new` (debug only) and at render-boundary lowering. The `DiagnosticSink::emit_error` path also accepts a `SifrDiagnostic` containing a `SourceSpan`. Validating at sink-accept time would fail-fast at emission rather than waiting for render — a stale `SourceSpan` (e.g. one whose source was unregistered) could pass `SourceSpan::new` validation in debug, then be carried through emission and only fail at JSON serialization. Adding sink-accept validation costs one source-map lookup per emit and converts a render-time bug into an emit-time bug, where it is closer to the call site.

This is a low-priority polish — the existing two-point validation is sufficient to keep the JSON wire honest — but worth a one-line addition to §"Source Mapping Architecture" if the cost-benefit lands favorably during `milestone_diag_1` implementation.

### L2 — Canonical-string accessor name is unspecified

Line 94 says:

> validation checks that each constant's canonical string accessor returns the registry id

…but does not name the accessor. `DiagnosticCode::TYPE_ASSIGNMENT_MISMATCH.canonical()`? `.as_str()`? `.code()`? Naming it now in §"Diagnostic Identity Policy" or in `milestone_diag_2a`'s registry record description avoids a contributor PR cycle on bikeshedding. Suggested: `pub fn code(&self) -> &'static str` returning the `SIFR-FAMILY-NNNN` form.

### L3 — `DiagnosticBuilder::cancel(self)` consumption status is correctly stated but worth one explicit test

Line 416 ("`cancel(self)` is the only legal way to discard a builder without building, emitting, or returning") and line 694 ("the only legal non-emitting consumption path") agree. Round 1's B3 sub-point asked whether `cancel` counts as "consume" for the discipline; the proposal answers yes by listing it among the legal terminators at line 416. Worth one unit test in `milestone_diag_1` that constructs a builder, calls `cancel`, and asserts no diagnostic was sent to the sink and no debug-mode panic fired. This is just a confirmation case, not a gap.

## Implementation-order reassessment

The sequencing graph (lines 1011–1025) remains internally consistent at the milestone level:

```
diag_1 → diag_2a → diag_3 → diag_2b → diag_4a → diag_6 → diag_5 → diag_7 → diag_8 → diag_4b → diag_9 → diag_10 → diag_11
```

Two of round 1's flagged risks remain unresolved as a result of the new findings:

- **B1 above** — `2b`'s self-blocking validation activation forces a real choice: either rephrase the activation to delay emission-presence checks past `2b`, or move the constants out of `2b` and into the per-family migration milestones. The first is less surgical; the second changes the milestone graph.
- **M1 above** — `4a` quietly migrates HIR/type-system transport even though its scope text doesn't say so. Explicit scope language closes this.

`4a`'s scope is also large (renderers + HIR `LoweringError` retirement + parser/workspace/codegen/build/test-runner migration + `CompilePhase::TypeCheck` deletion). Per AGENTS.md ("Prefer small, reviewable PRs with clear validation"), `4a` will land as a multi-PR sub-phase. State this. A sequence inside `4a`:

1. Renderer trait changes (consume `SifrDiagnostic`, no message inference).
2. `LoweringError` user-facing replacement with `LoweringOutcome`/`DiagnosticSink`.
3. Parser/workspace/codegen/build/test-runner migration.
4. `CompilePhase::TypeCheck` deletion + HIR/type-system mechanical transport migration.

Each step is independently reviewable and validates. Splitting `4a` along these lines does not change the milestone graph but documents the expected PR cadence.

## Cross-checks against earlier review patches (still confirmed integrated)

For audit completeness, these prior findings remain correctly integrated and need no further edit:

- **F1–F10** from the source-map review (must-use builders, per-span labels, JSON snippet text, on-disk byte offsets, 1-based UTF-8 char columns, 4-variant applicability, dedup vs grouping separation, `lowered_from`, source-map record fields, multibyte tests).
- **R1–R12** from the phase-wide review (path-first ordering tuple + insertion order, decidable registry/emission/fixture rules, central path normalization + duplicate baseline detection + fixture-grammar contradiction rule + single-sorted-stream test, domain-local constructors, pending domain objects, reservation-only tooling routing, no replacement-text-in-help, no nested coded chains, `ErrorEmitted` typed proof in `milestone_diag_1`, build-time `#[test]` validation, insertion order recorded by sink, `expect-error` grammar declared).

## Bottom line

The round-1 patches landed and the proposal's *content* is now defensibly close to a clean target architecture. The remaining issues are not directional disagreements — they are second-order under-specifications that round 1 didn't touch and that the round-1 patches partially exposed:

- B1 is a hard contradiction in `milestone_diag_2b` activation timing that makes that milestone unmergeable.
- B2 is an undefined `DiagnosticSink::emit` surface that the proposal references in three load-bearing places.
- M1–M6 are clarifications around must-use enforcement on `SifrDiagnostic` itself, the cap-summary diagnostic identity, internal-diagnostic visibility under the cap, the args ordering function, the dead "no display path" clause, and the implicit HIR/type-system migration in `4a`.

Resolve B1 and B2, fold M1–M6 into the same edit, and the proposal is mergeable into `milestone_diag_1`. None of these findings re-open the model or the sequencing graph at a structural level — they pin behavior the proposal currently leaves to interpretation.
