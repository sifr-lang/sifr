# Review (Round 2): Ad-Hoc Phase — Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics

Reviewer: agent
Date: 2026-04-29
Source: `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`
Repo state at review: branch `main`, commit `38b1f9c9`
Lens: principal-engineer / compiler architecture
Prior review: `reviews/semantic-diagnostic-code-taxonomy-proposal-review.md`

## Verdict: READY WITH FOCUSED AMENDMENTS

The amended document closes the large majority of the round-1 blockers. Phase 27 reconciliation, `sifr_type_system` integration, the workspace string-classifier, the typed-span split, the registry-as-source-of-truth, the JSON Schema, the fixture-grammar update, the per-milestone fixture ownership, the family-overlap rules, and the hard guardrails are all now explicitly in scope.

What remains is a smaller set of principal-engineer-level concerns about (a) two real ordering inconsistencies the new sequencing introduces, (b) two data-model inconsistencies that will surface during the first migration PR, and (c) a small number of under-specified artifacts (doc-generator tool, source-span escape hatch, message-template substitution model) that will produce churn without alignment up front.

Severity tags: 🔴 blocker, 🟠 must-fix, 🟡 should-fix, 🟢 polish.

The no-fallback / no-historical-compatibility stance is preserved correctly throughout the amended document and does not need defending again.

---

## 1. Round-1 blocker resolution

For traceability, the 30 X-amendments from the round-1 review map onto the amended document as follows. Most are addressed; the few that are partial or open are flagged and re-stated in §2.

