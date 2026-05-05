# Phase-Wide Diagnostics Lessons from TypeScript and Rust

This assessment expands the prior source-map review to the rest of the ad-hoc diagnostic phase:

- `/Users/yaseralnajjar/work/sifr/TypeScript`
- `/Users/yaseralnajjar/work/sifr/rust`
- `/Users/yaseralnajjar/work/sifr/codebase/issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`

The current proposal is already aligned on the largest architectural points: a dedicated `sifr_diagnostics` crate, a canonical source-map-backed model, family-local `SIFR-<FAMILY>-NNNN` identity, no fallback compatibility, structured suggestions, and registry/docs generation.

The remaining learnings are mostly phase-wide guardrails around ordering, test contracts, code-action/tooling identity, and where diagnostic constructors should live.

## TypeScript Lessons

### 1. Diagnostics Need a Total Ordering, Not Only Grouping

TypeScript's `createDiagnosticCollection()` stores diagnostics in sorted arrays and deduplicates with `compareDiagnosticsSkipRelatedInformation`.

Relevant local references:

- `/Users/yaseralnajjar/work/sifr/TypeScript/src/compiler/utilities.ts`
- `createDiagnosticCollection`
- `compareDiagnostics`
- `compareDiagnosticsSkipRelatedInformation`

TypeScript orders by file path, start, length, canonical code, and canonical message head, then considers related information. This is separate from whether diagnostics are grouped for display.

Current Sifr proposal defines compact grouping and recovery deduplication, but it does not yet define a global deterministic diagnostic ordering policy for:

- human rendering
- compact rendering
- JSON rendering
- recovery cap application
- fixture baselines

Recommended phase gap fix:

- Add a `Diagnostic Ordering Policy`.
- Sort the canonical stream at the driver/sink-flush boundary before rendering and before applying the top-level recovery cap.
- Suggested order:
  - source display path
  - primary byte start
  - primary byte end
  - severity rank: `Error`, `Warning`, `Note`
  - diagnostic kind: source before internal when earlier fields tie
  - code
  - `message_template`
  - stable serialized args key/value order
  - sink insertion order

This keeps output deterministic across hash-map iteration, module traversal order, and recovery paths.

### 2. Diagnostic Codes Are Tooling Routing Keys

TypeScript registers code fixes against diagnostic codes in `services/codeFixProvider.ts`:

- `errorCodeToFixes`
- `registerCodeFix`
- `getSupportedErrorCodes`
- `getFixes`
- `codeFixAll`

The server also validates client requests by checking that the requested error code is actually present in the requested source range before returning code fixes:

- `/Users/yaseralnajjar/work/sifr/TypeScript/src/server/session.ts`
- `BADCLIENT: Bad error code ... not found in range ...`

Current Sifr proposal models suggestions, but it does not explicitly state the future tooling contract around code-action routing.

Recommended phase gap fix:

- Registry records should reserve optional tooling metadata, with no validation or LSP/code-action implementation in this phase:
  - supported code-action ids
  - whether a diagnostic can participate in fix-all
  - whether machine-applicable suggestions are derivable from emitted suggestions
- Future LSP/code-action entry points should validate that the requested code exists in the active diagnostics for the requested span. That belongs to a future tooling phase, not this phase's hard rules.

This is important because once codes become precise, they will be used by editors and tools as stable routing keys, not just rendered labels.

### 3. Test Harness Baselines Need Centralized Normalization

TypeScript uses baseline generation with path sanitization and duplicate baseline detection:

- `/Users/yaseralnajjar/work/sifr/TypeScript/src/testRunner/compilerRunner.ts`
- `verifyDiagnostics`
- `verifySourceMapRecord`
- `Utils.removeTestPathPrefixes`
- `/Users/yaseralnajjar/work/sifr/TypeScript/src/testRunner/runner.ts`
- duplicate baseline name detection

Current Sifr proposal requires fixture grammar cleanup and registry validation, but it should also specify baseline hygiene:

- normalize paths centrally, not inside individual fixtures
- fail on duplicate baseline names
- fail on duplicate or contradictory diagnostic expectations
- keep JSON/human/compact baselines based on the same ordered diagnostic stream

This matters because source-map and path-remapping improvements otherwise become flaky across machines.

### 4. Incremental Diagnostics Can Reuse Structured Diagnostics Later

TypeScript's builder caches semantic diagnostics per file and converts reusable diagnostics back into current program diagnostics:

- `/Users/yaseralnajjar/work/sifr/TypeScript/src/compiler/builder.ts`
- `semanticDiagnosticsPerFile`
- `ReusableDiagnostic`
- `convertToDiagnosticRelatedInformation`

Sifr does not need incremental compilation in this phase, but the diagnostic model should avoid storing renderer-only state as identity. The current proposal already does this with `SourceId`, `SourceSpan`, `message_template`, and scalar args.

Recommended phase note:

- Keep `SourceId` session-local and do not serialize it as stable identity.
- Any future incremental cache must rehydrate diagnostics through source-map/project identity, not reuse stale session-local ids.

This is likely a documentation note rather than a new milestone.

## Rust Lessons

### 5. Every Active Code Needs Docs, Tests, and Actual Emission

Rust's tidy check verifies a full code lifecycle:

- `/Users/yaseralnajjar/work/sifr/rust/src/tools/tidy/src/error_codes.rs`
- code listed in registry
- long-form explanation exists
- doctest or code example exists
- UI test exists
- code is actually emitted by the compiler
- removed explanations are blocked

