# Review: Ad-Hoc Phase — Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics

Reviewer: Claude Opus 4.7
Date: 2026-04-29
Source: `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`
Repo state at review: branch `main`, commit `38b1f9c9`
Lens: principal-engineer / compiler architecture

## Verdict: NOT READY — significant amendments required before implementation

The intent of the proposal is correct and the destination architecture is sound: a typed `DiagnosticCode`, a shared diagnostic model, structured emission at the checker site, and the elimination of `SIFR-TYPE-0001` as a generic semantic bucket.

However, in its current form the document is not directly implementable as a principal-engineer-grade plan. It (a) silently overlaps and conflicts with the already-merged Phase 27, (b) misses two of the three diagnostic surfaces in the codebase (`sifr_type_system::TypeError`, the workspace string-classifier in `sifr_driver`), (c) under-specifies the data model in ways that will produce churn during implementation (span type, severity discipline, internal vs. codegen panics, warnings, reveal-type), (d) leaves the milestone sequencing exposed to large-fixture-cascade risk, and (e) is silent on several concrete artifacts the registry/guardrail goals depend on (registry source-of-truth, fixture-grammar update, JSON Schema, doc-site contract).

The findings below are organised as: **G** (gaps), **A** (weak assumptions), **S** (sequencing risks), **D** (data-model issues), **T** (taxonomy/numbering), **R** (rollout/doc), and **X** (concrete amendments to apply to the issue document). Severity tags: 🔴 blocker, 🟠 must-fix, 🟡 should-fix, 🟢 polish.

---

## 1. Missing related work and context

### G1. 🔴 The proposal is not connected to Phase 27 (which already shipped a partial form of this work)

`internal_docs/roadmap.md` lists Phase 27 ("Runtime Safety and Diagnostics Contract") as **completed**, with `milestone_27_4` ("Span and Diagnostic Schema Quality") explicitly scoped to:

> Replace the current predominantly string-oriented frontend diagnostic plumbing with one canonical structured diagnostic model shared by parser/lowering/type-check/codegen … Standardize stable diagnostic codes, related-span labels, help text, deterministic documentation URLs, and structured fix-suggestion fields.

The exit gate of Phase 27 reads "Compiler diagnostics are stable, span-accurate, recovery-capable, and panic-free on user input." Yet:

- HIR emits string-based `LoweringError { message, line: Option<u32>, col: Option<u32> }` ([crates/sifr_hir/src/lower/mod.rs:81-97](crates/sifr_hir/src/lower/mod.rs:81)) — not a structured diagnostic.
- `sifr_driver` derives the public code from `CompilePhase`, not from a registry ([crates/sifr_driver/src/diagnostics.rs:130-140](crates/sifr_driver/src/diagnostics.rs:130)).
- 90 of 179 e2e fail fixtures still expect `SIFR-TYPE-0001` as a catch-all (verified: `grep -c "SIFR-TYPE-0001" crates/sifr/tests/e2e/fail/*.sifr | grep -v ":0$" | wc -l`).

Phase 27 was therefore under-delivered. This proposal is in substance a **remediation of milestone_27_4 + 27_5**, not an "ad-hoc" parallel track. The document must:

1. Cite Phase 27 explicitly under a new "Relationship to existing roadmap" section.
2. State which Phase 27 milestone(s) this work amends or supersedes.
3. Require an update to `internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md` and `internal_docs/roadmap.md` (Phase 27 row) describing the corrected exit gate.
4. Decide whether Phase 27 is reopened, or whether this counts as an addendum delivered post-completion. Either way, the "completed" status in roadmap.md must change or be qualified.

Without this connection, two reviewers will reach contradictory conclusions about whether Phase 27's exit gate is honoured, and the next phase that depends on stable diagnostics (LSP, editor tooling, decimal/borrow expansion) will inherit ambiguous state.

### G2. 🔴 Architecture document already mandates a *different* code format

`internal_docs/architecture.md:711` says:

> every top-level Sifr compiler diagnostic has a stable code owned by a specific compiler phase (parser, type checker, borrow checker, codegen). **Error codes use `E####` and warning codes use `W####`.** `Note` and `Help` entries attach to a parent diagnostic instead of defining separate top-level codes.

The proposal silently introduces a different format (`SIFR-FAMILY-NNNN`) without acknowledging or amending this contract. This is a hard contradiction: either architecture.md is the source of truth and the proposal must adopt `E####`/`W####`, or the proposal is a deliberate change and architecture.md must be updated as part of milestone_diag_2.