| Round-1 ref | Topic | Status in amended doc | Notes |
|---|---|---|---|
| X1 | e2e fixture grammar parser | ✅ Addressed | milestone_diag_5 + guardrail "no `is_message_error_code` or `diagnostic_error_code`" + negative `[E2507]` test. |
| X2 | Existing code renumbering | ✅ Addressed | Per-code table including PARSE/CODEGEN/BUILD/WORKSPACE renumbering. See §2 N1 for one residual inconsistency. |
| X3 | Stability scope (pre/post 1.0) | ✅ Addressed | "Stability Policy" section. Trigger event softer than ideal — see §2 N12. |
| X4 | Inventory of `ctx.error(...)` call sites | ✅ Addressed | milestone_diag_3 explicitly inventories all surfaces. See §2 N2 for ordering vs. registry. |
| X5 | Lossless JSON definition | ✅ Addressed | Round-trip identity, explicit `null`, `deny-unknown-fields`, schema regeneration check, all in milestone_diag_1 DoD. |
| X6 | Per-code docs pages | ✅ Addressed | "Every active code has a docs page under `docs/errors/<CODE>.md`; reserved codes are exempt." |
| X7 | `sifr_diagnostics` workspace member + dependency direction | ✅ Addressed | Dedicated "Dependency Ownership" section. |
| X8 | Per-milestone fixture/baseline ownership | ✅ Addressed | "No migration milestone is complete until its fixtures, verification baselines, and focused tests are green." |
| X9 | Span plumbing first | ✅ Addressed | `SourceSpan` lands in milestone_diag_1; milestone_diag_9 only closes coverage. |
| X10 | Decimal milestone covers `sifr_type_system::check` | ✅ Addressed | milestone_diag_6 explicitly. |
| X11 | Renderer / phase-mapping split | ✅ Addressed | diag_4a (renderer consumption) → diag_4b (phase-mapping deletion), with diag_4b ordered after diag_8 in the Mermaid graph. |
| X12 | Sequencing graph | ✅ Addressed | Mermaid sequencing flowchart. See §2 N2 for one ordering issue inside it. |
| X13 | Non-Option span at type level | ✅ Addressed | `SifrDiagnostic::Source(SourceDiagnostic { primary_span: SourceSpan, ... })`. See §2 N4 for the codegen-with-span loophole. |
| X14 | TextRange end-to-end + line/column derivation | ✅ Addressed | `SourceSpan { source_id, range: TextRange }`; `DiagnosticSpan` includes byte_start/end + line/column + end_line/end_column. |
| X15 | `ChildSeverity` (Note/Help only) | ✅ Addressed | `ChildSeverity` enum present; hard rule "no `Severity::Error` as a child severity." |
| X16 | INTERNAL allocation policy | ✅ Addressed | `SIFR-INTERNAL-9001` catch-all + dedicated codes for known families. |
| X17 | `CompilePhase` retired | ✅ Addressed | milestone_diag_4 + DoD: "Retire `CompilePhase` and the phase-derived `Display` label path." |
| X18 | Type System Integration | ✅ Addressed | "Type System Integration" subsection. `From<TypeError> for SifrDiagnostic` explicitly forbidden as long-term design. |
| X19 | Stdlib sub-allocation | ✅ Addressed | 5200..5599 active, 5600..5999 reserved, 50-code sub-ranges per stdlib module. |
| X20 | Family overlap rules | ✅ Addressed | "Family ownership rules for overlaps" subsection. See §2 N5 for one residual ambiguity. |
| X21 | Numbering convention | ✅ Addressed | "Numbering convention" subsection. See §2 N1 for one inconsistency the convention introduces. |
| X22 | Registry record shape | ✅ Addressed | milestone_diag_2 DoD lists `id, family, summary, state, docs_path, fixture, message_template, owner_module`. |
| X23 | Registry source-of-truth + drift CI | ✅ Addressed (mostly) | milestone_diag_2 DoD: "Make `crates/sifr_diagnostics/src/codes.rs` the source of truth. Generate human docs from the code registry." See §2 N6 — generator binary still unnamed. |
| X24 | JSON Schema checked in | ✅ Addressed | milestone_diag_1 DoD includes schemars-generated schema. |
| X25 | Versioned JSON envelope | ✅ Addressed | `{ "version": 1, "diagnostics": [...] }`. See §2 N7 for the `version` vs. per-diagnostic `schema_version` redundancy. |
| X26 | Coverage script | ✅ Addressed | milestone_diag_11 names `scripts/check_diagnostic_code_coverage.py`. |
| X27 | Per-file required doc edits | ✅ Addressed | "Required Documentation Updates" table. |
| X28 | Phase plan ordering | ✅ Addressed | "Relationship to Existing Roadmap" section. See §2 N12 for the residual amend-vs-reopen optionality. |
| X29 | Hard Rules additions | ✅ Addressed | All six round-1 additions present (HR-add-1..6). |
| X30 | PDoD additions | ✅ Addressed | Eight bullets present, including `sifr_diagnostics` ownership, `TypeError` retirement, `workspace_diagnostic_code` deletion, fixture-grammar update, JSON Schema, registry/docs sync, fixture coverage, Phase 27 status. |

Also preserved correctly:
- The semantic-vs-phase identity policy.
- The Non-Goals list (no compatibility aliases, no message-prefix classifiers, no historical migration layer).
- The recovery contract from milestone_27_5 (now milestone_diag_10) with `(severity, code, message_template, primary file)` grouping.

---

## 2. New and remaining gaps

The amendments above close 28/30 round-1 issues cleanly. The numbered findings below are issues either introduced by the amendment, partially addressed, or not raised in round 1 but visible now that the document is concrete enough to read end-to-end.

### N1. 🔴 Numbering convention contradicts the existing-code renumbering table

The "Numbering convention" subsection states:

> The family base is reserved and not used for an active diagnostic. The first active code in a range is base + 1, for example `SIFR-NAME-1001`.

The "Existing code renumbering" table immediately below states:

> `SIFR-PARSE-0001` — May remain if it is a real parser bucket with a registry entry; split later if parser can distinguish precise parse categories.

These are mutually exclusive. `SIFR-PARSE-0001` is the family base (`SIFR-PARSE-0001..0999`), so under the new numbering convention it must be Reserved, not Active. Either:

