# Final-Loop Review #3: Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics

This review evaluates [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) after the round-2 patch pass against [reviews/semantic-diagnostic-code-taxonomy-final-loop-review-2.md](semantic-diagnostic-code-taxonomy-final-loop-review-2.md). It also re-checks the proposal against the upstream artifacts:

- [reviews/source-map-diagnostics-ts-rust-assessment.md](source-map-diagnostics-ts-rust-assessment.md)
- [reviews/source-map-diagnostics-ts-rust-assessment-review.md](source-map-diagnostics-ts-rust-assessment-review.md)
- [reviews/phase-wide-diagnostics-ts-rust-assessment.md](phase-wide-diagnostics-ts-rust-assessment.md)
- [reviews/phase-wide-diagnostics-ts-rust-assessment-review.md](phase-wide-diagnostics-ts-rust-assessment-review.md)
- [reviews/semantic-diagnostic-code-taxonomy-final-loop-review-1.md](semantic-diagnostic-code-taxonomy-final-loop-review-1.md)

User constraints applied unchanged: pre-production, no fallback compatibility, no historical compatibility, no global numeric error-code allocation, family-local `SIFR-<FAMILY>-0000..9999`, elegant compiler/language diagnostic architecture.

## Verdict

**Not yet satisfied.** Round 2's B1, B2, and M1–M6 are all integrated. The two-phase activation in `milestone_diag_2b` (registry-internal vs emission-presence checks), the explicit `DiagnosticSink::emit` surface, the explicit `LowerCtx::emit`/`emit_error` split, the `SifrDiagnostic`-level must-use/`cancel(self)` discipline, the `SIFR-INTERNAL-0002` reservation/activation for cap summaries, the carve-out for internal diagnostics under the cap, the canonical-JSON args ordering function, the deletion of the "no display path" branch, the `milestone_diag_4a` HIR/type-system mechanical-transport language, the multi-PR sub-phase declaration, the active-only constants/helpers rule, and the canonical-string-accessor mention all appear as written.

But round 2's patches re-exposed one hard contradiction and three previously-latent under-specifications. The contradiction is between the blanket "internal diagnostics use `Severity::Error`" policy and the new `SIFR-INTERNAL-0002` cap-summary diagnostic, which is required to be `Severity::Note` and internal at the same time. The under-specifications are around how `SifrDiagnostic` instances escape the must-use discipline through public struct fields, where cap-omission summary diagnostics enter the canonical stream, and how `LowerCtx::emit` dispatches across the two `DiagnosticSink` methods.

Resolve B1 before opening `milestone_diag_1`. M1–M3 are cheap to capture in the same edit pass. M4 and L1–L3 are forward-looking polish.

## Round-2 verification

For the record — these are all integrated correctly:

| Round-2 finding | Where | Status |
| --- | --- | --- |
| B1 (split `2b` activation: registry-internal non-vacuous in `2b`; emission-presence per-family) | proposal lines 745–746 | ✓ |
| B1 sub (representative fixture path in registry, fixture file in migration milestone) | line 780 | ✓ |
| B2 (specify `DiagnosticSink::emit` for non-error severities) | lines 384–388, 691 | ✓ |
| B2 sub (cap-omission summary emission path) | lines 396, 1186 | ✓ but see M2 below |
| M1 (`4a` HIR/type-system mechanical transport explicit; `7`/`8` DoDs reframed as additive) | lines 798, 895, 923 | ✓ |
| M2 (`SifrDiagnostic` itself is must-use with `cancel(self)`) | lines 306, 705 | ✓ but see M1 below |
| M3 (cap-omission summary code reserved/activated as `SIFR-INTERNAL-0002`) | lines 976, 984, 1167–1168, 1186 | ✓ |
| M4 (internal diagnostics carved out of the source-diagnostic cap) | line 1184 | ✓ but see B1 below |
| M5 (canonical-JSON args comparison) | line 596 | ✓ |
| M6 (delete dead "no display path" clause) | line 591 | ✓ |
| L1–L3 (sink-emit-time validation optional, accessor name, cancel test) | lines 668, 711 | ✓ partial — see L3 below |