The proposal's existing precedent is mixed — `SIFR-PARSE-0001`, `SIFR-WORKSPACE-0101` etc. are already in code, while `[E2501]` is the message-embedded form. The proposal's choice (`SIFR-FAMILY-NNNN`) is the better format for a public language because it carries family identity in the prefix and is grep-friendly. But the architecture doc must be amended in lockstep, **and** the "Note/Help attach to parent" rule from architecture.md must be carried over (the proposal's `DiagnosticChild` allows arbitrary `Severity`, which is incompatible with architecture.md unless it is restricted to Note/Help).

### G3. 🟠 The proposal misses one of the three diagnostic surfaces

The codebase has **three** diagnostic surfaces today:

1. `sifr_hir::LoweringError` ([crates/sifr_hir/src/lower/mod.rs:83](crates/sifr_hir/src/lower/mod.rs:83)) — explicitly named.
2. `sifr_driver::CompileError` + `CompilerDiagnostic` ([crates/sifr_driver/src/diagnostics.rs](crates/sifr_driver/src/diagnostics.rs)) — explicitly named.
3. **`sifr_type_system::TypeError` + `TypeErrorKind`** ([crates/sifr_type_system/src/lib.rs:31-65](crates/sifr_type_system/src/lib.rs:31)) — **not mentioned** in the proposal at all.

`TypeError` is already a discriminated enum (`TypeMismatch`, `UndefinedVariable`, `WrongArgumentCount`, `UseAfterMove`, etc.) — the *closest* thing in the tree to the proposal's target shape. It has no span and no code, but it is the natural seed for the typed `DiagnosticCode` enum. Worse, `sifr_type_system::check` emits `[E2503]/[E2504]` decimal pseudo-codes directly in `message: String` ([crates/sifr_type_system/src/check.rs:31,43,361,373](crates/sifr_type_system/src/check.rs:31)) — milestone_diag_4 will silently miss those if the proposal does not list `sifr_type_system` in scope.

**Required:** every milestone scope and the "shared diagnostic model" dependent list must explicitly include `sifr_type_system`. The migration plan must say that `TypeError` either (a) is wrapped into the canonical `SifrDiagnostic` at HIR boundaries, or (b) is replaced by direct emission of `SifrDiagnostic` from the type system. (Option (b) is cleaner, given the "no compatibility layer" hard rule.)

### G4. 🟠 The proposal misses the existing string-prefix classifier it forbids

A hard rule says "Do not map strings to codes after the fact / Do not infer codes from message prefixes." That rule is **already violated** by [crates/sifr_driver/src/diagnostics.rs:96-128](crates/sifr_driver/src/diagnostics.rs:96), which scans `CompileError.message` for substrings to assign `SIFR-WORKSPACE-0001..0103`.

The proposal must explicitly call out the dead code: `CompileError::workspace_diagnostic_code` and the matching `match self.phase { ... }` branch are to be deleted and replaced by direct registry-keyed emission from `crates/sifr_driver/src/workspace/`. Otherwise an implementer will leave the classifier in place because no milestone scope names it.

### G5. 🟠 The fixture grammar parser is silent in the proposal

`crates/sifr/tests/e2e.rs:596-685` (`parse_expected_error`, `is_diagnostic_code`, `is_message_error_code`, `diagnostic_error_code`, `normalize_error_code`) accepts both `SIFR-XXXX-NNNN` codes and bare `Edddd` codes from message bodies. After this phase:

- `is_message_error_code` and `diagnostic_error_code` must be removed.
- `is_diagnostic_code` must validate the new family-prefix set against the registry (or accept any `SIFR-[A-Z]+-\d{4}` if the registry is the gate elsewhere).
- The `[CODE]` square-bracket fallback path (line 602) should remain only for the grammar `[SIFR-XXXX-NNNN]:`, not for `[Edddd]`.

The proposal's milestone_diag_8 lists "Update all e2e fail annotations" but does not name this parser. **Concrete amendment X1 below.**

### G6. 🟡 No mention of `reveal_type()` and warnings

`LowerCtx` has `reveal_types: Vec<String>` and `warnings: Vec<String>` ([crates/sifr_hir/src/lower/mod.rs:113-115](crates/sifr_hir/src/lower/mod.rs:113)). Architecture.md says warnings use `W####`. The proposal scopes only errors. It must explicitly say:

- Whether `reveal_type` notes flow through the same `SifrDiagnostic` channel with `Severity::Note` and a stable code (e.g. `SIFR-INFO-0001` or a dedicated family) — and where they appear in JSON.
- How warnings get codes (proposal's families are all error-shaped; some families like `SIFR-OWN-*` will inevitably emit warnings if/when borrow checking gains a soft mode).
- That `LowerCtx::warn(String)` is migrated to `ctx.emit_warning(...)` with the same registry/code discipline as errors.

### G7. 🟡 Missing reference to `crates/sifr_frontend` (planned but not extant)

`internal_docs/architecture.md:225` documents `sifr_frontend/` as the canonical query facade, but **no such crate exists in the workspace** (verified: `Cargo.toml` `[workspace]` members + `ls crates/`). The proposal's "Acceptable only if crate sequencing makes a separate crate impractical" fallback (`crates/sifr_frontend/src/diagnostics.rs`) refers to a nonexistent crate. This is misleading.

The proposal should either:
- Drop the `sifr_frontend` fallback (recommended — `sifr_diagnostics` is the right answer and orthogonal to the frontend facade), **or**
- Acknowledge that creating `sifr_frontend` is itself a multi-phase undertaking (it is, per architecture.md) and not block on it.

### G8. 🟡 No mention of the existing recovery contract from milestone_27_5

The proposal's milestone_diag_3 says "Ensure compact grouping uses `(severity, code, canonical message, primary file)`." That contract already lives in `apply_diagnostic_recovery_limits` ([crates/sifr_driver/src/diagnostics.rs:178-221](crates/sifr_driver/src/diagnostics.rs:178)) and the compact renderer ([crates/sifr/src/main.rs:300-365](crates/sifr/src/main.rs:300)), with documented limits 50 / 5 / 5. The proposal must:

- Cite the milestone_27_5 limits as preserved and unchanged.
- Specify that grouping continues to operate on canonicalised diagnostics (post-builder, pre-render), since two diagnostics with the same code but different child notes are semantically distinct and must not be over-merged.

---

## 2. Weak assumptions

### A1. 🔴 "No fallback / no migration" is ambiguous given the existing `SIFR-WORKSPACE-0001..0103` codes

Existing workspace diagnostics already use the proposal's `SIFR-FAMILY-NNNN` shape but in the *0001..0103* range. The proposal allocates `SIFR-WORKSPACE-6000..6499`. This is a **renumbering** of already-emitted codes. The hard rule "Do not preserve … compatibility" technically allows it, but the proposal does not address:

- Whether `SIFR-PARSE-0001`, `SIFR-CODEGEN-0001`, `SIFR-BUILD-0001` are *also* renumbered (they don't fit the family-base offsets in the table — `SIFR-PARSE-0001` lives in `0001..0999`, but `SIFR-WORKSPACE-0001` does not live in `6000..6499`).
- How fixtures referencing `SIFR-WORKSPACE-0101`, `SIFR-WORKSPACE-0102`, `SIFR-WORKSPACE-0103` ([crates/sifr_driver/src/tests/diagnostics.rs:44-52](crates/sifr_driver/src/tests/diagnostics.rs:44)) get migrated.
- Whether the existing test `SIFR-PARSE-0001` is a single bucket (still acceptable as long as it has a registry entry) or itself needs sub-codes.

**Concrete amendment X2:** add a paragraph "Existing code renumbering" after "Proposed Diagnostic Families" stating which currently-emitted codes are renumbered, with a one-row table per existing code → new code. Explicitly call out that no aliases are kept and that all baselines, source classifiers, and tests are updated in the same PR as each rename.

### A2. 🟠 "Stable codes" is undefined pre-1.0

The proposal repeats "stable" 8+ times but does not define stability scope. Sifr is pre-release; reasonable definitions:

- **Pre-1.0:** a code may be retired or renumbered only via a registry change accompanied by a fixture/baseline update. Not a backwards-compat promise.
- **Post-1.0:** codes are immutable; new categories require new codes; retirements use `SIFR-DEPRECATED-XXXX` markers.

**Concrete amendment X3:** add a "Stability" section before "Hard Rules" stating the pre-1.0 rule and naming the trigger that flips to post-1.0 (likely Phase 39 — Stable Channel GA).

### A3. 🟠 "If a helper is missing, the implementation should add the helper and assign the code deliberately"

There are **517 `ctx.error(...)` / `ctx.warn(...)` call sites** in `crates/sifr_hir/src/lower/` (verified: `grep -rn "ctx.error\|ctx\.warn" crates/sifr_hir/src/lower --include="*.rs" | wc -l`). The proposal does not estimate how many distinct codes this will produce, who reviews the code-to-call-site mapping, or whether de-duplication happens before milestone_diag_8.

This is a **scale assumption**, not a sequencing detail. A 50-code design is very different from a 200-code design. Without an estimate, the registry, docs site, and fixture coverage targets are unsized.

**Concrete amendment X4:** require `milestone_diag_2` to produce a one-time inventory: "for each `ctx.error(...)` site, what is the proposed code?" — checked in as a CSV or appendix. The inventory is the gate to milestone_diag_5/6, not afterthought.

### A4. 🟡 "Make JSON serialization lossless"

Lossless is not defined. Concretely it should mean:

- Round-trip identity: `serde_json::from_str(serde_json::to_string(&d)?)? == d` for all `SifrDiagnostic` instances.
- All `Option<...>` fields serialise to `null` when `None` (not skipped), so consumers can rely on field presence.
- `#[serde(deny_unknown_fields)]` is used for deserialisation paths.
- A JSON Schema is checked in (or generated; `schemars` is already in workspace deps), with a regression test asserting schema/struct alignment.

**Concrete amendment X5:** lift these into milestone_diag_1 DoD.

### A5. 🟡 "Every emitted diagnostic must derive a deterministic docs URL"

`format!("https://sifr.sh/docs/errors/{code}")` is already implemented. The proposal does not say whether the docs site must actually serve a page for every emitted code (today most likely 404), nor who owns producing those pages. Two failure modes:

1. URLs that 404 are emitted to users → bad first impression.
2. CI does not check URL liveness → drift goes silent.

**Concrete amendment X6:** require `docs/errors/<CODE>.md` (or a generated landing page) to exist for every active registry code, enforced by a guardrail script that reads the registry and asserts the file is present. Mark "reserved" codes as not requiring a doc page.

### A6. 🟡 "Move or recreate the canonical diagnostic structures there"

`crates/sifr_diagnostics` does not exist. "Move or recreate" elides the actual decision. Recommended: create the new crate, define the canonical types there, and have `sifr_driver` re-export only what its public API requires. The driver-owned `CompileError`/`CompilePhase`/`CompileResultFull` must be either retired or demoted to a driver-internal type that wraps `Vec<SifrDiagnostic>`.

**Concrete amendment X7:** `milestone_diag_1` DoD must explicitly say `crates/sifr_diagnostics` is added as a workspace member with `[lints]` inheritance, that it has zero sifr-internal dependencies, and that `sifr_hir`, `sifr_codegen`, `sifr_driver`, `sifr_type_system`, and `sifr` (CLI) all add it as a dependency.

---

## 3. Sequencing risks

### S1. 🔴 milestone_diag_5 and milestone_diag_6 will simultaneously break ~150 fixtures

90 of 179 fail fixtures expect `SIFR-TYPE-0001`. milestone_diag_5 ("Name, Import, Type, and Call Diagnostics") plus milestone_diag_6 ("Ownership, Flow, Match, …") together cover most of those 90 fixtures. The proposal places "Update all e2e fail annotations" in milestone_diag_8 — the **last** milestone. Between milestones 4–8 the fixture suite will be wedged.

This is unworkable: PRs in 5/6 either (a) skip fixture updates and turn CI red for weeks, or (b) update fixtures inline, in which case milestone_diag_8's "Update all e2e fail annotations" is empty.

**Concrete amendment X8:** restructure milestone DoDs so that **each migrating milestone owns its own fixture/baseline updates**. milestone_diag_8 then contains only the *new* guardrail tests and the residual cleanup. State this explicitly: "no migration milestone is complete until its fixtures and verification baselines reflect the new codes and the suite is green."

### S2. 🟠 Span threading (milestone_diag_7) is ordered after migration (milestones 4–6)

Threading `TextRange` through HIR helpers requires touching every helper signature. If you migrate to typed builders in milestone_diag_5/6 first and then add spans in 7, you will edit every builder twice. Reverse the dependency: span/range plumbing first (or as a sub-step of milestone_diag_2/5), then the family migration.

A defensible split:

- `milestone_diag_5a`: extend `LowerCtx::error` to take a `TextRange` and propagate it; mechanical, no public-facing change.
- `milestone_diag_5b`: replace `ctx.error(String)` with typed builders that already include the span (the helper signature is `(span, ...args)`).
- `milestone_diag_7`: cleanup of any remaining spanless emitters, add related spans.

**Concrete amendment X9:** reorder/split milestone_diag_5 to plumb spans first, then migrate codes, then add related spans. Document this in the Milestones section.

### S3. 🟠 Decimal-first migration (milestone_diag_4) is fine but must include `sifr_type_system`

milestone_diag_4 mentions only HIR/decimal but `[E2503]/[E2504]` are emitted from `sifr_type_system::check` ([crates/sifr_type_system/src/check.rs:31](crates/sifr_type_system/src/check.rs:31)). Without explicit scope, an implementer will land milestone_diag_4 PR, see decimal mixed-arithmetic still emit `[E2503]`, and re-open the milestone.

**Concrete amendment X10:** milestone_diag_4 scope must list "Replace `[E2503]`, `[E2504]` emission in `sifr_type_system::check.rs`" alongside the HIR changes.

### S4. 🟡 milestone_diag_3 ("Renderer and Driver Integration") cannot complete before milestone_diag_4–6 land

The DoD says `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` is gone "from public diagnostic code assignment." But you cannot remove that mapping until *every* lowering site emits a structured code. Either:

- milestone_diag_3 is split: "renderers consume `SifrDiagnostic`" (achievable early) vs. "phase-to-code mapping deleted" (achievable only after 5/6).
- OR milestone_diag_3 becomes the integration milestone that lands *with* milestone_diag_6.

**Concrete amendment X11:** split milestone_diag_3 into 3a (renderer consumption) and 3b (phase-to-code mapping deletion). Pin 3b to land with or after milestone_diag_6.

### S5. 🟡 No dependency graph between milestones

The proposal lists 8 milestones but does not say which depend on which. An explicit dependency graph (Mermaid or text) prevents implementers from queueing PRs in the wrong order.

Suggested graph:

```
diag_1 (model) → diag_2 (registry) → diag_3a (renderer) → diag_4 (decimal) → diag_5 (name/import/type/call) → diag_6 (own/flow/match/…) → diag_3b (phase mapping deletion) → diag_7 (spans completion) → diag_8 (guardrails + final baselines)
```

**Concrete amendment X12:** add a "Sequencing" subsection with an explicit dependency graph.

---

## 4. Data-model issues

### D1. 🔴 `primary_span: Option<DiagnosticSpan>` is too permissive for HIR/parser diagnostics

The proposal's hard rule "Do not allow spanless HIR diagnostics when the AST node has a source location" is not enforceable at the type level if `primary_span` is `Option`. An implementer will reach for `None` whenever the call site is awkward.

Recommended split:

```rust
pub enum SifrDiagnostic {
    Source(SourceDiagnostic),       // always carries a primary_span
    Internal(InternalDiagnostic),   // panic boundary, build, workspace path errors
}
```

Or a typestate / `NonOptional<DiagnosticSpan>` for source diagnostics. Either way, `Option<DiagnosticSpan>` should not be the path for HIR-emitted errors.

If structural separation is too heavy, at minimum:

- Make `primary_span` non-`Option` on a sub-type used by HIR/parser builders.
- Require a builder-level test that asserts every HIR-builder helper receives a `TextRange`, not an `Option<TextRange>`.

**Concrete amendment X13:** specify the typed split (or the equivalent invariant) in milestone_diag_1's DoD.

### D2. 🔴 Span type is unspecified

`DiagnosticSpan` in driver today is `{ file, line, column }` ([crates/sifr_driver/src/diagnostics.rs:55-60](crates/sifr_driver/src/diagnostics.rs:55)) — a single point, not a range. Real diagnostics need ranges (start_byte, end_byte, or start_line/start_col + end_line/end_col) to highlight underlines.

`ruff_text_size::TextRange` is already a workspace dep and is used in the parser. The proposal must:

- Adopt `TextRange` (or `(Position, Position)` byte-range) end-to-end.
- Define how `TextRange` is mapped to file URIs and line/column for human/JSON renderers.
- Specify how multi-file diagnostics encode the file (logical module path? file URI? canonical absolute path?).

**Concrete amendment X14:** `milestone_diag_1` data model must specify span type as a byte-range with a file ref, and the JSON schema must include both `byte_offset_start/end` and computed `line/column` (computing line/col from a byte offset is a renderer concern, but lossless JSON requires the byte offsets to be present).

### D3. 🟠 `Severity` of children must be restricted

`DiagnosticChild { severity: Severity, message: String }` permits `Severity::Error` on a child, which contradicts architecture.md ("Note and Help entries attach to a parent diagnostic instead of defining separate top-level codes"). Children should be restricted to `Severity::Note | Severity::Help`.

**Concrete amendment X15:** introduce a `ChildSeverity { Note, Help }` enum, or document the invariant and enforce it in the constructor.

### D4. 🟠 Internal/panic diagnostics need a clear path

`run_codegen_with_boundary` ([crates/sifr_driver/src/diagnostics.rs:255-267](crates/sifr_driver/src/diagnostics.rs:255)) wraps panics as `phase: Codegen` → `SIFR-CODEGEN-0001`. Per the proposal's table, panics should be `SIFR-INTERNAL-9000..9999`. The proposal doesn't explain:

- How the panic boundary picks the right INTERNAL code (catch-all `SIFR-INTERNAL-9001`? family-tagged based on the wrapped phase?).
- Whether the panic boundary expands to cover lowering and parser, not just codegen (Phase 27_6 lists "Convert remaining user-triggerable panics to diagnostics" as in-scope; this proposal must be consistent with that).

**Concrete amendment X16:** specify INTERNAL code allocation policy: a stable catch-all `SIFR-INTERNAL-9001` for unclassified panics + dedicated codes for known panic families. Update the panic-boundary functions to emit `SIFR-INTERNAL-*`.

### D5. 🟡 `CompilePhase` should be retired, not retained

The proposal says: "CompilePhase may remain useful for exit-code grouping and internal boundaries." But:

- Exit codes (per Phase 27_6) are determined by *severity and origin*, not phase: 0 success, 1 user, 2 CLI, 3 internal. Phase is irrelevant.
- The `Display` impl on `CompileError` ([crates/sifr_driver/src/diagnostics.rs:223-233](crates/sifr_driver/src/diagnostics.rs:223)) hardcodes "parse error", "type error", "codegen error", "build error" — which is exactly the phase-derived public framing the proposal forbids.

**Concrete amendment X17:** mark `CompilePhase` for removal (not retention) in milestone_diag_3b. Replace the human-readable label with a derivation from `DiagnosticCode`'s family.

### D6. 🟡 No canonical mapping from `TypeError` to `SifrDiagnostic`

The proposal's typed builder examples (`Diagnostic::undefined_variable`, etc.) are HIR-shaped. But `sifr_type_system::TypeErrorKind` already has equivalent variants. The proposal should specify whether `TypeError` is:

- (a) Converted at the HIR boundary into `SifrDiagnostic` via an `impl From<TypeError> for SifrDiagnostic`. Risk: indirection that re-creates a hidden classifier.
- (b) Replaced by direct emission of `SifrDiagnostic` from the type system. Cleaner.

**Recommendation: (b)**, with `TypeErrorKind` being deleted in favour of typed builder calls living next to the type-checking helpers. This aligns with the proposal's "no compatibility layer" hard rule.

**Concrete amendment X18:** add "Type System Integration" subsection: `sifr_type_system` adopts the canonical model directly; `TypeError`/`TypeErrorKind` are retired; type-checking helpers either (i) take a `&mut DiagnosticSink` parameter, or (ii) return `Result<T, SifrDiagnostic>`.

---

## 5. Taxonomy and numbering issues

### T1. 🟠 `SIFR-DECIMAL-2500..2599` is too narrow

The proposed range allocates 100 codes for decimals. The "Decimal Code Migration" table already uses 8 of them. Realistic future scope:

- Per-method scale validation (decimal.round, decimal.quantize, bigdecimal.round, …) — likely 10+ codes.
- Per-context arithmetic (mixed bigdecimal/int, decimal/bigint, …) — 5+ codes.
- Construction-site narrowing (literal vs. variable, hex/bin/oct, …) — 5+ codes.
- BigInt overlap (bigint already shows up in `bigint_int_mixed_arithmetic.sifr`, `bigint_overflow_conversion.sifr`) — 5+ codes.

Realistic ceiling is ~50, comfortably inside 100, but BigInt may want its own family (or `SIFR-BIGINT-26xx`). Conversely:

### T2. 🟠 `SIFR-STDLIB-5200..5999` is too generous, and ownership is unspecified

800 codes for stdlib is implausible. More importantly, the proposal does not say *who owns sub-ranges*. Stdlib has many modules (`io_json`, `sys_fs`, `crypto_regex_uuid`, `argparse`, `bytes`, `subprocess`, `time`, …). Without a sub-range allocation policy, implementers will collide.

**Concrete amendment X19:** define a sub-range allocation policy. Suggested: each stdlib module gets a contiguous 50-code range, registered in `crates/sifr_diagnostics/src/codes.rs` next to the stdlib module table. Reduce the family ceiling to `5200..5599` and reserve `5600..5999` for future stdlib expansion.

### T3. 🟠 Family table has overlaps that should be made explicit

`SIFR-CALL-2600..2899` (arity, keyword) overlaps with `SIFR-PROTO-4200..4499` (callable contracts) and `SIFR-CLASS-4500..4899` (constructor arity). The proposal's "design principle — code identifies user-facing kind" is right, but in practice:

- "Wrong number of args to `sqrt()`" (SIFR-CALL) vs. "Wrong number of args to `__init__`" (SIFR-CLASS) is a real ambiguity.
- "Iterator protocol violation" (SIFR-PROTO) vs. "missing `__iter__` method" (SIFR-CLASS or SIFR-PROTO?) is ambiguous.

Without disambiguation rules, the same surface failure will be assigned different codes by different implementers.

**Concrete amendment X20:** for each family pair with potential overlap, add a one-line "ownership rule" (e.g., "all callable arity errors are SIFR-CALL regardless of whether the callable is a free function, method, or constructor").

### T4. 🟡 Numbering convention is implicit

`SIFR-PARSE-0001..0999` starts at 0001; `SIFR-NAME-1000..1499` starts at 1000. Inconsistent.

**Concrete amendment X21:** state the convention (e.g., "family starts at the family base; first code is `<base>` + 1; `<base>` itself is reserved as `<family>-NNNN-base` not for active use") and apply uniformly.

### T5. 🟡 Reserved-code policy needs a registry shape

Guardrail says "Every active registry code must have fixture coverage or be explicitly marked reserved." But the registry shape is unspecified. Suggested fields per code:

```rust
DiagnosticCode {
    id: &'static str,        // "SIFR-NAME-1001"
    family: Family,
    summary: &'static str,   // 1-line user-facing summary
    state: CodeState,        // Active | Reserved | Retired
    docs_path: &'static str, // "docs/errors/SIFR-NAME-1001.md"
    fixture: &'static str,   // path to a representative fixture, or "" for Reserved
}
```

**Concrete amendment X22:** add the registry record shape to milestone_diag_2 and require a unit test asserting that for every `state == Active` row, `fixture` exists and `docs_path` exists.

---

## 6. Rollout, docs, and tooling

### R1. 🟠 Registry source-of-truth and sync mechanism are undefined

The proposal references three artifacts:

- `internal_docs/diagnostic_codes.md`
- `docs/errors/diagnostic-codes.md`
- `crates/sifr_diagnostics/src/codes.rs`

It says "Registry and code constants cannot silently diverge." How is this enforced? Three options:

- **(a) Code is source-of-truth, docs are generated.** Best. A `cargo run --bin gen-error-docs` regenerates both `.md` files from the registry. Diff is checked in CI.
- **(b) Docs are source-of-truth, code is generated.** Worse — doc-driven build pipelines are fragile.
- **(c) All three are hand-maintained, with a script that asserts they agree.** Brittle.

**Concrete amendment X23:** declare option (a). Add the generator binary as part of milestone_diag_2 DoD. CI runs `cargo run --bin gen-error-docs && git diff --exit-code` to fail on drift.

### R2. 🟠 No JSON Schema for `SifrDiagnostic`

`schemars` is in workspace deps; the proposal mentions LSP/tooling as a goal but does not require a JSON Schema. Without one:

- LSP integrators inspect the Rust source.
- Backwards-incompatible field additions are not caught.
- The "lossless JSON" claim is unverifiable.

**Concrete amendment X24:** milestone_diag_1 requires a checked-in `crates/sifr_diagnostics/schema/sifr_diagnostic.schema.json`, regenerated from `#[derive(JsonSchema)]` and asserted in CI.

### R3. 🟡 No mention of editor/LSP fixture format

If diagnostic codes are stable and structured, IDE tooling will want a deterministic JSON format. The proposal says "JSON, human, and compact render from the same canonical diagnostics" — good — but doesn't lock the JSON envelope (top-level shape, version field, error-array shape). Without a versioned envelope, IDE consumers cannot detect when the schema evolves.

**Concrete amendment X25:** add a top-level JSON envelope `{ "version": 1, "diagnostics": [...] }` and bump `version` on incompatible schema changes. Asserted in milestone_diag_1.

### R4. 🟡 "Fixture or explicitly marked reserved" needs a fixture-coverage guardrail

Today fixtures live in `crates/sifr/tests/e2e/fail/*.sifr`. The guardrail "Every active registry code must have fixture coverage" requires:

- A way to find the code in fixtures (e.g., grep for `# expect-error: SIFR-NAME-1001`).
- A guardrail script (Python or Rust) that walks the registry and checks each Active code is referenced by ≥1 fixture.

**Concrete amendment X26:** add `scripts/check_diagnostic_code_coverage.py` (or extend an existing guardrail) and wire it into `scripts/run_all_tests.sh`.

### R5. 🟡 Architecture and roadmap docs not just "updated" — versioned

The proposal lists `internal_docs/architecture.md` under "Required Documentation Updates" but does not say *what* changes. Concrete required edits:

- Architecture.md:711 — update `E####`/`W####` claim to `SIFR-FAMILY-NNNN`.
- Architecture.md:225 — clarify `sifr_frontend` is a planned crate, distinct from the new `sifr_diagnostics`.
- Roadmap.md Phase 27 row — change status from "completed" to "completed (amended by ad-hoc semantic diagnostic taxonomy)" or reopen.
- Phase 27 doc — note that milestone_27_4 was incomplete and was completed by this ad-hoc phase.

**Concrete amendment X27:** replace the bullet list under "Required Documentation Updates" with a per-file table specifying *what* edit each file receives.

### R6. 🟡 No explicit position in the workflow gate

`AGENTS.md` and `.cursor/skills/project-workflow/SKILL.md` define the required milestone-by-milestone PR workflow. The proposal does not say where this ad-hoc phase slots — before/after which numbered phase. This matters because the project workflow requires "Move to the next item" in order.

**Concrete amendment X28:** add an "Ordering vs. Phase Plan" section. Recommendation: this work is a remediation that should land before any further phase work touches diagnostics (Phase 28+ semantics, Phase 36 tooling), and the merge of milestone_diag_8 reopens-and-recloses Phase 27.

---

## 7. Hard Rules — gaps and additions

The "Hard Rules" list is good but missing several rules implied by the proposal text:

- **HR-add-1:** No public diagnostic type may be defined outside `crates/sifr_diagnostics`. (Prevents a new ad-hoc `LoweringDiagnostic` from drifting out of `SifrDiagnostic`.)
- **HR-add-2:** No diagnostic helper may be added without a registry entry in the same PR. (Prevents drift between code and registry.)
- **HR-add-3:** No `expect-error:` annotation in a fixture may carry a code not present in the registry. (Enforces the registry as the single source of truth for fixture authors.)
- **HR-add-4:** No diagnostic emitter may use `String::from`/`format!` to construct a code; codes are always `DiagnosticCode` values. (Prevents typos.)
- **HR-add-5:** Severity::Error never appears as a child severity. (D3 above.)
- **HR-add-6:** `Option<TextRange>` is not used for HIR/parser source diagnostics. (D1 above.)

**Concrete amendment X29:** append these to the "Hard Rules" list.

---

## 8. Phase Definition of Done — gaps

The PDoD is good but missing:

- The `sifr_diagnostics` crate is a workspace member with isolated dependencies.
- `sifr_type_system` emits `SifrDiagnostic` directly (or, alternatively, `TypeError` is fully retired).
- The `workspace_diagnostic_code` string-prefix classifier is deleted.
- The fixture-grammar parser (`is_message_error_code`, `diagnostic_error_code`) is updated/retired.
- A JSON Schema for `SifrDiagnostic` is checked in.
- A CI check enforces registry/docs sync.
- A CI check enforces fixture coverage of active codes.
- Phase 27 status in `roadmap.md` reflects the amendment.

**Concrete amendment X30:** extend the PDoD bullet list with the above.

---

## 9. Concrete amendments — consolidated

The following edits should be made directly to `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`. Numbers correspond to the X-tags above.

| # | Section | Edit |
|---|---|---|
| X1 | milestone_diag_8 | Add explicit scope item: "Update `crates/sifr/tests/e2e.rs` fixture grammar parser: remove `is_message_error_code` and `diagnostic_error_code`; tighten `is_diagnostic_code` to validate against the registry." |
| X2 | After "Proposed Diagnostic Families" | Add "Existing Code Renumbering" subsection with a per-code table covering `SIFR-PARSE-0001`, `SIFR-CODEGEN-0001`, `SIFR-BUILD-0001`, `SIFR-WORKSPACE-0001..0103`. |
| X3 | New section before "Hard Rules" | Add "Stability" section defining pre-1.0 vs. post-1.0 stability and the trigger event. |
| X4 | milestone_diag_2 | Add "produce a checked-in inventory of every `ctx.error(...)` call site mapped to its proposed code" as a DoD bullet. |
| X5 | milestone_diag_1 | Add lossless-JSON definition: round-trip identity, explicit `null` for `Option`, `#[serde(deny_unknown_fields)]`, JSON Schema check. |
| X6 | milestone_diag_2 | Add "Every Active code has a `docs/errors/<CODE>.md` page; reserved codes are exempt." |
| X7 | milestone_diag_1 | Add explicit dependency direction: `sifr_diagnostics` has no sifr-internal deps; `sifr_hir`, `sifr_codegen`, `sifr_driver`, `sifr_type_system`, and the CLI all depend on it. |
| X8 | All migration milestones (4–6) | Each milestone owns its own fixture/baseline updates; milestone_diag_8 contains only new guardrails and residual cleanup. |
| X9 | milestone_diag_5 | Split into 5a (span plumbing) and 5b (typed builders) so spans land before code migration. |
| X10 | milestone_diag_4 | Add `sifr_type_system::check.rs` to scope; replace `[E2503]` and `[E2504]` emissions. |
| X11 | milestone_diag_3 | Split into 3a (renderer consumption) and 3b (phase-mapping deletion); pin 3b after milestone_diag_6. |
| X12 | New "Sequencing" section | Add a dependency graph diagram across milestones. |
| X13 | milestone_diag_1 | Specify HIR/parser diagnostics use a non-`Option` span at the type level. |
| X14 | milestone_diag_1 | Specify `TextRange` (byte offsets) end-to-end; JSON includes both byte offsets and computed line/column. |
| X15 | milestone_diag_1 | Restrict `DiagnosticChild.severity` to Note/Help (introduce `ChildSeverity` enum). |
| X16 | milestone_diag_8 (or new) | Specify `SIFR-INTERNAL-9000..9999` allocation policy and update panic boundaries. |
| X17 | milestone_diag_3b | Mark `CompilePhase` and the phase-derived `Display` impl for deletion, not retention. |
| X18 | New "Type System Integration" subsection | `sifr_type_system` adopts canonical model directly; `TypeError`/`TypeErrorKind` are retired. |
| X19 | "Proposed Diagnostic Families" | Reduce `SIFR-STDLIB-5200..5999` to `5200..5599`, reserve the rest; define per-stdlib-module sub-range allocation. |
| X20 | "Proposed Diagnostic Families" | Add ownership disambiguation rules for overlapping families (CALL vs. CLASS vs. PROTO). |
| X21 | "Proposed Diagnostic Families" | State and apply a uniform numbering convention (family base inclusive vs. exclusive). |
| X22 | milestone_diag_2 | Specify the registry record shape and the active/reserved/retired state machine. |
| X23 | milestone_diag_2 | Declare `crates/sifr_diagnostics/src/codes.rs` source-of-truth; docs are generated; CI asserts no drift. |
| X24 | milestone_diag_1 | Require checked-in JSON Schema generated via `schemars`. |
| X25 | milestone_diag_1 | Specify versioned JSON envelope `{ "version": 1, "diagnostics": [...] }`. |
| X26 | milestone_diag_8 | Add `scripts/check_diagnostic_code_coverage.py` to required guardrails. |
| X27 | "Required Documentation Updates" | Replace bullet list with per-file table of specific edits. |
| X28 | New "Ordering vs. Phase Plan" section | State that this work amends Phase 27 (milestone_27_4) and updates roadmap.md. |
| X29 | "Hard Rules" | Append HR-add-1 through HR-add-6. |
| X30 | "Phase Definition of Done" | Append the eight bullets in §8 above. |

---

## 10. Suggested rewrites of selected paragraphs

### "Target Architecture", "HIR should stop exposing"

Current:

> HIR should stop exposing:
> ```rust
> pub struct LoweringError { pub message: String, pub line: Option<u32>, pub col: Option<u32> }
> ```
> and instead expose structured diagnostics directly:
> ```rust
> pub type LoweringDiagnostic = SifrDiagnostic;
> ```

Suggested: drop `LoweringDiagnostic` entirely. Type aliases hide the canonical type. HIR returns `Vec<SifrDiagnostic>` directly. There is no per-crate diagnostic alias; all crates use `SifrDiagnostic` from `sifr_diagnostics`.

### "Diagnostic Builder API", "If a helper is missing, the implementation should add the helper"

Current text places the responsibility on the implementer in the moment. Suggested: move the helper-coverage gate to milestone_diag_2 as a one-shot inventory (per X4). The end-state rule "no `ctx.error(String)` for user-facing diagnostics" is right; the rule "if a helper is missing, add one" is too informal — it should be replaced with "every emitted code has a typed builder; the registry is the gate."

### "Acceptable only if crate sequencing makes a separate crate impractical: `crates/sifr_frontend/src/diagnostics.rs`"

Drop entirely. `crates/sifr_frontend` does not exist; the proposal should not depend on a phantom crate. The fallback weakens the architectural commitment to a separate `sifr_diagnostics` crate.

---

## 11. Strengths to preserve

To be clear, several aspects of the proposal are already at principal-engineer quality and should be kept:

- **The semantic vs. phase distinction** in "Design Principle" is exactly right.
- **The Non-Goals section** correctly forbids the most common cleanup-time regressions (string→code classifiers, message-embedded codes, compatibility aliases).
- **The decimal sub-table** is concrete and migration-ready (modulo X10).
- **The "Diagnostic Builder API" sketch** with named-constructor helpers is the right shape for both ergonomics and code-search.
- **The hard rule against spanless HIR diagnostics with a known AST node** is correct (modulo enforcement at the type level — D1).
- **The bounded recovery hooks** ("compact grouping uses (severity, code, canonical message, primary file)") preserve the milestone_27_5 contract.

These should remain unchanged in the amended document.

---

## 12. Bottom line

This is fundamentally a **good architectural target** but a **draft-quality plan**. To be production-grade and directly implementable it needs:

1. **Reconciliation with Phase 27** (roadmap, architecture.md, phase doc).
2. **Inclusion of `sifr_type_system`** and the workspace string-classifier in scope.
3. **A typed span model** end-to-end (`TextRange`, non-`Option` for source diagnostics).
4. **Per-milestone fixture/baseline ownership** (instead of deferring all fixture churn to milestone_diag_8).
5. **A registry source-of-truth + sync gate** + JSON Schema + fixture-coverage check.
6. **Explicit ordering/dependency graph** across the eight milestones, with 5 split for span-first and 3 split for renderer-first.
7. **A cleaned numbering and family-overlap rule set** (decimal vs. bigint, call vs. class, stdlib sub-allocation).

Apply amendments X1–X30 and the proposal becomes a concrete, sequenced, defensible plan. As written, it is approximately 70% of the way there: the destination is clear; the path is not yet tight enough to hand to an implementer without follow-up questions on every milestone.

Recommended next step: revise the issue with the X-numbered amendments above, then submit for a second review pass before opening any milestone PR.