- Renumber the existing parser bucket to `SIFR-PARSE-0002` and reserve `SIFR-PARSE-0001`, **or**
- Make `SIFR-PARSE-0001` the documented exception (and explicitly weaken the numbering convention to "the first active code is base or base + 1, with the family base reserved unless the table says otherwise"), **or**
- Set the parser family base to `SIFR-PARSE-0000..0999` so `0001` is base + 1 and the convention holds.

This is a real implementability blocker because milestone_diag_2 requires "Every emitted code exists in the registry" with a `state` of Active/Reserved/Retired, and the registry's first row is internally inconsistent.

The same question applies in principle to `SIFR-WORKSPACE-0001..0103`: under the new family `SIFR-WORKSPACE-6000..6499`, the renumbering should presumably map `0001 → 6001`, `0002 → 6002`, …, `0101 → 6101`, `0102 → 6102`, `0103 → 6103`. The table does not pin this exact mapping. It should — the cost of leaving it implicit is two implementers picking two different mappings during baseline regeneration in milestone_diag_11.

### N2. 🔴 Registry milestone (diag_2) precedes inventory milestone (diag_3) in the sequencing graph, but the registry DoD requires the inventory to be complete

The Mermaid graph orders:

```
diag_1 → diag_2 → diag_3 → diag_4a → ...
```

milestone_diag_2 DoD includes:
- "Every emitted code exists in the registry."
- "Every active registry code has a fixture or is explicitly marked reserved."
- "Document code purpose, message shape, span policy, help policy, suggestion policy, and at least one example for each active code."

milestone_diag_3 DoD then says: "The inventory covers all raw HIR `ctx.error(...)` call sites" and "No diagnostic category is migrated without a known target code."

This is circular: you cannot list "every emitted code" in the registry until the inventory enumerates them, and the inventory's value as a worklist depends on having the registry to map into. With 517 `ctx.error/ctx.warn` call sites in `crates/sifr_hir/src/lower/` (verified) plus parser/type-system/codegen surfaces, this is not a theoretical ordering nit — it is a real bottleneck.

Two clean ways to resolve:

**(a) Swap the order.** Run inventory before registry: `diag_1 → diag_3 → diag_2 → diag_4a → ...`. The registry then encodes the result of the inventory and is verifiable.

**(b) Split diag_2.** `diag_2a` = registry skeleton (family ranges, code-record schema, generator tooling, no Active codes) before inventory; `diag_2b` = registry population (Active codes filled in from inventory) after inventory. The DoD that "Every emitted code exists in the registry" then attaches to `diag_2b`.

Option (b) is preferable because diag_2a's deliverables (`crates/sifr_diagnostics/src/codes.rs`, generator tool, JSON Schema check, family-range constants) are needed in code form before diag_3 can fill in target codes. (a) leaves nowhere for diag_3 to write the codes it discovers.

### N3. 🟠 `Result<LoweringResult, Vec<SifrDiagnostic>>` vs. `ctx.emit(...)` accumulator are not reconciled

"Target Architecture" says HIR returns:

```rust
Result<LoweringResult, Vec<SifrDiagnostic>>
```

"Diagnostic Builder API" says HIR emits via:

```rust
ctx.emit(Diagnostic::undefined_variable(name, span));
```

These two patterns describe different ownership models:

- `Result<T, Vec<S>>` is a one-shot return: success with a fully-lowered HIR or failure with a diagnostic vector. Multi-error recovery is impossible from a successful `Ok` branch.
- `ctx.emit(...)` implies a sink-based accumulator that collects diagnostics during lowering; the function returns `Ok(partial_hir)` even with diagnostics, and a separate "did errors occur" check decides whether to proceed.

milestone_diag_10 (Recovery Semantics) preserves bounded multi-error recovery and explicit recovery hard limits. That requires the accumulator pattern. The proposal should pick the accumulator pattern explicitly and amend the `Result<...>` shape to match. Suggested:

```rust
pub struct LoweringResult { /* HIR + recovery state */ }

impl<'a> LoweringContext<'a> {
    pub fn emit(&mut self, diagnostic: SifrDiagnostic);
    pub fn into_result(self) -> (LoweringResult, Vec<SifrDiagnostic>);
}
```