Round 2's contract is met. Findings below are gaps surfaced by re-reading the proposal as a whole now that those fixes have settled.

## Blocking gaps

### B1 — Internal diagnostics severity contradiction (High)

`milestone_diag_1` scope line 686 says:

> Define the canonical top-level `Severity` enum exactly as `Error | Warning | Note`; internal diagnostics use `Severity::Error`. Help text is represented through `help` fields or `ChildSeverity::Help`, not as standalone top-level diagnostics.

But the cap-summary policy now requires:

- Line 984 (`milestone_diag_10` DoD): "Recovery-cap omission summaries are structured `Severity::Note` diagnostics with `SIFR-INTERNAL-0002`."
- Line 1167–1168 (`Internal code allocation policy`): "`SIFR-INTERNAL-0002` is reserved for structured recovery-cap omission summaries. It is activated in `milestone_diag_10`."
- Line 1186 (`Non-Error Diagnostics`): "rendering appends structured `Severity::Note` summaries using `SIFR-INTERNAL-0002`…"
- Line 396 (target architecture): "Cap-omission summary diagnostics are `Severity::Note` diagnostics and are emitted through `DiagnosticSink::emit(...)`."

`SIFR-INTERNAL-0002` is an internal code (the family is `SIFR-INTERNAL`). Its diagnostics are `InternalDiagnostic` values per the model. Line 686 says all internal diagnostics carry `Severity::Error`. Lines 396/984/1167/1186 say `SIFR-INTERNAL-0002` carries `Severity::Note`. These cannot both be true.

Two compatible resolutions exist; pick one:

1. **Soften line 686 to a per-code rule.** Replace "internal diagnostics use `Severity::Error`" with: "`SIFR-INTERNAL-*` codes carry the severity declared by their registry entry. ICE-class internal diagnostics (e.g. `SIFR-INTERNAL-0001`) declare `Severity::Error`; structured internal diagnostics such as `SIFR-INTERNAL-0002` (recovery-cap omission summaries) declare `Severity::Note`. The registry validates the declared severity at build time." This is consistent with the rest of the registry-driven design.
2. **Demote cap summaries to source notes.** Move cap-omission summaries out of `SIFR-INTERNAL-*` into a non-internal note family — but the proposal already commits `SIFR-INTERNAL-*` as the home for "structured recovery-cap omission summaries" (line 1167) and the rationale ("describe compiler-side state, not user fixable") fits the internal family. This resolution would require re-homing the code and is the worse choice.

Resolution (1) is the right call and is a one-line edit to line 686. Without it, `milestone_diag_1`'s `Severity` definition rejects the very `InternalDiagnostic` values that `milestone_diag_10` requires.

A related sub-issue: the `InternalDiagnostic` struct (lines 246–254) has a `severity: Severity` field, so the type permits any severity at the data level. The policy contradiction is in the prose, not the type — but the registry-level validation rule (the "validates the declared severity" in resolution 1) needs to be added to the build-time `#[test]` validation list at line 744 so the rule is enforced and not just documented. Add: "registry-declared severity matches the constraint for the code's family (e.g. `SIFR-INTERNAL-0001` declares `Error`, `SIFR-INTERNAL-0002` declares `Note`)".

## Medium-severity gaps

### M1 — `SourceDiagnostic`/`InternalDiagnostic` public fields bypass the must-use discipline (Medium)

The model section (lines 233–254) shows:

```rust
pub struct SourceDiagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub message_template: &'static str,
    pub args: BTreeMap<String, DiagnosticArg>,
    pub primary_span: SourceSpan,
    pub related_spans: Vec<RelatedSpan>,
    pub children: Vec<DiagnosticChild>,
    pub help: Option<String>,
    pub suggestions: Vec<DiagnosticSuggestion>,
}
```

…with all fields `pub`. The discipline at line 306 commits `SifrDiagnostic` itself to drop-bomb behavior with `cancel(self)` as the only legal non-emitting path. But with public fields on the inner structs, a contributor can write:

```rust
let diag = SifrDiagnostic::Source(SourceDiagnostic {
    code: DiagnosticCode::TYPE_ASSIGNMENT_MISMATCH,
    severity: Severity::Error,
    /* … all fields populated by hand … */
});
// Bypasses DiagnosticBuilder entirely.
// Bypasses helper-module ownership of construction.
// Drops at end of scope: drop bomb fires, but the diagnostic-helper-locality
// rule from §"Diagnostic Builder API" was already violated.
```

This is more than a discipline footgun. The proposal commits in several places to construction *only* through domain helpers and the builder:

- Line 223: "Domain helpers may use `DiagnosticCode` constants and model builders from `sifr_diagnostics`, but `sifr_diagnostics` must not become a monolithic semantic helper crate."
- Line 518: "If a helper is missing, the implementation should add the helper and assign the code deliberately."
- Line 1152: "Do not add a diagnostic helper without a registry entry in the same PR."

If `SourceDiagnostic { … }` literal construction works at any call site, the registry/helper coupling is enforceable only by review.

Pin one in `milestone_diag_1`'s scope:

- **Option A (preferred):** Make struct fields private with `pub(crate)` field access inside `sifr_diagnostics`. The builder is the only public construction path. Public read accessors (`code()`, `severity()`, `primary_span()`, etc.) cover renderer consumption. This matches rustc's `Diag` shape and makes the must-use discipline a typed contract rather than discipline.
- **Option B:** Mark `SifrDiagnostic`, `SourceDiagnostic`, and `InternalDiagnostic` as `#[non_exhaustive]` and keep fields public. This forces external constructors through the builder for forward-compatibility but allows `sifr_diagnostics`-internal construction. Weaker than option A — internal construction can still drift from the builder discipline — but lower-effort.

The proposal needs to state which. Today, the model section reads as a public-fields contract that contradicts the must-use commitment at line 306. State:

> `SourceDiagnostic` and `InternalDiagnostic` are constructed only through `DiagnosticBuilder`. The builder is the sole public construction path; struct fields are crate-private with public read accessors so renderers and serializers can inspect diagnostic content without recreating the construction surface.

If the chosen shape is option A, also add a one-line `milestone_diag_1` DoD: "no test or compiler crate constructs `SourceDiagnostic` or `InternalDiagnostic` through struct literals."

### M2 — Cap-omission summary insertion path into the canonical stream is ambiguous (Medium)

Line 396 (target architecture):

> Cap-omission summary diagnostics are `Severity::Note` diagnostics and are emitted through `DiagnosticSink::emit(...)`.

Line 1186 (Non-Error Diagnostics):

> When diagnostics are omitted because of the cap, rendering appends structured `Severity::Note` summaries using `SIFR-INTERNAL-0002`…

The cap admission step happens at sink-flush, after the canonical sort and after admission of source diagnostics. Three plausible flows are consistent with the prose, and they have different semantics:

1. **Flow A (sink re-emit + re-sort):** Cap admission step calls `DiagnosticSink::emit(summary)` for each omitted-bucket summary. The sink records a new insertion order; the canonical sort runs again; renderers consume the resulting stream. Requires re-sorting; summaries land in canonical position (after source diagnostics, before the rest of the internal-diagnostic block by code/template).
2. **Flow B (admission appends, no sink path):** Cap admission step constructs summary `SifrDiagnostic` values and appends them to the admitted stream that renderers consume. Summaries do not pass through `DiagnosticSink::emit` and therefore have no insertion order assigned. Renderers do not re-sort. Summaries land at the end of the admitted source-diagnostic block, before non-capped internal diagnostics.
3. **Flow C (rendering appends per-format):** Each renderer appends its own summaries. Three renderers may append slightly different summaries (or the same summaries but in different positions). This breaks the "one canonical stream" commitment at line 583 and the "JSON, human, and compact render from the same canonical diagnostics" DoD at line 813.

Flow C is forbidden by the proposal's own commitments. Flow A is consistent with line 396 ("emitted through `DiagnosticSink::emit(...)`"). Flow B is consistent with line 1186 ("rendering appends"). The wording today is compatible with either A or B, and contributors implementing this for the first time will pick one without realizing it is a decision.

