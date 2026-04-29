# Final-Loop Review #4: Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics

This review evaluates [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) after the round-3 patch pass against [reviews/semantic-diagnostic-code-taxonomy-final-loop-review-3.md](semantic-diagnostic-code-taxonomy-final-loop-review-3.md). It re-checks that the round-3 findings are integrated and looks for any contradictions, under-specifications, over-scoping, or implementation-order risks introduced by those edits or surfaced by re-reading the proposal as a whole.

User constraints applied unchanged: pre-production, no fallback compatibility, no historical compatibility, no global numeric error-code allocation, family-local `SIFR-<FAMILY>-0000..9999`, elegant compiler/language diagnostic architecture.

## Verdict

**SATISFIED: no blocking gaps remain.**

Round 3's B1, M1–M4, and L1–L5 are all integrated cleanly. The internal-severity contradiction is resolved by registry-declared severities, the model-struct construction is locked to `DiagnosticBuilder` via crate-private fields with public read accessors, the cap-summary insertion path through `DiagnosticSink::emit` plus a final canonical re-sort is pinned, the `LowerCtx::emit`/`emit_error` severity-dispatch contract is explicit, and the `cancel`-usage decidable guardrail (`scripts/check_diagnostic_cancel_usage.py`) is wired into both `milestone_diag_1`'s DoD and `scripts/run_all_tests.sh`. The proposal is mergeable into `milestone_diag_1`.

A small set of polish-level observations are recorded below for completeness. None blocks implementation.

## Round-3 verification

For the record — every round-3 item is integrated correctly:

| Round-3 finding | Where | Status |
| --- | --- | --- |
| B1 (replace blanket internal-Error rule with registry-declared severity per code) | line 684 | ✓ |
| B1 sub (registry-declared severity in build-time validation list) | line 745 | ✓ |
| M1 (struct fields crate-private; builder is sole construction path; public read accessors) | lines 233–254, 306 | ✓ |
| M1 sub (example switched to `DiagnosticBuilder::source(...)` with `.build()`) | lines 326–336 | ✓ |
| M2 (cap summaries via `DiagnosticSink::emit` during admission + final canonical re-sort) | lines 394, 1192 | ✓ |
| M2 sub (`milestone_diag_4a` rephrased; admission as no-op pass) | lines 796, 815 | ✓ |
| M3 (`LowerCtx::emit` severity-dispatch contract pinned; `emit_error` required at taint sites) | line 394 | ✓ |
| M3 sub (`LowerCtx` wrapper language tightened in `milestone_diag_1`) | line 689 | ✓ |
| M4 (`cancel`-usage decidable guardrail in DoD + run_all_tests + hard rules) | lines 711, 1023, 1104, 1119, 1122 | ✓ |
| L1 (`SIFR-INTERNAL-0001`/`-0002` reservation made explicit in `milestone_diag_2a`) | line 739 | ✓ |
| L2 (`message` rendered by `DiagnosticBuilder::build()` from template + args; helpers don't pre-render) | line 320 | ✓ |
| L3 (canonical accessor named `code() -> &'static str`) | lines 94, 742 | ✓ |
| L4 (`milestone_diag_4a` DoD strengthened: no HIR/type-system pre-`SifrDiagnostic` transport remains) | line 818 | ✓ |
| L5 (`LoweringError` reintroduction fenced in `9`/`10`) | lines 926–927 | ✓ |

Round-2 and round-1 fixes remain integrated; the Round-3 review's "still-confirmed" cross-checks are unchanged.

## Cross-cutting consistency check

I traced the cap-summary flow end-to-end against the canonical-stream commitment and ordering policy. The flow now reads:

1. Diagnostics enter `DiagnosticSink` during lowering/checking; each receives a monotonic insertion sequence (`milestone_diag_1` DoD line 690).
2. Driver sorts the sink's contents by the Diagnostic Ordering Policy tuple (lines 583–595).
3. Admission pass (no-op in `4a`, severity-aware 50-cap in `10`) selects admitted source diagnostics in canonical order (line 1189).
4. If diagnostics were omitted, the cap-admission step emits `SIFR-INTERNAL-0002` `Severity::Note` summaries through `DiagnosticSink::emit(...)` with per-bucket counts (line 1192). These get fresh insertion sequences from the sink.
5. Driver performs a *final* canonical sort over admitted source diagnostics, internal diagnostics (which now include the cap summaries by construction), and renders.
6. Renderers consume the final sorted stream and never re-sort (lines 597, 1192).

No contradiction. The "admitted source diagnostics, internal diagnostics, and cap summaries" enumeration in line 1192 is technically redundant (cap summaries are internal diagnostics), but redundancy here is harmless and makes the data flow explicit for first-time readers.

The `LowerCtx::emit` severity-dispatch and `emit_error` taint-proof contract is internally consistent: `ErrorEmitted` is the only thing that can construct a tainted value or poisoned binding, and only `DiagnosticSink::emit_error` returns it, so `LowerCtx::emit_error` is the typed-required path at every taint site. `LowerCtx::emit(error_diag)` works for non-tainting error emission (matches type checker's "emit and continue" pattern) and the discarded proof is a deliberate, documented behavior, not an accident. This is the right shape.

The `SourceDiagnostic`/`InternalDiagnostic` `pub(crate)` fields plus public read accessors close the construction-bypass hole without changing the public consumption surface for renderers and tests. `SifrDiagnostic` itself remains a `pub enum` with `pub` variants — that's correct, since the variants are uninhabitable from outside the crate (their inner structs have crate-private fields), so wildcard pattern matching still works for renderers but struct-literal construction does not leak.

## Polish-level observations (non-blocking)

These are minor and do not block opening `milestone_diag_1`. List them here so a follow-up edit pass can absorb them without churn.

### P1 — Builder-supplied severity is not validated against registry-declared severity

`DiagnosticBuilder::source(code, severity, primary_span)` and `DiagnosticBuilder::internal(code, severity)` (lines 405–411) take `severity` as a parameter. The registry declares one severity per code (line 745: "registry-declared severity constraints"). There is no specified runtime check that the supplied severity matches the registry-declared severity — a helper that passes the wrong severity would emit a diagnostic with a severity inconsistent with the registry, and the build-time validation (which inspects registry data, not emission sites) would not catch this. `DiagnosticSink::emit`/`emit_error` validates severity matches the *method*, not the *registry*, so calling `LowerCtx::emit_error(warning_diag)` would still trip the sink's severity check, but `LowerCtx::emit(error_with_wrong_severity_in_registry)` would not.

This is caught downstream by fixture baselines (each fixture locks code+severity), so it is not a correctness blocker. But two cleaner alternatives exist if a follow-up wants tighter typing:

- **Drop severity from the builder constructors entirely** and derive it from the code via `DiagnosticCode::declared_severity()`. This makes the registry the single source of truth for severity at the type level. Fits the "registry as source of truth" theme already established for code, template, args, and docs.
- **Validate at `build()`** that `supplied_severity == code.declared_severity()`, debug-panic on mismatch.

If the proposal intends severity-is-fixed-per-code (which the build-time registry validation implies), the first option is the more elegant move. If the design wants to leave room for variable severity (e.g., a future strict-mode promoting some warnings to errors), the current shape is fine and this observation is moot — but in that case, "registry-declared severity constraints" should be qualified as "registry-declared *default* severity" so a future reader does not interpret it as immutable.

Either way, this is a small clarification, not a blocker.

### P2 — `DiagnosticSink::emit_error` proof discarded by `LowerCtx::emit` is invisible at the call site

Line 394 says `LowerCtx::emit` "intentionally discards the proof" for `Severity::Error`. The reasoning (callers that don't need the proof use `emit`; callers that do use `emit_error`) is correct, but the proof discard is not visible in source. A contributor reading `ctx.emit(error_diag)` cannot tell whether the omission was deliberate (no taint needed) or a bug (taint forgotten).

The type system enforces correctness for taint-bearing callers — they cannot construct a tainted value without `ErrorEmitted`, so they will reach for `emit_error`. The risk is the inverse: a reviewer rubber-stamping an `emit(error_diag)` call that should have been `emit_error` because the call site doesn't yet construct the tainted value but a follow-on commit adds one.

Two options if the proposal wants to harden:

- Add a clippy-style local lint or `scripts/check_lowerctx_emit_error_usage.py` that flags `ctx.emit(...)` calls where the immediately following statements introduce a taint construct (heuristic; brittle).
- Document the convention in `internal_docs/architecture.md` once `milestone_diag_4a` lands so the discipline is searchable.

Lower priority than P1; the type system already does most of the work.

### P3 — Schema generator binary name is implicit

The docs generator is named explicitly: `cargo run -p sifr_diagnostics --bin gen-error-docs` (line 724). The schema sync script `scripts/check_diagnostic_schema_sync.py` (line 1103) implies a generator on the Rust side, but the binary is unnamed. Convention would suggest `cargo run -p sifr_diagnostics --bin gen-schema` or similar. This is a one-line addition to `milestone_diag_1`'s scope to avoid bikeshedding when a contributor implements it.

### P4 — Cap-summary message-template shape is unspecified

Line 1192 gives example rendered text:
> `3 additional errors omitted by recovery cap` and `10 additional reveal_type results omitted by recovery cap`.

But the underlying `message_template` and `args` shape is not pinned. Two reasonable shapes:

- One template per severity bucket: `{count} additional {severity_plural} omitted by recovery cap` with `args: { count, severity_plural }`. Clean grouping.
- Specialized template for `reveal_type` overflow: `{count} additional reveal_type results omitted by recovery cap` with `args: { count }`. Treats `reveal_type` as a distinct bucket.

Per line 318 (`message_template` must not contain dynamic identifiers), the template needs to use placeholder substitution rather than baking severity strings into the template. Either shape works; pinning one in `milestone_diag_10`'s scope avoids two contributors reaching for different shapes. Not blocking — this is implementation detail that the migration milestone naturally pins when the summary fixture lands.

### P5 — `DiagnosticChild`, `RelatedSpan`, `DiagnosticSuggestion`, `SuggestionEdit` retain `pub` fields

The model-section structs at lines 256–283 have `pub` fields, while `SourceDiagnostic` and `InternalDiagnostic` were tightened to `pub(crate)`. A contributor outside `sifr_diagnostics` could construct a `DiagnosticSuggestion { ... }` directly and pass it to `DiagnosticBuilder::suggestion(...)` (line 417), bypassing any future builder-side validation of suggestion edits.

This is consistent with the design choice to make the *diagnostic top-level* construction the protected boundary while keeping inner value types as plain data. Renderers and serializers benefit from public field access. The proposal does not commit to validating suggestion-edit shapes (e.g., overlapping edits) at builder time, so there is currently no validation surface to bypass.

This is a deliberate design choice, not a gap. Recording it because round-3's M1 was framed as "make struct fields private" generically — the proposal correctly applied that to the top-level diagnostic structs only, not to inner value types.

## Implementation-order assessment

The sequencing graph (lines 1042–1054) is internally consistent and unchanged from round 3:

```
1 → 2a → 3 → 2b → 4a → 6 → 5 → 7 → 8 → 4b → 9 → 10 → 11
```

Two fences remain correctly placed:

- `2b`'s active-code validation becomes non-vacuous only when `2b` populates active entries (line 746). `4a`+ family migrations populate emission-presence per-family. Global enforcement is in `11`. This sequencing passes a registry validation walk.
- `LoweringError`'s deletion is fenced through `9`/`10` (line 926–927) and asserted absent in `11` (line 1025). No window allows a contributor to silently re-introduce it after `8`.

`milestone_diag_4a` remains the largest milestone and the multi-PR sub-phase declaration at lines 803–808 is the right call. Order within `4a` is correct: renderer integration first (so the new model can be consumed), then `LoweringError` user-facing replacement, then transport migration of remaining surfaces, then `CompilePhase::TypeCheck` deletion + HIR/type-system mechanical transport migration last so the no-fallback gate is the final step.

`milestone_diag_10` activates `SIFR-INTERNAL-0002` simultaneously with cap-summary emission. The reservation in `2a` (line 739) means `10`'s registry hygiene check stays valid: the code is registered before activation, and `2b`'s "every emitted code exists in the registry" rule is met when `10`'s emission lands.

No new sequencing risks were introduced by round 3's edits.

## Bottom line

The round-3 patches landed cleanly. Every blocking gap from round 3 is resolved at the level the proposal commits to enforce. The proposal is at the boundary where it can be opened as `milestone_diag_1` and implemented to its specification without re-litigating model, sequencing, or discipline contracts.

The five polish observations (P1–P5) are forward-looking and can be folded into a follow-up edit pass without blocking implementation: P1 and P3 are one-line clarifications, P2 is a discipline note for `internal_docs/architecture.md`, P4 is naturally pinned when `milestone_diag_10`'s summary fixture lands, and P5 is recording a deliberate design choice rather than asking for a change.

**SATISFIED: no blocking gaps remain.**