The driver then checks `diagnostics.iter().any(|d| d.severity() == Severity::Error)` to gate downstream phases.

This matters for the round-1 hard rule "Solve root causes": the proposal currently shows two incompatible APIs and an implementer will pick one and deviate from the document on the first PR.

### N4. 🟠 The `Source` / `Internal` split forces codegen errors with source spans into the wrong variant

The amendment introduces:

```rust
pub enum SifrDiagnostic {
    Source(SourceDiagnostic),     // primary_span: SourceSpan (mandatory)
    Internal(InternalDiagnostic), // no span field
}
```

Codegen errors (the `SIFR-CODEGEN-7000..7499` family) are listed as Source-equivalent — they originate in user code and the user can fix them. The Span Policy says: "Codegen diagnostics should preserve original source mapping where available."

But codegen failures are not always span-bearing: a panic boundary catches an unexpected codegen state with no AST node attached. Today `run_codegen_with_boundary` synthesizes a generic `SIFR-CODEGEN-0001` from a panic.

Under the amendment, the implementer faces three bad options:
1. Emit `SourceDiagnostic` with a fake `SourceSpan` covering the whole file → violates the no-fallback rule.
2. Emit `InternalDiagnostic` with no span → loses source context for codegen failures that *did* have a span at the failure site.
3. Add a third variant.

The proposal should resolve this with a stated rule. Cleanest: `SourceDiagnostic` is for any user-fixable error with a mandatory span (parser, HIR, type-system, codegen-with-span); `InternalDiagnostic` is exclusively the panic-boundary path emitting `SIFR-INTERNAL-*`. Codegen errors that lose source context but are not internal should not exist; if they do, they should be classified as internal and surfaced through `SIFR-INTERNAL-*` with the original codegen failure as a child note.

This needs a one-paragraph clarification in milestone_diag_1 or in "Source Mapping Architecture", because the split is otherwise a real degree of freedom that two implementers will resolve differently.

### N5. 🟠 `message: String` and `message_template: &'static str` co-existing is under-specified for compact grouping and JSON round-trip

The model carries both `message` (rendered) and `message_template` (grouping key). Compact grouping uses `message_template`. JSON serialization is "lossless" with round-trip identity.

Two unstated choices that will produce divergent implementations:

1. **Substitution storage.** If `message` is computed eagerly at emission and `message_template` is stored, the substitution variables are lost. JSON consumers cannot re-render the template in a different locale or with redacted values. To do that, the model needs `args: BTreeMap<String, JsonValue>` (or equivalent positional `Vec<JsonValue>`). The proposal does not include this field. Add it, or document that `message_template` is purely a grouping key and is *not* intended to be re-rendered downstream.

2. **Template grammar.** The example `"type mismatch: expected {expected}, got {actual}"` uses named braces. The proposal does not specify whether it is `{name}` (named), `{}` (positional), `{0}`/`{1}` (indexed), or whether `{` itself can be escaped as `{{`. Lossless JSON requires a single canonical form. Pin it down in milestone_diag_1.

Recommended: named-brace `{name}` with `{{`/`}}` escaping, and an `args: BTreeMap<String, JsonValue>` field per `SourceDiagnostic` (and per `InternalDiagnostic` if it is rendered through templates). The compact grouper uses `(severity, code, message_template, primary_file)`. The renderer substitutes `args` into `message_template` to produce `message` if `message` is omitted on the wire.

### N6. 🟠 Documentation generator binary is unnamed

milestone_diag_2 DoD says: "Generate human docs from the code registry rather than hand-maintaining divergent docs." and milestone_diag_11 DoD says "CI or local validation can regenerate docs and fail on drift."

But the document does not name:
- The generator binary (e.g., `cargo run -p sifr_diagnostics --bin gen-error-docs`).
- The drift-check script (e.g., `scripts/check_diagnostic_docs_sync.py`).
- The output paths committed to the repo (per the `https://sifr.sh/docs/errors/<CODE>` URL contract, the pages must exist statically somewhere, and `docs/errors/<CODE>.md` is the implied location).