Pick one in §"Non-Error Diagnostics":

- **Flow A (recommended):** Cap-omission summaries are emitted through `DiagnosticSink::emit(...)` *during the cap-admission step*; the driver then performs a final canonical sort over the admission set + summaries before rendering. Internal diagnostics already sort after source diagnostics, so the summaries land in their natural canonical position with no special-case ordering. This makes the summaries first-class participants in the canonical stream, consistent with line 396 and the "one stream" commitment.
- **Flow B:** Cap-omission summaries are not emitted through the sink. The cap-admission step produces a final admitted-stream value `[admitted source diags…, summaries…, internal diags…]` that all renderers consume verbatim. This avoids a re-sort but introduces a parallel construction path for `SifrDiagnostic` outside the sink, which weakens the "the sink is the only legal channel" invariant.

Flow A is more consistent with the rest of the architecture. Strike "rendering appends" at line 1186 and replace with: "the cap-admission step emits structured `Severity::Note` summaries through `DiagnosticSink::emit(...)` using `SIFR-INTERNAL-0002`. The driver re-sorts after admission so summaries land in canonical position; renderers consume the final sorted stream and do not re-sort."

A corollary: line 815 ("Ensure all renderers consume the same deterministically sorted canonical diagnostic stream before compact grouping and before recovery-cap omission summaries are computed") becomes wrong under flow A — sorting happens both before and after summary emission. Rephrase: "before recovery-cap admission, and again over the admitted set + cap summaries before rendering."

### M3 — `LowerCtx::emit` dispatch across `DiagnosticSink::emit` and `emit_error` is implicit (Medium)

Line 384 specifies:

```rust
pub fn emit(&mut self, diag: SifrDiagnostic);  // Severity must be Warning or Note.
pub fn emit_error(&mut self, diag: SifrDiagnostic) -> ErrorEmitted;
```

— `DiagnosticSink::emit` validates `Severity::Warning | Severity::Note`. Line 396 then says:

> `LowerCtx::emit(...)` is an ergonomic wrapper for warning/note diagnostics and error diagnostics that do not need an `ErrorEmitted` proof at the call site. `LowerCtx::emit_error(...)` returns `ErrorEmitted` and is required when the caller will construct a tainted value, poisoned binding, or any other cascade-suppression value.

If `LowerCtx::emit` accepts errors, it cannot route them through `DiagnosticSink::emit` (which would assertion-fail on `Severity::Error`). It must dispatch internally:

```rust
fn emit(&mut self, diag: SifrDiagnostic) {
    match diag.severity() {
        Severity::Error => { let _proof = self.sink.emit_error(diag); }
        _              => self.sink.emit(diag),
    }
}
```

This is workable and matches the helper-call examples (`ctx.emit(undefined_variable(...))`) where the call site doesn't need an `ErrorEmitted`. But the discard of the proof is invisible at the call site, which means a contributor can write `ctx.emit(error_diag)` to suppress a tainting requirement and the type system will not complain. The discipline-equivalent ("if you need a proof, call `emit_error`") is documented but not typed.

Pin the `LowerCtx::emit` contract in §"Target Architecture" or `milestone_diag_1`'s scope:

> `LowerCtx::emit` accepts any `SifrDiagnostic` and routes by severity to the corresponding `DiagnosticSink` method. For `Severity::Error`, the returned `ErrorEmitted` is intentionally discarded; call sites that need the proof must call `LowerCtx::emit_error` directly. A clippy-style `must_use` reminder on `LowerCtx::emit` is not required because the discarded proof is well-defined behavior — taint sites are the typed-proof boundary.

Without this, the first contributor implementing `LowerCtx::emit` will either re-derive the dispatch correctly (likely) or re-introduce a "proof-leaking" bug where errors silently route through `emit` and assertion-fail at runtime. Pin the rule once.

A related sub-point: line 691 says `LowerCtx` "wrappers… make proof-returning emission explicit where tainting needs it." The phrase "where tainting needs it" is the policy — explicit `LowerCtx::emit_error` is required at taint sites. Tighten to: "any HIR call site that constructs a tainted value, poisoned binding, or `ErrorEmitted`-bearing cascade-suppression value must call `LowerCtx::emit_error` directly. `LowerCtx::emit` is the right call only when the caller does not depend on the typed proof."