Current Sifr proposal has registry/docs sync and fixture coverage. The missing gap is the inverse check: active codes should not exist forever without an emission path unless they are explicitly `Reserved`.

Recommended phase gap fix:

- `check_diagnostic_code_coverage.py` should validate:
  - every emitted code is registered
  - every active code has representative fixture coverage
  - every active code has generated docs
  - every active code's canonical `DiagnosticCode::...` constant appears in non-test compiler source outside `sifr_diagnostics`, otherwise it must be `Reserved` or deleted
  - every active code has representative fixture proof
  - retired code docs remain present and are never deleted

This preserves a clean registry while still allowing intentional future reservations.

### 6. Diagnostic Constructors Should Be Domain-Local, Not a Monolith

Rust keeps many typed diagnostics close to the compiler domain that emits them, for example:

- `/Users/yaseralnajjar/work/sifr/rust/compiler/rustc_parse/src/errors.rs`
- `#[derive(Diagnostic)]`
- `#[diag(..., code = E0178)]`
- `#[primary_span]`
- `#[subdiagnostic]`
- `#[multipart_suggestion]`

Current Sifr proposal says HIR should emit through typed helpers, but it risks centralizing all helpers in `sifr_diagnostics`.

Recommended phase gap fix:

- `sifr_diagnostics` owns the canonical model, registry, code constants, source map, schema, and renderers.
- Domain crates own domain-specific constructors/helpers near the checking logic.
- A helper may reference `DiagnosticCode` constants from `sifr_diagnostics`, but broad constructor modules like `sifr_diagnostics::Diagnostic::type_mismatch(...)` should not become a monolithic semantic layer.
- Shared helper modules are acceptable only where they remove real duplication within one domain.

This aligns with the repository's no-monolithic-files rule and keeps diagnostic wording close to semantic ownership.

### 7. Arbitrary Per-Fixture Normalization Is an Escape Hatch

Rust compiletest supports fixture-local normalization directives:

- `/Users/yaseralnajjar/work/sifr/rust/src/tools/compiletest/src/directives.rs`
- `normalize-stdout`
- `normalize-stderr`
- regex replacement parsing

This is powerful, but for Sifr's current diagnostic cleanup it would undermine the goal. The phase should explicitly prefer central normalization and reject broad fixture-local regex normalization for diagnostics.

Recommended phase gap fix:

- Allow only central path/remapping normalization for diagnostic baselines.
- Forbid fixture-local regex normalization in diagnostic baselines unless a future issue explicitly adds a reviewed exception mechanism.
- Keep baseline update/bless flows explicit and never part of normal validation.

### 8. JSON Shape Should Avoid Rustc's Nested Suggestion-Ambiguity

Rust JSON serializes suggestions as `help` children with spans that may include replacement text:

- `/Users/yaseralnajjar/work/sifr/rust/compiler/rustc_errors/src/json.rs`
- `Diagnostic.children`
- `DiagnosticSpan.suggested_replacement`
- `suggestion_applicability`

The current Sifr proposal improves on this by modeling `DiagnosticSuggestion { message, applicability, edits }` as first-class data rather than hiding suggestions inside child diagnostics. Keep that direction.

No change recommended except making sure renderer tests prove suggestions are not duplicated as both children and top-level suggestions in JSON.

### 9. Emission Discipline Should Include Stashing/Enrichment Rules

Rust can stash diagnostics and later steal/emit them with additional context:

- `/Users/yaseralnajjar/work/sifr/rust/compiler/rustc_errors/src/lib.rs`
- `stashed_diagnostics`
- `stash_diagnostic`
- `steal_diagnostic`
- `emit_stashed_diagnostics`

Current Sifr proposal correctly borrows `#[must_use]`/non-clone/drop discipline. It should also state whether Sifr allows delayed enrichment.

Recommended phase gap fix:

- Do not add general stashed diagnostics in this phase.
- If a diagnostic needs related spans from a later pass, either:
  - collect enough context before constructing it, or
  - use an explicit pending domain object that is not a `SifrDiagnostic` until finalized.
- Constructed `SifrDiagnostic` values are immutable evidence ready for emission/return.

This avoids a partially emitted/forgotten-diagnostic class of bugs.

## Recommended Proposal Patches

1. Add `Diagnostic Ordering Policy` after `Grouping and Deduplication Keys`.
2. Update `Diagnostic Builder API` to state domain crates own domain-local constructors; `sifr_diagnostics` must not become a monolithic semantic helper crate.
3. Update `Diagnostic Emission Discipline` to forbid general stashed diagnostics in this phase and require pending domain objects before final diagnostic construction.
4. Update registry record shape in `milestone_diag_2a` to include optional tooling metadata and long-form docs/fixture fields.
5. Update `milestone_diag_4a` to require all renderers consume the same sorted canonical stream before cap/grouping.
6. Update `milestone_diag_5` to require central baseline normalization, duplicate baseline/expectation detection, and no fixture-local diagnostic regex normalization.
7. Update `milestone_diag_11` and validation plan to enforce active-code emission coverage and retired-doc retention.
8. Reserve optional registry fields for future code-action routing, but do not implement or validate an LSP/code-action pipeline in this phase.

## Verdict

The current proposal is structurally sound after the source-map patches. The main remaining gaps are not about adding new code families or fallback behavior. They are about making the diagnostic system deterministic, testable, editor-ready, and resistant to registry/docs drift.

These patches are consistent with the user's requirement: no old compatibility, no fallback, and an elegant language/compiler surface before production release.