Without naming the artifacts, two implementers will produce two different generators that don't share output, and the milestone_diag_11 guardrails will silently disagree.

Recommended one-paragraph addition in milestone_diag_2: "The generator is `cargo run -p sifr_diagnostics --bin gen-error-docs`. It writes `docs/errors/<CODE>.md` and `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` from `crates/sifr_diagnostics/src/codes.rs`. CI runs the generator and asserts `git diff --exit-code` against the registry."

### N7. 🟡 `schema_version: u32` (per-diagnostic) and envelope `"version": 1` are two version numbers for the same schema

The model has:

- `SourceDiagnostic.schema_version: u32`
- `InternalDiagnostic.schema_version: u32`
- Envelope `{ "version": 1, "diagnostics": [...] }`

Two version numbers for one schema is a long-term drift hazard. Either:

- Drop the per-diagnostic `schema_version` and rely on the envelope (preferred). Per-diagnostic versioning only matters if diagnostics travel outside the envelope, which is not the model.
- Keep both and document precisely what each means. Envelope-version-bumps and per-diagnostic-version-bumps need different change rules; that has to be written down or future maintainers will assume they can move independently.

The amended doc says `schema_version` is "not a compatibility promise for unreleased historical output" — that argument applies to the envelope version equally. One number is enough.

### N8. 🟡 No source-span escape hatch for HIR emission sites without a `TextRange`

Hard Rule: "Do not use `Option<TextRange>` for parser/HIR source diagnostics when a source range exists." The phrase "when a source range exists" leaves the case "no source range exists" undefined.

In practice: synthesized HIR nodes (e.g., desugared comprehensions, auto-generated `__init__` bodies, `__repr__` placeholders) may not carry a parser `TextRange`. The proposal does not say what to do.

Three defensible answers; the proposal must pick one:

1. **Parser invariant.** Every AST/HIR node has a `TextRange`. Synthesized nodes inherit the range of their nearest ancestor. Diagnostics on synthesized nodes use the inherited range.
2. **Pre-spanned helper.** A helper such as `Diagnostic::without_source_span(...)` exists and emits an `InternalDiagnostic` (not a `SourceDiagnostic`) — this prevents the loophole at the type level. Synthesized-node failures that have no real source location are explicitly internal-class.
3. **Explicit synthesized span type.** A `SourceSpan::synthesized(parent_range, reason: &'static str)` constructor that records "this span is inherited from ancestor X due to desugaring step Y."

(1) is preferred because it preserves the invariant that every `SourceDiagnostic` has a real, non-fake span. It must be enforced in the parser → HIR adapter, not in the diagnostic emitter.

### N9. 🟡 Severity enum is implicit; no canonical definition in the proposal

The proposal references `Severity::Error`, `Severity::Note`, `Severity::Warning`, `Severity::Help` (in code examples and hard rules) and introduces `ChildSeverity { Note, Help }` for children, but never lists the full `Severity` variants explicitly. The existing `sifr_driver` code has `Severity::{Error, Warning, Note, Help}`. The new model should pin this canonical list in milestone_diag_1.

In particular, an implementer reading the document might add `Severity::Info` (a common addition for `reveal_type`) or `Severity::Bug` (for unrecoverable internal panics). Lock it down: state the canonical variants are `{Error, Warning, Note, Help}` and that internal panics are `Severity::Error` with `SIFR-INTERNAL-*`.

### N10. 🟡 Warnings are now in scope but their accounting is unspecified

"Non-Error Diagnostics" section adds warnings to the same `SifrDiagnostic` stream (preferred path). milestone_diag_10 (Recovery) speaks of error-count limits. The proposal should state:

- Whether warnings count toward the recovery hard limit (currently 50 errors).
- Whether warnings are subject to compact grouping the same way errors are.
- Whether warnings affect exit codes (today exit codes follow milestone_27_6: 0/1/2/3 by severity-and-origin).
- Whether the JSON envelope `diagnostics: []` mixes errors, warnings, notes, and help, or splits them.