### M4 — `cancel(self)` is "limited to tests/internal probes" but unenforceable (Medium)

Line 421:

> `cancel(self)` is the only legal way to discard a builder without building, emitting, or returning a diagnostic, and is limited to tests/internal probes.

Line 705:

> `SifrDiagnostic::cancel(self)` exists only for tests/internal probes and is the only legal non-emitting cancellation path after `build()`.

Both are public methods. "Limited to tests/internal probes" is a discipline rule with no compile-time, test-time, or lint-time enforcement. A contributor can call `diag.cancel()` at an arbitrary call site and silently swallow a diagnostic. The proposal's emission discipline is otherwise typed-enforced (`must_use`, non-`Clone`, drop bomb) — `cancel` is the only place where the discipline relies on review.

Three options:

1. **Discipline only.** Document the rule and rely on review. Acceptable if `cancel` is rarely used and reviewable by grep.
2. **`#[cfg(any(test, feature = "internal-probes"))]` gating.** `cancel` exists only in test/probe builds. Strongest enforcement but blocks debugging in non-test code paths.
3. **Lint-style guardrail.** Add a `scripts/check_diagnostic_cancel_usage.py` (or extend `check_diagnostic_baseline_hygiene.py`) that fails if `.cancel()` appears in non-test source under `crates/` outside an explicit allowlist. Decidable, matches the proposal's other tidy-style checks, doesn't block test ergonomics.

Pick one in `milestone_diag_1`'s scope. Without a pin, "limited to tests/internal probes" is aspirational. Option 3 is the most consistent with the proposal's existing decidable-guardrail approach (see line 1118 for the registry-hygiene equivalent).

## Low-severity notes

### L1 — `SIFR-INTERNAL-0002` reservation in `milestone_diag_2a`'s skeleton is implicit

Line 721 says `milestone_diag_2a` defines "code family namespaces, the per-family local `0000..9999` convention, and initial reserved codes." The "Internal code allocation policy" at lines 1166–1168 names two reserved codes:

- `SIFR-INTERNAL-0001` (stable catch-all)
- `SIFR-INTERNAL-0002` (cap-omission summary, activated in `milestone_diag_10`)

Whether `2a`'s "initial reserved codes" includes `SIFR-INTERNAL-0002` as `Reserved` is not stated. State explicitly that `2a`'s skeleton reserves both `SIFR-INTERNAL-0001` (initial state: `Reserved` → `Active` upon first emission) and `SIFR-INTERNAL-0002` (initial state: `Reserved` → `Active` in `milestone_diag_10`). Otherwise `milestone_diag_10` has to add the registry entry alongside activation, and the registry hygiene check at line 1118 ("every emitted code is registered") fails between the moment `10`'s emission lands and the moment its registry entry lands in the same PR. Reserving in `2a` is the cleaner sequencing.

### L2 — `message` rendering source is unspecified

`SourceDiagnostic` carries both `message: String` (rendered) and `message_template: &'static str` plus `args: BTreeMap<String, DiagnosticArg>` (template + args for grouping). Where does `message` come from?

Two candidates:

1. **Rendered at `DiagnosticBuilder::build()`.** The builder has `.message_template(...)` and `.arg(...)` setters but no `.message(...)` setter. `build()` renders `message` from template + args using a single canonical renderer. Contributors do not write `message` directly.
2. **Passed by helper, asserted at `build()`.** The helper passes both `message_template` and the rendered message; `build()` asserts that re-rendering the template with the args produces the supplied message string.

Option 1 is cleaner and matches the builder API as written (no `.message(...)`). Option 2 provides defense against template/args/message drift at the cost of double-rendering. Pin option 1 in §"Target Architecture":

> `SourceDiagnostic.message` and `InternalDiagnostic.message` are rendered from `message_template` and `args` by `DiagnosticBuilder::build()` at construction time. The builder is the only renderer; helpers do not pass pre-rendered message strings.

