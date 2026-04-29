# Review: Phase-Wide Diagnostics Lessons from TypeScript and Rust

This review evaluates [reviews/phase-wide-diagnostics-ts-rust-assessment.md](phase-wide-diagnostics-ts-rust-assessment.md) against:

- the parent proposal in [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
- the prior source-map-focused review in [reviews/source-map-diagnostics-ts-rust-assessment-review.md](source-map-diagnostics-ts-rust-assessment-review.md)
- the local TypeScript and rustc trees at `/Users/yaseralnajjar/work/sifr/TypeScript` and `/Users/yaseralnajjar/work/sifr/rust`

The user constraints are explicit: Sifr is pre-production, no fallback or compatibility layers, and family-local `0000..9999` numbering rather than a global numeric range. Findings below respect those constraints.

## Verdict

The phase-wide assessment is **directionally correct and consistent with the no-fallback contract**. Lessons 6 (domain-local constructors), 7 (no fixture-local regex normalization), and 9 (no stashing) are particularly load-bearing and absent from the proposal as written. None of the eight lessons contradict the no-compatibility goal.

However, **three of the eight lessons are under-specified to the point of being unimplementable as written** (Lesson 1 ordering policy, Lesson 3 baseline hygiene, Lesson 5 active-code coverage), and **two important reference-implementation patterns are missing entirely** (typed error guarantees / `ErrorGuaranteed`, and the explicit chain-vs-related-span decision). The eight recommended proposal patches cover the right scope but need to be sharpened before they go into the proposal — otherwise they encode hand-wavy guardrails that won't catch the failures they describe.

Block opening `milestone_diag_1` on the strength of this assessment alone. Fold findings R1–R7 below into the proposal first; R8–R12 are forward-looking but cheap to capture now.

## Findings

Severity scale matches the prior review: **High** = will require model/schema/registry change after `milestone_diag_1` if not addressed now; **Medium** = ambiguity that will surface as fixture or renderer churn; **Low** = forward-looking nice-to-have.

### R1 — Ordering policy is under-specified and silently diverges from TypeScript (High)

The assessment's Lesson 1 cites TypeScript's `compareDiagnostics` and proposes:

> severity rank: `Error`, `Warning`, `Note`
> source diagnostics before internal diagnostics when severity ties
> source display path
> primary byte start
> primary byte end
> code
> `message_template`
> stable serialized args key/value order
> related-span count, with richer diagnostics before poorer duplicates only in the dedupe comparison if needed

TypeScript's actual key (`utilities.ts:8716-8740`) is:

```
file path → start → length → code → message head
```

then `compareRelatedInformation` (richer-first) is appended to form `compareDiagnostics`. **Severity is not in the TS primary sort.** The assessment puts severity first and cites TS as the source — this is a genuine divergence, not a clarification.

The implications matter for users:

- Severity-first ordering hides a `reveal_type(...)` `Note` on line 5 below `Error`s on lines 100-200, which is exactly opposite to how `reveal_type` is conversational and expects to render near its source location. The proposal already commits to one diagnostic stream covering errors, warnings, and notes (proposal §"Non-Error Diagnostics", line 990 onward); severity-first ordering quietly breaks that intent.
- "source diagnostics before internal diagnostics when severity ties" — internal diagnostics are typically ICEs after panic boundaries. Demoting them below user errors when severities tie can hide them; demoting them above is also wrong because they should be visible near where they fired. The right rule is probably "source first within file, internal last regardless," but the assessment doesn't pick.
- "richer diagnostics before poorer duplicates only in the dedupe comparison if needed" sneaks dedup logic into the sort key conditionally. That is exactly the kind of fallback-shaped logic the user vetoed. Sort and dedup must be separate passes with separate keys.
- "stable serialized args key/value order" — `args: BTreeMap<String, DiagnosticArg>` is already deterministic by Rust's contract. The policy needs to say *what is compared and how* (e.g. canonical-JSON serialization of the BTreeMap, byte-compared) rather than gesturing at "stable order."

**Patch direction**:

1. State the ordering tuple unambiguously, justify each level, and pick a TypeScript-style or rustc-style baseline:

   ```text
   ordering key = (
       primary display path (lexicographic),
       primary byte_start,
       primary byte_end,
       severity rank (Error < Warning < Note),
       diagnostic kind (Source < Internal),
       code (lexicographic on canonical SIFR-FAMILY-NNNN form),
       message_template (lexicographic),
       canonical-JSON-serialized args (byte-compared),
       insertion order (stable tiebreaker)
   )
   ```

   Path-first matches TypeScript and keeps `reveal_type` near its source; severity-first does not. If the team prefers severity-first, that's a deliberate departure from TS and must be documented as such, not ascribed to TS.

2. Append "insertion order" as the final tiebreaker. With future parallel HIR lowering (see also F9 of the prior review), full ordering must terminate even if every other field ties. Insertion order requires the sink to record it; this is a one-`u32`-per-diag cost.

3. Apply the sort exclusively at the driver / sink-flush boundary, **before** the recovery cap and **before** any renderer (human/compact/JSON). Add a hard rule that no renderer re-orders the canonical stream.

4. Remove the conditional "richer diagnostics before poorer duplicates only in the dedupe comparison if needed" clause. Dedup is a separate pass owned by `milestone_diag_10` per the proposal; sort is owned by the driver from `milestone_diag_4a`. They share `message_template` but not their key.

### R2 — Active-code coverage check is under-specified and risks false positives (High)

Lesson 5 asks `check_diagnostic_code_coverage.py` to validate "every active code is emitted by at least one reachable call site or fixture, otherwise it must be `Reserved`." Two problems:

- "Reachable call site" is unenforceable without static reachability analysis, which is intractable. The Rust tidy check (`rust/src/tools/tidy/src/error_codes.rs`) does not check reachability — it checks that the code appears as a string literal somewhere in the compiler source **and** is exercised by a UI test. That's a weaker, decidable property: textual presence + fixture proof.
- "Fixture coverage" is conflated with "emission path." A fixture can assert a code without there being any compiler call site that actually emits it (if the fixture is wrong) and vice versa.

**Patch direction**: replace the bullet with three independent decidable rules, each with its own check:

1. **Registry presence**: every code constant in `crates/sifr_diagnostics/src/codes.rs` has a registry entry with `state ∈ { Active, Reserved, Retired }`.
2. **Active emission**: for every `Active` code, the canonical code constant (`DiagnosticCode::FOO_BAR`) must appear at least once in non-test crate sources outside `sifr_diagnostics` itself (textual grep is sufficient and matches Rust's tidy approach). Codes only present in tests or only in `sifr_diagnostics` itself are not emitted by the compiler — demote to `Reserved` or delete the constant.
3. **Active fixture proof**: for every `Active` code, at least one e2e fixture asserts it. The harness (per Lesson 3) refuses to load fixtures asserting an unknown code, so the inverse — "every active code has a fixture" — must be its own check.

Add a fourth rule explicit to the no-fallback contract: **`Retired` codes have a docs page that remains, but no code constant in `codes.rs`** — a retired code cannot be emitted by definition. The assessment says retired-doc retention; it does not say retired-constant deletion. Without that, retired codes leak back into emission paths.

### R3 — Baseline hygiene rules don't define their own terms (High)

Lesson 3's checklist:

> normalize paths centrally, not inside individual fixtures
> fail on duplicate baseline names
> fail on duplicate or contradictory diagnostic expectations
> keep JSON/human/compact baselines based on the same ordered diagnostic stream

The first and fourth bullets are concrete and actionable. The middle two are not:

- "Duplicate baseline names" — TypeScript's check (`runner.ts`) catches duplicate baseline file names across the test suite. For Sifr the analogous risk is two fixtures named the same under `crates/sifr/tests/e2e/fail/`. The check is trivial to implement but the recommendation should name the check (e.g. extend the existing e2e discovery or add a one-liner walker in `scripts/run_e2e_pass.sh`).
- "Contradictory diagnostic expectations" is undefined. Two interpretations: (a) the same fixture asserts two different codes for overlapping spans, (b) two fixtures assert different things about the same code's wording. Both are plausible; pick one. The right one for Sifr is (a) — within a single fixture, an `expect-error` annotation on a span cannot contradict another `expect-error` on the same span.

**Patch direction**: in the proposal's `milestone_diag_5` scope, write the baseline-hygiene rules as concrete validations:

- one centralized path normalizer in the harness, applied to every renderer's output before snapshot comparison; no per-fixture regex; codify the path display policy from the source-map architecture (proposal line 558) as the single source of truth
- harness fails at startup with a `SIFR-INTERNAL-*` (not a fallback diagnostic) if two fixtures share a baseline name
- within one fixture, two `expect-error` annotations on overlapping spans are a fixture-grammar error, not a runtime mismatch
- JSON, human, and compact baselines for the same fixture are produced from one sorted-and-capped diagnostic stream; a fixture-level snapshot test asserts the three are derivable from the same `Vec<SifrDiagnostic>`

### R4 — Domain-local constructor lesson contradicts the proposal's own examples (High)

Lesson 6 is exactly right: `sifr_diagnostics` should not become a monolithic semantic helper crate. But the proposal's §"Diagnostic Builder API" (lines 434-446) writes:

```rust
ctx.emit(Diagnostic::undefined_variable(name, span));
ctx.emit(Diagnostic::type_mismatch(expected, actual, span));
ctx.emit(Diagnostic::wrong_arg_count(callable, expected, actual, span));
ctx.emit(Diagnostic::use_after_move(name, span));
ctx.emit(Diagnostic::borrow_escape_return(name, span));
ctx.emit(Diagnostic::non_exhaustive_match(subject_type, uncovered, span));
```

Read literally, those constructors are methods on a `Diagnostic` type that lives in `sifr_diagnostics` — i.e. exactly the monolithic semantic helper crate Lesson 6 forbids. The proposal needs to be rewritten so the example helpers live in their domain crates.

**Patch direction**: rewrite §"Diagnostic Builder API" to show the right home for each constructor. Concrete proposal:

- `sifr_diagnostics` exposes: `DiagnosticCode` constants, `DiagnosticBuilder`, `SourceSpan`, `RelatedSpan`, `DiagnosticSink`, severity/applicability enums, and the JSON/render plumbing.
- Domain crates own their own constructors, namespaced by domain:
  - `sifr_hir::name_resolution::diagnostics::undefined_variable(name, span) -> SifrDiagnostic`
  - `sifr_type_system::diagnostics::type_mismatch(expected, actual, span) -> SifrDiagnostic`
  - `sifr_hir::ownership::diagnostics::use_after_move(name, span) -> SifrDiagnostic`
- A constructor body builds a `DiagnosticBuilder`, sets `code = DiagnosticCode::TYPE_ASSIGNMENT_MISMATCH`, sets `message_template`, fills `args`, attaches related spans, and calls `.build()` (consuming the must-use builder).
- The hard rule "Do not construct diagnostic codes with `format!` or raw strings at emission sites" (proposal line 988) extends to "Do not define cross-domain helpers in `sifr_diagnostics`; helpers live next to the checker that emits them."

This change is mechanical but must land **before** `milestone_diag_1` writes the `Diagnostic` API surface — otherwise the API encodes the monolith.

### R5 — Stashing veto is right but needs the explicit alternative pattern (High)

Lesson 9 forbids general stashed diagnostics and offers two alternatives: "collect enough context before constructing it" or "use an explicit pending domain object that is not a `SifrDiagnostic` until finalized." Confirmed against rustc — `stash_diagnostic` / `steal_diagnostic` / `delay_span_bug` exist precisely because rustc allows partial diagnostics with later enrichment, and `delayed_bugs` co-exists with `ErrorGuaranteed` (rustc_errors/src/lib.rs:298, 423-427).

The veto is consistent with the no-fallback contract — stashing is exactly the kind of "we'll fix it up later" pattern Sifr is rejecting. But the proposal currently has no language for the alternative, and HIR lowering will hit cases where related spans are only known after a later pass (e.g. "previous move location" needs forward analysis).

**Patch direction**: add a new subsection in §"Diagnostic Emission Discipline" titled "Pending Domain Objects":

- A pending domain object is a domain-crate-owned struct (e.g. `PendingMoveError { name: Symbol, primary: SourceSpan, prior_move: Option<SourceSpan> }`) carrying the data needed to construct a `SifrDiagnostic` once enough context is known.
- Pending objects are not `SifrDiagnostic` and do not flow through `DiagnosticSink`. They are domain values until finalized.
- The finalize step constructs a `SifrDiagnostic` once and emits it once; partial finalization is forbidden.
- Pending objects do not implement `Display` or any rendering trait; they cannot accidentally surface to users.
- An ICE-style invariant ("this only fires if no other diagnostic fired") is checked by inspecting the sink at the end of the relevant pass, not by stashing — the sink already retains the canonical stream by R1's insertion-order requirement.

This gives HIR a path to express forward-collected context without reintroducing rustc's stashing surface.

### R6 — Tooling-routing recommendations are right but should be reservation-only (Medium)

Lesson 2 lists tooling fields to add to the registry (supported code-action ids, fix-all eligibility, machine-applicable suggestion presence) and a future code-action validation rule. The reservation idea is correct; the framing is over-scoped for this phase.

Sifr has no LSP, no code-action surface, and no `fix-all` consumer in this phase. Building the tooling-routing fields now without consumers risks bit-rot before they're used. But leaving the registry record shape unable to grow these fields locks in re-work later.

**Patch direction**: state explicitly that the registry record reserves *optional* fields with documented defaults, and that no validation runs on those fields in this phase:

- `tool_actions: Vec<ToolActionId>` — empty by default; populated by future LSP integration.
- `fix_all_eligible: bool` — false by default.
- `has_machine_applicable_suggestion: bool` — derived from suggestion `applicability` at registry-load time, not authored manually.

The future code-action validation rule ("server validates that the requested code is present in the active diagnostics for the requested span") belongs in a follow-up phase explicitly, not in this phase's hard rules. Adding it to this phase's hard rules section is over-scoping.

Note: `has_machine_applicable_suggestion` should be derived, not authored, to avoid a second source of truth that drifts.

### R7 — Suggestion JSON shape (Lesson 8) is fine but the test it asks for is missing (Medium)

Lesson 8 says "make sure renderer tests prove suggestions are not duplicated as both children and top-level suggestions in JSON." This is the right test to demand and is missing from `milestone_diag_11`'s guardrail list. The proposal has `children: Vec<DiagnosticChild>` (Note/Help only) and `suggestions: Vec<DiagnosticSuggestion>` — the model already separates them, but no test enforces the separation. A future helper that accidentally appends a "fix: replace `x` with `y`" as a `Help` child rather than a `DiagnosticSuggestion` will produce duplicate-feeling JSON.

**Patch direction**: add a guardrail in `milestone_diag_11`:

> JSON output: a `Help` child message must not contain a literal source replacement; replacements live exclusively in `DiagnosticSuggestion::edits`. A unit test constructs a diagnostic with both shapes and asserts the JSON has the suggestion in `suggestions` and a non-replacement `Help` line in children, never duplicated.

This is one fixture's worth of work and locks in the JSON shape.

### R8 — Missing: chain vs related-span decision is undocumented (Medium)

TypeScript has `DiagnosticMessageChain` (`compiler/types.ts:7244-7295`): a tree of nested messages each with their own `code`, used heavily for type-system diagnostics like "Type 'X' is not assignable to type 'Y'. Type 'A' is not assignable to type 'B'." rustc has no equivalent — it uses related spans, sub-diagnostics, and labels.

The Sifr proposal models `children: Vec<DiagnosticChild>` with severity restricted to `Note | Help` and **no code or span on a child**. That implicitly rejects chains. The decision is correct (rustc-style related spans are simpler, dedup is cleaner with one code per top-level diagnostic, and Sifr's compact grouping by `(severity, code, message_template, primary file)` only works if each top-level diagnostic owns one code), but it should be written down.

The phase-wide assessment doesn't discuss this at all. Without an explicit decision, a future contributor will read TS, see chains, and propose adding them — which would re-open dedup/grouping/JSON-shape questions.

**Patch direction**: add a one-paragraph subsection in §"Target Architecture":

> Sifr does not model nested coded child diagnostics (TypeScript's `DiagnosticMessageChain`). Each top-level `SifrDiagnostic` owns exactly one `DiagnosticCode`. Layered explanations are expressed via `RelatedSpan` (with `RelatedKind`) and `DiagnosticChild` (`Note`/`Help`, no code). This keeps compact grouping, recovery dedup, and JSON shape one-code-per-diagnostic, mirroring rustc.

### R9 — Missing: typed error-guarantee mechanism for tainted values (Medium)

rustc's `ErrorGuaranteed` (rustc_errors/src/lib.rs:62, 423-427, 663) is a zero-sized type returned by `emit()` that proves an error has been emitted. Tainted HIR values carry `ErrorGuaranteed`, so downstream code that inspects them can either surface the guarantee (instead of producing a follow-on diagnostic) or panic with the guarantee in hand if it tries to do real work on a tainted value.

`milestone_diag_10` says "define which diagnostics produce a typed error expression or poisoned binding to prevent cascades" but doesn't say *how* the typed error expression certifies that a diagnostic was actually emitted. Without `ErrorGuaranteed`-style typing, the cascade-suppression path is policy rather than enforcement — a contributor can construct a tainted value without having emitted anything, and the no-fallback contract has no language defending against that.

**Patch direction**: add a primitive in `sifr_diagnostics`:

```rust
#[derive(Copy, Clone, Debug)]
pub struct ErrorEmitted(());

impl DiagnosticSink {
    pub fn emit_error(&mut self, diag: SifrDiagnostic) -> ErrorEmitted { ... }
}
```

Tainted HIR types carry `ErrorEmitted`. `ErrorEmitted` cannot be constructed outside `DiagnosticSink::emit_error`. This is the rustc pattern at minimal scope and integrates cleanly with the must-use/non-clone builder. Add it to `milestone_diag_1`, not `milestone_diag_10` — the type belongs to the model, not to recovery semantics.

This is consistent with no-fallback: the alternative (untyped tainting) is exactly the kind of "this should be fine because we emitted somewhere" pattern that rots into a fallback path.

### R10 — Missing: registry-validation timing (Medium)

The proposal validates the message_template/args correspondence at "registry loading" (line 332). The assessment doesn't address when registry loading runs. Two options:

- **Compiler startup**: the registry is loaded once per `cargo run -p sifr` invocation, validated, then used. Validation cost is paid every run, but errors are caught even on a single invocation.
- **Build-time check via test**: validation is in a `#[test]` so `cargo test` catches drift. Runtime cost is zero. Errors are caught only by the test suite, not by users.

Rust uses build-time `#[derive(Diagnostic)]` plus tidy. TypeScript uses a build step that validates `diagnosticMessages.json`. Sifr has no equivalent today.

**Patch direction**: pick build-time. Add a validation `#[test]` in `sifr_diagnostics` that loads the registry and asserts:

- every placeholder in every `message_template` has a matching declared arg name,
- every declared arg is referenced in `message_template` or marked JSON-only,
- every active code has a docs page,
- every `DiagnosticCode` constant has a registry entry,
- code constant names match the canonical `SIFR-FAMILY-NNNN` form.

The runtime cost stays zero for users. Drift surfaces in `scripts/run_all_tests.sh --profile quick`. This belongs in `milestone_diag_2a` because the registry skeleton is the natural home for the validation harness.

### R11 — Missing: insertion-order recording in `DiagnosticSink` (Low)

R1 calls for insertion order as the final tiebreaker. The proposal does not specify that `DiagnosticSink` records insertion order. With future parallel HIR lowering this matters — without insertion order, two diagnostics that tie on every other key sort non-deterministically across runs.

**Patch direction**: in `milestone_diag_1`, state that `DiagnosticSink` assigns a monotonic `u32` insertion sequence to every accepted diagnostic. The sequence is not part of the JSON output (it's an internal sort tiebreaker), but it must survive into the sort step. Cost: 4 bytes per diagnostic.

### R12 — Missing: review of `expect-error` annotation grammar against new codes (Low)

The proposal mentions `expect-error` annotations (line 1014: "Do not allow an `expect-error` fixture annotation to use a code absent from the registry") but neither the proposal nor the assessment specifies the annotation grammar. Today's grammar may accept message-substring matchers (`expect-error: SIFR-TYPE-0001 use of undefined`) — those substrings can stop matching when message wording legitimately changes for non-semantic reasons.

**Patch direction**: in `milestone_diag_5`, declare the `expect-error` grammar precisely:

- one canonical form: `expect-error: SIFR-FAMILY-NNNN`
- optional `at-span:` qualifier when a fixture has multiple errors at distinguishable positions
- no message-substring matcher (the canonical form is the code; message wording is owned by the registry)
- the harness rejects unknown forms with a `SIFR-INTERNAL-*` startup error and a closest-match hint, matching the proposal's "fail loudly" rule

This consolidates Lesson 3's harness-validation rule with the actual annotation surface and prevents accidental message-coupled tests from re-emerging.

## Items the assessment got right and shouldn't change

- Domain-local constructors over a monolithic helper crate (Lesson 6, modulo R4's correction of the proposal's own examples).
- No fixture-local regex normalization (Lesson 7) — directly consistent with no-fallback.
- No general stashed diagnostics (Lesson 9) — directly consistent with no-fallback, just needs R5's explicit alternative.
- Suggestions remain first-class data, not nested in children (Lesson 8) — keep the proposal's existing direction.
- The `SourceId` non-stable-identity guidance (Lesson 4) duplicates F9 of the prior review but is restated correctly; no harm.
- The verdict that the eight patches are not new code families and not fallback behavior — accurate.

## Items that would be over-engineered if added now

- A full code-action / `fix-all` validation pipeline in this phase (R6 demotes Lesson 2 to reservation-only).
- A separate baseline-update / `--bless` CLI mode beyond what the existing `insta` workflow provides — Lesson 3 already says "explicit and never part of normal validation," and `insta`'s existing review flow covers it.
- A `delay_span_bug` equivalent for invariant checks — R5's pending-domain-object pattern + the sink-inspection alternative already cover this without rustc's complexity.
- Macro hygiene / `SyntaxContext`-style spans — restated from prior review F-list; still over-scoped.

## Patch summary for the proposal

Concrete edits before `milestone_diag_1` opens. R# numbers reference findings above.

1. Add §"Diagnostic Ordering Policy" between §"Grouping and Deduplication Keys" and §"Source Mapping Architecture" with the explicit ordering tuple (R1). State that sort runs at the driver/sink-flush boundary, before the recovery cap, before any renderer.
2. Append "insertion order" as the final tiebreaker and require `DiagnosticSink` to record it (R1, R11).
3. Rewrite §"Diagnostic Builder API" so example helpers live in domain crates (`sifr_hir::name_resolution::diagnostics::undefined_variable`, etc.), and `sifr_diagnostics` exposes only `DiagnosticBuilder`, `DiagnosticCode` constants, and primitives (R4). Drop `Diagnostic::undefined_variable`-style examples.
4. Add §"Pending Domain Objects" inside §"Diagnostic Emission Discipline": pending objects are domain-crate values, never `SifrDiagnostic`, never rendered, finalized once into a single emitted diagnostic (R5).
5. Add an `ErrorEmitted` zero-sized type to `milestone_diag_1` and require taint values that cause cascade-suppression to carry it (R9).
6. In `milestone_diag_2a`, add a build-time validation `#[test]` covering placeholder/arg correspondence, docs presence, and constant/registry/code-form sync (R10).
7. Rewrite `milestone_diag_5` baseline-hygiene rules into concrete validations: central path normalizer, duplicate-baseline check, fixture-grammar contradiction rule, and one-source-of-canonical-stream-feeds-three-renderers fixture-level test (R3). Declare the `expect-error` grammar (R12).
8. Replace Lesson 5's "reachable call site" wording with three decidable rules (registry presence, textual emission of code constant outside `sifr_diagnostics`, fixture proof) plus retired-constant-deletion (R2). Wire into `scripts/check_diagnostic_code_coverage.py` and `scripts/run_all_tests.sh`.
9. Demote Lesson 2's tooling-routing rules to optional registry record reservations with no validation; move the future server-side validation rule out of this phase's hard rules (R6).
10. Add a `milestone_diag_11` guardrail: `Help` children must not contain literal replacement text; replacements live in `DiagnosticSuggestion::edits` (R7).
11. Add a one-paragraph subsection in §"Target Architecture" stating Sifr deliberately does not model TypeScript's `DiagnosticMessageChain`; layered explanations use `RelatedSpan` + `DiagnosticChild` only (R8).

## Bottom line

Approve the assessment's directional conclusions. None of its eight lessons contradicts the user's no-fallback or family-local-numbering constraints, and three of them (6, 7, 9) are load-bearing for keeping the canonical model honest. But its three highest-impact lessons (1 ordering, 3 baseline hygiene, 5 active-code coverage) are stated at a level that won't survive contact with implementation; sharpen them as in R1–R3 before they go into the proposal. Add R4 (rewrite the proposal's own helper examples), R5 (pending-domain-object alternative to stashing), and R9 (typed `ErrorEmitted`) so the no-fallback contract is enforced by types, not policy. R8 and R10–R12 are cheap to capture now and prevent re-work later.