A one-sentence rule per item is enough. The current ambiguity will produce inconsistent renderer behavior across human/compact/JSON modes.

### N11. 🟡 Generic-bound vs. type-mismatch family ownership rule is one-line vague

Family ownership rule: "Generic bound/conformance failures are `SIFR-PROTO-*` when the failure is about satisfying a protocol, and `SIFR-TYPE-*` when the failure is about ordinary type compatibility."

The dividing line is not concrete. Two examples would resolve it:

- `fn foo[T: Comparable](x: T)` called with a non-`Comparable` type → `SIFR-PROTO-*` (constraint failure on a protocol bound).
- `fn foo(x: int)` called with `str` → `SIFR-TYPE-*` (no protocol involved).
- `fn foo[T](x: T) -> T` returning `int` when `T = str` is inferred → `SIFR-TYPE-*` (generic instantiation conflict, not a protocol bound).

Add these (or equivalent) to the family-ownership subsection so the inventory milestone (diag_3) does not invent its own boundary.

### N12. 🟡 Phase 27 status decision (amend vs. reopen) is left optional

"Relationship to Existing Roadmap" still offers two options:

> Update `internal_docs/roadmap.md` so Phase 27 is marked as amended by this ad-hoc phase, or reopened and then re-closed when this phase completes.

The implementer needs one. The cleaner option is "amend": Phase 27 stays completed in roadmap.md but a new "Amended by ad-hoc semantic diagnostic phase ([link])" sub-line is added. Reopening implies milestone_27_4 was failed-as-merged, which carries unclear consequences for subsequent phases that were gated on Phase 27 closure.

State the preferred option and the rationale.

Similarly, "Stability Policy" says "Post-1.0 stability begins at the first documented stable Sifr release" without naming the trigger phase. Phase 39 (Stable Channel GA, per the existing roadmap) is the natural anchor; pin it down to remove the ambiguity.

### N13. 🟡 milestone_diag_11 DoD double-counts items already promised by earlier milestones

The final guardrail milestone lists items such as:

- "The JSON schema is checked in and synchronized with the Rust model." — already DoD of milestone_diag_1.
- "Active registry codes have generated docs pages." — already DoD of milestone_diag_2.
- "Active registry codes have representative fixture coverage." — already DoD of every migration milestone (X8).

These should be reframed in milestone_diag_11 as **enforcement** items (the guardrail script that verifies them), not redo'd as DoD items. Otherwise milestone_diag_11 will be artificially gated on re-doing earlier work and the dependency graph becomes brittle.

Suggested rewrite: "milestone_diag_11 verifies that the guardrails added in this milestone catch violations of the constraints established in diag_1 through diag_10." Then list only the new guardrail tests/scripts in the scope — not the prior constraints themselves.

### N14. 🟢 `SourceId(String)` is an implementation detail that will get changed in week 2

```rust
pub struct SourceId(String);
```

In a multi-file project a `SourceId` is hot — it appears in every span, every diagnostic, every renderer call. `String` allocates per source. The first PR that runs project-mode against a moderately-large workspace will replace this with `SourceId(Arc<str>)` or an interned `u32`.

Drop the concrete type from the proposal text and say `SourceId` is opaque and cheaply cloneable. Implementation chooses interning vs. `Arc<str>` based on performance characteristics observed during milestone_diag_1.

### N15. 🟢 Non-error diagnostics ("Prefer" path) is preferred but not chosen

"Non-Error Diagnostics" section offers two paths:
- "Prefer: model them as `SifrDiagnostic` values with `Severity::Note` or `Severity::Warning`."
- "If a value is intentionally not part of diagnostics, document that boundary..."

For a principal-engineer-grade plan, "prefer" is not a decision. Choose one. The clear answer is the "prefer" path: `reveal_type` is `Severity::Note` with code `SIFR-INFO-NNNN` (or use a dedicated `SIFR-REVEAL-*` family) and warnings are `Severity::Warning` with their own family codes. Pin it down so milestone_diag_3 doesn't have to re-litigate.