This also implies: `args` keys must cover every placeholder in `message_template` at build-time (already required by registry validation at line 342) and the builder uses a single canonical placeholder substitution function.

### L3 — Canonical-string accessor is mentioned but still unnamed

Line 94 says "validation checks that each constant's canonical string accessor returns the registry id" but does not name the accessor. The proposal needs a name to enable the build-time test at line 744 to refer to it. Suggested: `pub fn code(&self) -> &'static str` returning `"SIFR-FAMILY-NNNN"`. Or `.canonical()`. Pick one and write it into the registry record description in `milestone_diag_2a`.

This is a one-word decision but contributors implementing `milestone_diag_2a` will bikeshed it without the name.

### L4 — `milestone_diag_4a` step (4) DoD is satisfied by deletion alone

The sub-PR sequence at lines 802–807 lists step (4) as "`CompilePhase::TypeCheck` deletion plus HIR/type-system mechanical transport migration." Step (4)'s DoD inherits the milestone DoD at line 816 ("`CompilePhase::TypeCheck` no longer assigns `SIFR-TYPE-0001` to any diagnostic path") — which is satisfied by the deletion in step (4) before the mechanical migration runs.

This means a partial step (4) PR that deletes the mapping but stops short of migrating all HIR/type-system call sites passes the milestone DoD while leaving call sites in the "fail to compile" state described at line 797. That's acceptable per the no-fallback contract, but the milestone DoD should also assert that "no HIR or type-system call site emits diagnostics through `LowerCtx::error(String)` or any pre-`SifrDiagnostic` transport after `milestone_diag_4a`". This converts the guarantee from "the bad mapping is deleted" to "every call site has migrated." Add to line 816.

### L5 — `LoweringError` "fully deleted by residual cleanup in `milestone_diag_11`" timing

Line 687 commits to deletion of `LoweringError` "by residual cleanup in `milestone_diag_11`." Line 924 (`milestone_diag_8` DoD): "`LoweringError` has no remaining internal semantic-diagnostic callers after this milestone; any leftover symbol is residual cleanup only and cannot carry user-facing diagnostic text." Line 1021 (`milestone_diag_11` guardrail): "The `LoweringError` symbol does not exist in the workspace after residual cleanup."

The deletion is correctly fenced. But `8`'s DoD says the symbol *may* remain as residual; `11`'s guardrail says it *must not*. There's a window — between `8` and `11` — where the type lives without callers. The proposal doesn't say what `9` (span completion) or `10` (recovery semantics) do about it, and a contributor implementing `9` or `10` could re-introduce a `LoweringError` callsite without violating any milestone DoD until `11`. Add a guardrail at the end of `8`'s DoD: "no new `LoweringError` callers may be introduced in `milestone_diag_9` or `milestone_diag_10`. Existing residual symbol may be deleted in any of `9`/`10`/`11`."

This is low-priority because the no-fallback hard rules already forbid re-introducing the symbol in spirit; the explicit fence is just belt-and-suspenders.

## Implementation-order assessment

The sequencing graph (lines 1037–1051) remains internally consistent at the milestone level. Round-2's identified risks (`2b` self-blocking validation, implicit `4a` HIR/type-system scope) are both resolved.

Two new sequencing observations:

1. **`SIFR-INTERNAL-0002` activation in `milestone_diag_10`.** The cap-summary code is reserved in `2a` (per L1 above) and activated in `10` simultaneously with the cap admission code. This is fine. But the cap admission machinery itself (severity-aware admission per line 1183) is part of the design pinned in `1` and refers to "before recovery-cap omission summaries are computed" in `4a`'s DoD (line 795). The implementation contract is: cap admission *infrastructure* lands in `4a` (so renderers know the canonical-stream input is the admitted set, not the raw set), and cap admission *behavior* (the 50-cap rule, summary generation, `SIFR-INTERNAL-0002` activation) lands in `10`. State this explicitly so `4a` doesn't try to implement the cap and `10` doesn't have to retrofit renderer plumbing.

   A clean phrasing for `4a`'s scope: "Renderers consume the canonical post-admission stream. Admission is a no-op pass in `4a` (no cap, no summaries). The 50-cap rule, severity-aware admission, and summary generation activate in `milestone_diag_10`."