### N16. 🟢 Existing-code renumbering table is incomplete for `SIFR-WORKSPACE-*`

The renumbering table says "`SIFR-WORKSPACE-0001..0103` Renumbered into `SIFR-WORKSPACE-6001..6499` with no aliases." But the existing codes are non-contiguous (0001..0004 + 0101..0103, verified in source). The table should specify the exact mapping: either preserve the 0001/0101 split structure in the new family (`6001..6004`, `6101..6103`) or compact to contiguous (`6001..6007`). Either is fine; pick one and write it down.

---

## 3. No-fallback / no-historical-compatibility stance

The amended document preserves the stance correctly. Specifically:

- "Pre-1.0 stability means diagnostic codes can be renamed, split, or retired only through an explicit registry change accompanied by fixture, baseline, and docs updates in the same milestone. **No compatibility aliases are required before public release.**" — Correct.
- Hard Rules: "Do not preserve `SIFR-TYPE-0001` compatibility." / "Do not embed secondary codes in messages." / "Do not map strings to codes after the fact." — Correct and unambiguous.
- "Type System Integration": "Do not add `impl From<TypeError> for SifrDiagnostic>` as the long-term design. That recreates a hidden classifier layer..." — Correct, with the right explanation.
- "Existing Surface Inventory" closes with: "This inventory is not a compatibility table. It is a migration worklist used to ensure no raw diagnostic path survives." — Correct.

One potential erosion to watch: "Re-export diagnostic types from `sifr_driver` only as a temporary internal convenience during the same phase, not as the owning definition." This is acceptable as written — but it should have a deletion deadline. Recommend: re-exports must be removed by milestone_diag_4b (the same milestone that retires `CompilePhase`). Without that pin, the re-export becomes the long-term API by accident.

---

## 4. Concrete amendments worth applying before implementation

Only amendments still worth changing — most round-1 amendments are already in. Numbers below are independent of the round-1 X-numbers.

| # | Section | Edit | Severity |
|---|---|---|---|
| Y1 | Numbering convention + Existing code renumbering | Resolve the `SIFR-PARSE-0001` contradiction. Preferred: renumber to `SIFR-PARSE-0002` and reserve `SIFR-PARSE-0001` as the family base. Apply uniformly. | 🔴 |
| Y2 | Sequencing graph + milestone_diag_2 | Either swap diag_2 ↔ diag_3, or split diag_2 into diag_2a (registry skeleton + tooling) before diag_3 and diag_2b (registry population) after diag_3. State the chosen option in the Mermaid graph. | 🔴 |
| Y3 | Target Architecture | Reconcile `Result<LoweringResult, Vec<SifrDiagnostic>>` with `ctx.emit(...)`. Pick the accumulator pattern explicitly: `LowerCtx` collects diagnostics; lowering returns `(LoweringResult, Vec<SifrDiagnostic>)` or equivalent. Update the return-type sketch and remove the `Result<...>` framing. | 🟠 |
| Y4 | milestone_diag_1 + Source Mapping Architecture | Clarify the `Source` / `Internal` split for codegen: codegen errors with a source span are `SourceDiagnostic`; codegen failures lacking a span are explicitly classified as internal and use `SIFR-INTERNAL-*`. There is no third variant. | 🟠 |
| Y5 | milestone_diag_1 (data model) | Add `args: BTreeMap<String, JsonValue>` (or equivalent positional vector) to `SourceDiagnostic`/`InternalDiagnostic` so JSON consumers can re-render `message_template`. Specify the `{name}` template grammar and `{{`/`}}` escaping. | 🟠 |
| Y6 | milestone_diag_2 + milestone_diag_11 | Name the generator binary explicitly: `cargo run -p sifr_diagnostics --bin gen-error-docs`. Name the drift check (script path or `cargo` subcommand). State the canonical output paths. | 🟠 |
| Y7 | Target Architecture | Drop one of `SourceDiagnostic.schema_version` / `InternalDiagnostic.schema_version` or the envelope `"version": 1`. One version number is enough. Recommend keeping the envelope version. | 🟡 |
| Y8 | Span Policy | Add a paragraph: "Synthesized HIR nodes inherit the `SourceSpan` of their nearest ancestor with a parser-assigned range. The parser/HIR adapter must guarantee every node has a `SourceSpan` before lowering emits diagnostics. Diagnostics that have no real source mapping use `SIFR-INTERNAL-*`, not a fabricated source span." | 🟡 |
| Y9 | milestone_diag_1 | Lock the `Severity` enum: `{Error, Warning, Note, Help}`. State that `SIFR-INTERNAL-*` uses `Severity::Error`. State that `reveal_type` uses `Severity::Note`. | 🟡 |
| Y10 | milestone_diag_10 (Recovery) + Non-Error Diagnostics | Specify warning accounting: do warnings count toward the 50-error limit? Do they affect exit code? Do they share `diagnostics: []` with errors in the JSON envelope? Choose one rule per question. | 🟡 |
| Y11 | Family ownership rules | Add concrete examples for the generic-bound vs. type-mismatch boundary (see N11 above). | 🟡 |
| Y12 | Relationship to Existing Roadmap + Stability Policy | Pick one: Phase 27 is **amended** (not reopened). Pin post-1.0 stability trigger to Phase 39 (Stable Channel GA). | 🟡 |
| Y13 | milestone_diag_11 | Reframe DoD bullets that duplicate earlier milestone DoDs as enforcement items (the guardrail scripts that verify them), not as redo'd work. | 🟡 |
| Y14 | Non-Error Diagnostics | Pick the "Prefer" path explicitly: `reveal_type` and warnings are `SifrDiagnostic` values. Drop the alternative. | 🟢 |
| Y15 | Source Mapping Architecture | `SourceId` is opaque and cheaply cloneable; implementation chooses representation. Drop the concrete `String` newtype. | 🟢 |
| Y16 | Existing code renumbering | Pin the exact mapping for the non-contiguous `SIFR-WORKSPACE-0001..0004 + 0101..0103` codes (preserve split, or compact). | 🟢 |
| Y17 | Dependency Ownership | Add a deletion deadline: "Re-exports from `sifr_driver` must be removed by milestone_diag_4b." | 🟢 |

---

## 5. Strengths preserved from round 1

These remain correct and should not be touched:

- The semantic-vs-phase distinction in "Design Principle".
- The Non-Goals list.
- The renumbering table (modulo Y1 and Y16).
- The `SifrDiagnostic::Source(...)` / `SifrDiagnostic::Internal(...)` split (modulo Y4).
- The `ChildSeverity { Note, Help }` enum.
- The `(severity, code, message_template, primary file)` compact grouping rule.
- The hard rule against message-prefix classifiers and message-embedded pseudo-codes.
- The "Type System Integration" subsection rejecting `impl From<TypeError> for SifrDiagnostic>` as a long-term design.
- The decimal renumbering table.
- The per-milestone fixture/baseline ownership rule.

---

## 6. Bottom line

The amended proposal is substantially closer to a directly implementable plan than the round-1 draft. 28 of 30 round-1 amendments are addressed cleanly. The two remaining round-1 issues (registry/inventory ordering, numbering-vs-renumbering contradiction) and the new findings around the accumulator vs. `Result` ambiguity, the codegen-span loophole, the `message_template` substitution model, and the unnamed generator binary are real but small. With Y1–Y6 applied (the four 🔴/🟠 blockers and the two top must-fixes), the document is implementable as-is and a milestone_diag_1 PR can begin without follow-up clarifications.

The no-fallback / no-historical-compatibility stance is intact. The scope is large but bounded. The sequencing now correctly places span primitives, registry, and renderer-consumption before family migration, and per-milestone fixture ownership prevents the round-1 mass-fixture-cascade risk.

Recommended next step: apply Y1, Y2, Y3, Y4, Y5, Y6 and re-pin (no third review needed for those changes). Treat Y7–Y17 as polish landed during milestone_diag_1 / diag_2 PRs rather than as gating amendments.