2. **`milestone_diag_4a` step (4) and `milestone_diag_7`/`8` DoDs** (covered in M1 of round 2, now resolved). Step (4)'s mechanical transport migration to inventory codes leaves no `SIFR-TYPE-0001` callers; `7` and `8` then add domain helpers, related spans, dedupe args, and fixture coverage. The DoDs at lines 895 and 923 ("use category-specific helpers and fixtures rather than the mechanical inventory-assigned transport") are now framed correctly around what `7`/`8` add. ✓

`milestone_diag_4a` is large (renderers + inference removal + `LoweringError` retirement + parser/workspace/codegen/build/test-runner migration + `CompilePhase::TypeCheck` deletion + HIR/type-system mechanical transport). The multi-PR sub-phase declaration at lines 802–807 is the right call. Order within `4a`:

1. Renderer integration (consume `SifrDiagnostic`, no message inference, no cap).
2. `LoweringError` user-facing replacement.
3. Parser/workspace/codegen/build/test-runner transport migration.
4. `CompilePhase::TypeCheck` deletion + HIR/type-system mechanical transport.

This order is correct as written. Step (1) before (2) so renderers can consume the new model before HIR is migrated; (4) last so the public-mapping deletion is the final step gating no-fallback enforcement.

## Cross-checks against earlier review patches (still confirmed integrated)

For audit completeness, prior findings remain integrated and need no further edit:

- **F1–F10** from the source-map review (must-use builders, per-span labels, JSON snippet text, on-disk byte offsets, 1-based UTF-8 char columns, 4-variant applicability, dedup vs grouping separation, `lowered_from`, source-map record fields, multibyte tests).
- **R1–R12** from the phase-wide review (path-first ordering tuple + insertion order, decidable registry/emission/fixture rules, central path normalization + duplicate baseline detection + fixture-grammar contradiction rule + single-sorted-stream test, domain-local constructors, pending domain objects, reservation-only tooling routing, no replacement-text-in-help, no nested coded chains, `ErrorEmitted` typed proof in `milestone_diag_1`, build-time `#[test]` validation, insertion order recorded by sink, `expect-error` grammar declared).
- **Round-1 B1–B4** (path-first sort with severity-aware admission, `CompileError` deleted as public abstraction with `Result<T, ErrorEmitted>` driver, full `DiagnosticBuilder` surface, `LoweringError`/`TypeError`/`TypeErrorKind` deletion timeline).
- **Round-1 M1–M7** (schema sync script, skeleton-state validation tolerance, active-only constants/helpers, `SourceSpan` validation timing, `UPPER_SNAKE_CASE` constant naming, no-internal-JSON-provenance, `CompilePhase::TypeCheck` deletion in `4a`).
- **Round-2 B1–B2 and M1–M6** (per the verification table at the top of this review).

## Bottom line

The round-2 patches landed and the proposal is now at the boundary where one prose contradiction (B1, internal-diagnostic severity) and three behavior-pinning gaps (M1–M3) separate it from a fully implementable specification. None of these reopens the model or the sequencing graph at a structural level — they pin behavior the proposal currently leaves to interpretation:

- B1 is a hard contradiction between the blanket internal-severity rule and the cap-summary diagnostic.
- M1 — public struct fields on `SourceDiagnostic`/`InternalDiagnostic` undercut the must-use discipline.
- M2 — the cap-summary insertion path into the canonical stream has two compatible interpretations.
- M3 — `LowerCtx::emit`'s severity-dispatch contract is implicit.
- M4 and L1–L5 are forward-looking polish: `cancel` enforcement, `SIFR-INTERNAL-0002` reservation in `2a`, message rendering source, accessor name, `4a` step-(4) DoD, `LoweringError` deletion fence in `9`/`10`.

Resolve B1 and fold M1–M3 into the same edit; the proposal is then mergeable into `milestone_diag_1`. M4 and L1–L5 can land in a follow-up edit pass without blocking implementation.
