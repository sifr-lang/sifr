# Review 2: Semantic Diagnostic Code Taxonomy — Per-Family Namespace, Strictness Pass

Reviewer: agent
Date: 2026-04-29
Source: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Repo state: branch `main`, head `c891177b`, proposal modified-uncommitted
Prior reviews:
- [reviews/semantic-diagnostic-code-taxonomy-proposal-review.md](semantic-diagnostic-code-taxonomy-proposal-review.md)
- [reviews/semantic-diagnostic-code-taxonomy-proposal-review-2.md](semantic-diagnostic-code-taxonomy-proposal-review-2.md)
- [reviews/semantic-diagnostic-code-taxonomy-proposal-review-3.md](semantic-diagnostic-code-taxonomy-proposal-review-3.md)
- [reviews/semantic-diagnostic-code-taxonomy-family-namespace-review.md](semantic-diagnostic-code-taxonomy-family-namespace-review.md)

Lens: principal-engineer / compiler architecture. Strictness pass under the hard constraints stated in the task (no fallback, no compatibility layer, no global numeric range consumed, no `SIFR-TYPE-0001` catch-all).

Severity: 🔴 blocker · 🟠 must-fix · 🟡 should-fix · 🟢 polish.

---

## Verdict: READY WITH MINOR REVISIONS

Prior round findings (F1–F5) are correctly addressed in the current text:
- `milestone_diag_6` scope now reads `SIFR-DECIMAL-000x` per the migration table (line 578) ✅
- `SIFR-INTERNAL-0001` replaces `SIFR-INTERNAL-9001` (line 824) ✅
- Stdlib sub-range example is now `0100..0149` (50 codes), matching policy (line 140) ✅
- `milestone_diag_2a` wording uses "namespaces / per-family numbering convention / reserved family bases" (lines 478, 496) ✅
- `impl From<TypeError> for SifrDiagnostic` typo fixed (line 373) ✅

The per-family namespace design itself is sound, internally consistent on the substantive sections, and elegant. The tax of squeezing 17+ families into a global 10,000-slot pool is gone, and the full-string-as-identity rule cleanly removes the global-range constraint.

What remains is a small set of **strictness gaps** — soft-fallback wording that survives in non-decimal corners, two milestone ownership ambiguities, one sequencing tension, and a handful of clarity items. None block implementation, but several would, if left alone, allow drift that contradicts the proposal's own "no catch-all, no fallback" principle once code starts landing.

| # | Severity | Location | Issue |
|---|---|---|---|
| G1 | 🟠 | line 160 (renumbering table, `SIFR-PARSE-0001`) | "May remain if it is a real parser bucket" preserves a soft parser catch-all by name — directly parallels the retired `SIFR-TYPE-0001` policy but applies opposite treatment without justification |
| G2 | 🟠 | section header line 537, body lines 540–551, sequencing lines 719, 724 | `milestone_diag_4` is one section but is referenced as two milestones (`diag_4a`, `diag_4b`) in sequencing and as a dependency for `diag_5`. Scope items (parser/workspace/build/codegen migration, `CompileError` conversion) aren't assigned to either sub-step |
| G3 | 🟠 | no milestone owns it | Parser diagnostic refinement (introducing structured `SIFR-PARSE-*` codes beyond `0001`) has no milestone. As written, parser stays at the soft catch-all forever |
| G4 | 🟡 | line 164 (renumbering table, `SIFR-WORKSPACE-0001..0103`) | Same "may remain if … after registry review; otherwise retire" soft-fallback pattern as G1; review outcome isn't owned by a milestone |
| G5 | 🟡 | lines 561–567 (`milestone_diag_5`) and lines 575–588 (`milestone_diag_6`) sequencing | `diag_5` rejects bare `[Edddd]` expectations *before* `diag_6` rewires decimal emission. Fixtures asserting `[E2507]` directly need a clearly-defined transitional state; current wording is ambiguous |
| G6 | 🟡 | milestone numbering: `diag_2a → diag_3 → diag_2b` | Out-of-order numeric tags (`2a, 3, 2b`) are a documentation smell and confuse the table of contents and PR titles. Either renumber linearly or keep but explain |
| G7 | 🟡 | none — gap | Neither the `LoweringError → LoweringOutcome` retirement nor the `TypeError`/`TypeErrorKind` retirement nor the `CompileError` → `SifrDiagnostic` conversion is owned by a specific milestone. Phase DoD references them but no milestone scope claims them |
| G8 | 🟡 | line 839 (recovery cap) | The 50-cap applies to `reveal_type` notes but the user-visible behavior at the cap is unspecified — silent drop, truncation summary, or note demotion? Surprising at the boundary |
| G9 | 🟡 | line 460 vs line 245 | Top-level `Severity::Help` is permitted by the canonical `Severity` enum but no use case is described. Either remove it from top-level Severity (allow only as `ChildSeverity`) or document when a top-level Help diagnostic is emitted |
| G10 | 🟡 | line 766–770 validation block | Validation plan lists individual focused commands but does not invoke `scripts/check_diagnostic_docs_sync.py` (introduced in `diag_2a`, line 484) or the `gen-error-docs --check` form needed to detect drift locally |
| G11 | 🟡 | line 129 + family table | New family creation policy mentions "introducing a new namespace in the registry" but no rules constrain family-name shape (uppercase, ASCII, no abbrev rule, length cap) or family retirement state |
| G12 | 🟢 | line 172 (URL policy) | URL form `https://sifr.sh/docs/errors/<CODE>` — case sensitivity unspecified. URLs and on-disk filenames typically standardize on a single case |
| G13 | 🟢 | lines 218–223 (`args` field) | `BTreeMap<String, serde_json::Value>` permits arbitrary nested JSON in template args. Render correctness assumes scalars; nested structures aren't templatable. Should restrict, or document that nested args are inspection-only |
| G14 | 🟢 | line 273 (template syntax) | "Named braces such as `{expected}`" + escape rule is the entire spec. No formatting specifiers, no positional, no nesting — should be made explicit so generators can rely on it |
| G15 | 🟢 | line 142–148 (family ownership rules) | Workspace ↔ Import boundary unaddressed: e.g., "module not found because workspace cannot resolve path" — `SIFR-IMPORT-*` or `SIFR-WORKSPACE-*`? |

No 🔴 findings.

---

## 1. Substantive findings

### G1. 🟠 `SIFR-PARSE-0001` "may remain" preserves a soft parser catch-all

**Location:** line 160, existing-code-renumbering table.

> `SIFR-PARSE-0001` | May remain if it is a real parser bucket with a registry entry; split later if parser can distinguish precise parse categories.

The proposal is explicit and correct that `SIFR-TYPE-0001` is "retired as a public catch-all and never reused" (line 161). The same row pattern for `SIFR-PARSE-0001` (one row up) takes the opposite stance: keep it as a "real parser bucket" until the parser can be made more granular. That is a soft catch-all by name. The hedge "split later" with no milestone behind it means the split need never happen.

This contradicts the strict design principle on line 67 — "`SIFR-TYPE-0001` must not remain a general semantic fallback" — generalized to **all** families, which is the spirit of this phase. The user's brief is unambiguous: "no fallback compatibility path, no old historical compatibility, and no catch-all `SIFR-TYPE-0001` for new semantic diagnostics." The user's example is `SIFR-TYPE-0001`, but the rule is "no catch-all" — `SIFR-PARSE-0001` as currently described meets every functional definition of "catch-all."

The hard rules at lines 802–820 list seven concrete prohibitions, including line 808 "Do not add generic fallback diagnostics for user errors." A `SIFR-PARSE-*` bucket that lumps every parse failure under one code is a generic fallback for user errors.

**Why this matters now, not later:** "split later" without an owning milestone is exactly the failure mode the rest of the proposal forbids — a deferred catch-all that becomes load-bearing the moment downstream tooling, docs, and tests start depending on it.

**Suggested resolution (pick one):**

1. Treat `SIFR-PARSE-0001` symmetrically with `SIFR-TYPE-0001`: retire the catch-all, start active codes at `SIFR-PARSE-0002` with specific meaning, and require `milestone_diag_3` (inventory) to enumerate distinct parser conditions from the Ruff-fork's existing diagnostic surface (which already classifies syntax errors).
2. Pin a precise meaning to `SIFR-PARSE-0001`: e.g., "Ruff parse error with no upstream classification — used only when the parser produces an opaque error and never used as a default for parser conditions that have an upstream subcategory." Then it is a specific code, not a catch-all. Add a registry entry stating that constraint and a guardrail forbidding emission when a more specific parser code applies.
3. Schedule parser refinement explicitly (see G3 below) and convert this row's "may remain" into "remains as a transitional bucket only through `milestone_diag_X`; is split or retired in `milestone_diag_X`."

Recommended: option 2 (precise meaning + guardrail) for the registry, plus option 3 (own the refinement in a milestone) for the work itself. Together they remove the soft-fallback wording entirely.

**Suggested wording change (table row):**

> `SIFR-PARSE-0001` | Reserved meaning: opaque parser error with no upstream classification. Must not be used when a more specific parser condition is detectable. Guardrailed against use as a default emission code; structured `SIFR-PARSE-*` codes are introduced for distinct parser conditions identified in the inventory.

### G2. 🟠 `milestone_diag_4` is one section but referenced as two milestones

**Location:** section header line 537 ("milestone_diag_4: Renderer and Driver Integration"), scope body lines 540–551, sequencing diagram lines 719 and 724.

`milestone_diag_4` is presented as a single section with a `diag_4a` / `diag_4b` split inside its scope bullet. But:

- The Mermaid diagram (lines 719, 724) treats `diag_4a` and `diag_4b` as separate nodes with `diag_5..diag_8` running between them.
- `milestone_diag_5` Scope (line 562) implicitly depends on `diag_4a` having landed (renderers consuming `SifrDiagnostic`).
- `milestone_diag_5` would be mid-sequence between two halves of a single milestone if the section structure is taken literally.

This is inconsistent presentation. More importantly, the scope items inside `diag_4` body — "Migrate non-HIR emission surfaces still using phase-derived codes, including parser adapters, workspace/project discovery, build/materialization/rustc diagnostics, codegen boundaries, and test-runner diagnostics" (line 549) — are not assigned to either `4a` or `4b`. Likewise: the `CompileError` → structured-wrapper conversion (line 547), workspace message-prefix-inference removal (line 548), `CompilePhase` retirement (line 548), and the single Definition of Done block (lines 552–555) are not split between the two sub-milestones.

Result: an implementer reading `milestone_diag_4` cannot tell whether (e.g.) parser-adapter migration is a `4a` deliverable (must land before `diag_5`) or a `4b` deliverable (lands after `diag_8`). For PR sequencing this matters.

**Suggested fix:** Split the section into two top-level milestone subsections matching the Mermaid diagram. Each gets its own Scope and DoD. Concretely:

```text
### milestone_diag_4a: Renderer Integration
Scope:
  - Renderers (human, compact, JSON) consume SifrDiagnostic.
  - Parser adapter, workspace/project discovery, codegen, build, test-runner
    surfaces that still produce phase-derived codes are migrated through the
    inventory's transitional adapter, with each remaining call site tracked.
  - Compact grouping uses (severity, code, message_template, primary file).
  - Workspace message-prefix code inference (CompileError::workspace_diagnostic_code)
    is deleted.
DoD:
  - All renderers operate on SifrDiagnostic exclusively.
  - No renderer or driver code parses messages to recover codes.
  - JSON, human, compact render from the same canonical model.

### milestone_diag_4b: Phase-Mapping Retirement
Scope:
  - Delete CompilePhase::TypeCheck => "SIFR-TYPE-0001" and the rest of the
    phase-derived public diagnostic mapping.
  - Retire CompilePhase as a public Display source for diagnostics.
  - Remove transitional sifr_driver re-exports of sifr_diagnostics types.
  - Convert CompileError into either a structured diagnostic wrapper or an
    internal boundary error already carrying SifrDiagnostic.
DoD:
  - "CompilePhase::TypeCheck => SIFR-TYPE-0001" is gone.
  - sifr_driver no longer re-exports diagnostic types.
  - CompileError is not a public code source.
```

This also makes the Sequencing graph and section structure agree.

### G3. 🟠 No milestone owns parser diagnostic refinement

**Location:** missing.

Even with G1 resolved, the structured parser diagnostic work is unowned. Searching the milestones:

- `diag_4a` migrates the parser adapter to consume `SifrDiagnostic` (transport-level).
- `diag_5` cleans up the e2e harness (test contract).
- `diag_7` lists "Name, Import, Type, and Call Diagnostics" — parser is not in this list.
- `diag_8` lists "Ownership, Flow, Match, Class, Protocol, Result, and Stdlib" — parser is not in this list.
- `diag_11` is guardrails and baselines.

So no milestone introduces structured `SIFR-PARSE-*` codes beyond whatever the inventory in `diag_3` records. The Ruff fork already classifies syntax errors internally; the work is to map those upstream classifications to distinct `SIFR-PARSE-*` codes with stable docs entries.

**Suggested fix:** Either add a `milestone_diag_7.5` (parser categorization) before or alongside `diag_7`, or extend `diag_7`'s scope to "Parser, Name, Import, Type, and Call Diagnostics" and add the corresponding deliverables:

```text
- Map upstream Ruff-fork parser error categories to distinct SIFR-PARSE-* codes.
- Replace the SIFR-PARSE-0001 transitional bucket with category-specific codes
  for the conditions identified in milestone_diag_3.
- Each SIFR-PARSE-* code has a registry entry, docs page, and fixture.
```

If parser refinement is genuinely deferred to a later phase, say so explicitly: state in the proposal which post-phase work item picks it up, and add a guardrail that forbids new `SIFR-PARSE-0001` emission sites added in the meantime. As-is, the silence permits permanent drift.

### G4. 🟡 Workspace `0001..0103` row has the same soft-fallback pattern as G1

**Location:** line 164.

> `SIFR-WORKSPACE-0001..0103` | May remain if each code describes a precise workspace rule after registry review; otherwise retire and replace within the `SIFR-WORKSPACE-*` namespace.

This is similar to G1 but less acute — workspace codes are at least *specific* by hypothesis (each describes a precise rule). The issue is that the "registry review" outcome isn't owned by a milestone. `milestone_diag_2b` (registry population) is the natural home but it doesn't explicitly call out reviewing each existing workspace code's continued validity.

**Suggested fix:** Add to `milestone_diag_2b` Scope:

> - Review each existing `SIFR-WORKSPACE-0001..0103` code against the diagnostic identity policy. Mark any that fails the policy as retired and replace with a precise code in the same family.

DoD addition:

> - Every existing workspace code has either an active registry entry with a precise rule and docs page, or is marked retired with the replacement code recorded.

### G5. 🟡 `diag_5` / `diag_6` sequencing tension on decimal pseudo-code fixtures

**Location:** `milestone_diag_5` lines 561–567, `milestone_diag_6` lines 575–588.

`milestone_diag_5` says (line 562): "Update e2e expectation parsing to accept only canonical `SIFR-<FAMILY>-dddd` codes." It then notes (line 566): "Audit current fail fixtures for bare `[Edddd]` primary expectations before landing this milestone. If any remain, migrate those specific fixtures in the same PR so the suite stays green until decimal migration replaces the old pseudo-codes."

This is ambiguous. "Migrate those specific fixtures" can mean either:

1. **Update the fixture to assert the current top-level code (`SIFR-TYPE-0001`)**, since that's what the emitter still produces for decimal — as a transitional state until `diag_6` rewires emission. This works but momentarily creates `SIFR-TYPE-0001` expectations that the phase DoD (line 848) explicitly forbids in the end state.
2. **Update the fixture to assert the new code (`SIFR-DECIMAL-0007`) and migrate the emission site in the same PR**, partially overlapping `diag_6` work.

Either is workable but the proposal needs to commit. Option 1 avoids splitting `diag_6`'s deliverable but leaves transitional `SIFR-TYPE-0001` expectations in fixtures during the gap; option 2 keeps the no-catch-all rule clean throughout but partially bleeds `diag_6` into `diag_5`.

**Suggested fix (option 2 is stricter):** Reorder `diag_5` and `diag_6` so that `diag_6` lands first, *then* `diag_5` cleans up the expectation grammar. This removes the audit/transition entirely:

```text
diag_4a → diag_6 (decimal migration) → diag_5 (test harness contract cleanup)
       → diag_7 → diag_8 → diag_4b → diag_9 → diag_10 → diag_11
```

After `diag_6`, no fixture asserts `[E25xx]` because both the emission and the expectations have moved. `diag_5` then enforces the new grammar globally with no carve-outs needed. Update the Sequencing diagram and the milestone bodies to match.

If the order must stay as written, replace line 566 with explicit option-1 wording:

> If any fixtures assert bare `[Edddd]` primary expectations, migrate those fixtures in the same PR to assert the current top-level emission code as a transitional state. These transitional assertions are removed in `milestone_diag_6` when the emission migrates. The transitional state is bounded to `milestone_diag_5..milestone_diag_6` and is enforced by the inventory.

### G6. 🟡 Milestone numbering `diag_2a → diag_3 → diag_2b` is confusing

**Location:** milestone headers throughout, sequencing diagram lines 717–718.

The intent is clear (skeleton first, then inventory, then population) but the numbering reads as a mistake on first scan. `diag_2a` and `diag_2b` belong together by name but `diag_3` is interleaved between them. PR titles, validation logs, and roadmap entries will inherit this oddity.

**Suggested fix (clean):** Renumber linearly and let names carry the meaning:

```text
diag_1: Shared diagnostic model
diag_2: Diagnostic registry skeleton
diag_3: Diagnostic emission inventory
diag_4: Diagnostic registry population
diag_5: Renderer integration (was diag_4a)
diag_6: Test harness contract cleanup (was diag_5)
diag_7: Decimal migration
diag_8: Name/import/type/call migration
diag_9: Ownership/flow/match/class/protocol/result/stdlib migration
diag_10: Phase-mapping retirement (was diag_4b)
diag_11: Span completion
diag_12: Recovery semantics
diag_13: Guardrails and baselines
```

Cost: one rename pass. Benefit: monotonic ordering everywhere. Worth it before any code lands; expensive after.

### G7. 🟡 Three architectural retirements lack milestone ownership

**Location:** none — gap.

The proposal calls for retiring three concrete types, but no milestone scope claims them:

1. **`HIR LoweringError { message, line, col }`** (line 287, line 700). Phase DoD references it but no milestone scope says "retire `LoweringError`."
2. **`sifr_type_system::TypeError` and `TypeErrorKind`** (line 366–367, line 860). Phase DoD says "retired or fully replaced" but no milestone scope claims the retirement.
3. **`CompileError` → `SifrDiagnostic` conversion** (line 546–547). In `diag_4` body but unassigned to `4a` or `4b` (see G2).

These are non-trivial transitions touching many call sites. Leaving ownership implicit invites the work to get spread across migration milestones in an uncoordinated way.

**Suggested fix:** Add explicit Scope and DoD bullets:

- `milestone_diag_1` Scope: "Add the canonical `LoweringOutcome` and `DiagnosticSink` types alongside the existing `LoweringError` (which becomes private and is removed in `milestone_diag_4a`)."
- `milestone_diag_4a` Scope: "Replace `LoweringError` with `LoweringOutcome` in HIR and codegen call sites."
- `milestone_diag_7` Scope: "Retire `sifr_type_system::TypeError` and `TypeErrorKind`. The short-lived adapter described in the Type System Integration section is deleted in this milestone." (Alternatively `diag_8` if type-system errors are dominant in ownership/protocol/stdlib paths — pick one home.)
- `milestone_diag_4b` Scope (per G2 split): "Convert `CompileError` into a structured diagnostic wrapper or boundary error."

### G8. 🟡 `reveal_type` cap behavior is unspecified at the boundary

**Location:** line 839.

> The 50 top-level recovery cap applies to all top-level diagnostics after severity ordering, while the existing user-error exit behavior remains based on whether any top-level diagnostic has `Severity::Error`.

This means a file with 60 `reveal_type(...)` calls produces 50 visible notes and 10 silently dropped notes. That is surprising — `reveal_type` is an explicit user request to see information, and silently dropping the last 10 will look like a compiler bug.

**Suggested fix:** Define the cap-overflow behavior:

- Append a single synthesized note: `note: 10 additional reveal_type results omitted (recovery cap)`, attributed to the last consumed diagnostic or the file root.
- Or: count `Severity::Note` separately from `Severity::Error|Warning` against the cap, with the same overflow-summary appended for each stream.
- Or: exempt `reveal_type` notes from the cap entirely. Document the rationale.

Add a fixture covering 51+ `reveal_type` calls so the chosen behavior is locked.

### G9. 🟡 Top-level `Severity::Help` has no described use case

**Location:** line 460 (Severity = `Error | Warning | Note | Help`) vs line 245 (`ChildSeverity = Note | Help`).

`Help` is a meaningful child severity (a hint attached to a parent diagnostic). As a top-level severity it has no use case in the proposal. A standalone "help: did you mean ..." with no parent error is unusual for a compiler.

**Suggested fix:** Pick one:

1. Drop `Help` from the canonical `Severity` enum. Top-level diagnostics are `Error | Warning | Note`. `Help` survives as `ChildSeverity` only.
2. Document the use case (e.g., LSP code-actions emitting help-only diagnostics) and add a fixture.

Option 1 is recommended unless a real use case exists; the smaller surface is harder to misuse.

### G10. 🟡 Validation plan does not invoke docs sync or registry checks

**Location:** lines 766–770.

The focused-checks block is the developer's iteration loop. It runs the parser and renderer tests but doesn't run the registry/docs guardrails introduced in `milestone_diag_2a` (`scripts/check_diagnostic_docs_sync.py`) or `milestone_diag_11` (`scripts/check_diagnostic_code_coverage.py`). The Phase DoD relies on both. Drift can creep in without triggering local feedback.

**Suggested fix:** Append to the focused-checks block:

```bash
cargo run -p sifr_diagnostics --bin gen-error-docs -- --check
python3 scripts/check_diagnostic_docs_sync.py
python3 scripts/check_diagnostic_code_coverage.py
```

Mirror these into `scripts/run_all_tests.sh` so CI matches local validation (per AGENTS.md "CI mirrors these exact scripts — no CI-only behavior").

### G11. 🟡 New family creation lacks shape/retirement rules

**Location:** line 129 + family table.

> New families are added by introducing a new `SIFR-<FAMILY>-*` namespace in the registry. This does not require finding unused space in a global `0000..9999` range.

No constraints on family shape (uppercase ASCII, length ≤ N, no abbreviations except in this allowlist, etc.) and no policy on family retirement. The registry's `Active | Reserved | Retired` states (line 497) apply to codes; family-level state is unspecified.

**Suggested fix:** Add a short paragraph:

> Family names are uppercase ASCII letters, 3–12 characters, no digits, no abbreviations beyond the established allowlist (`PARSE`, `NAME`, `IMPORT`, `TYPE`, `DECIMAL`, `CALL`, `OWN`, `FLOW`, `MATCH`, `PROTO`, `CLASS`, `RESULT`, `STDLIB`, `WORKSPACE`, `CODEGEN`, `BUILD`, `INTERNAL`). New families are introduced by a registry PR that adds the family entry, the reserved `0000` base, and at least one active code with fixture. Families are retired by marking every code in the family `Retired`; retired families remain in the registry as documentation. A family is never reused for a different domain.

### G12. 🟢 Documentation URL case sensitivity

**Location:** line 172.

> `https://sifr.sh/docs/errors/<CODE>`

Should `<CODE>` be `SIFR-NAME-0001` (canonical form) or `sifr-name-0001` (URL-conventional lowercase)? Pick one and state it. Filenames `docs/errors/<CODE>.md` (line 499) inherit the same choice. Cross-platform filesystem case-insensitivity (macOS) and URL handling pressure suggest lowercase, but matching the diagnostic identity exactly suggests upper.

**Suggested fix:** State explicitly:

> Documentation URLs and filenames use the canonical uppercase code form, e.g., `https://sifr.sh/docs/errors/SIFR-NAME-0001` and `docs/errors/SIFR-NAME-0001.md`. The URL is case-sensitive; the filesystem case must match the canonical code form even on case-insensitive filesystems.

### G13. 🟢 Template `args` permits arbitrary nested JSON

**Location:** lines 218–223.

```rust
pub args: BTreeMap<String, serde_json::Value>,
```

`serde_json::Value` allows nested objects and arrays. The template renderer (named-brace substitution per line 273) is implicitly scalar-only. Mixing the two without a stated policy produces either render-time errors or silent stringification.

**Suggested fix:** Either:

1. Restrict `args` to a scalar-only type: `BTreeMap<String, ArgValue>` where `ArgValue = String | i64 | u64 | f64 | bool`.
2. Document that scalar args participate in `message_template` substitution and non-scalar args are inspection-only (rendered as `{name}` placeholders that the human/compact renderers leave unsubstituted, with full JSON value preserved in JSON output).

Option 1 is cleaner; option 2 is more flexible. Either is fine; pick one.

### G14. 🟢 Template syntax spec is one sentence

**Location:** line 273.

> `message_template` uses named braces such as `{expected}` and `{actual}`. Literal braces are escaped as `{{` and `}}`.

That's the entire spec. Question: are formatting specifiers (`{expected:>10}`, `{actual:?}`) supported? Are repeated names (`{name} ... {name}`) allowed? Is whitespace inside braces tolerated (`{ expected }`)? Without explicit answers, generators (docs, compact renderer, recovery deduplication keys) may diverge.

**Suggested fix:** Add:

> Template syntax: a placeholder is `{<name>}` where `<name>` is `[a-z][a-z0-9_]*`. No formatting specifiers, no positional placeholders, no whitespace inside braces. A name may appear multiple times. Literal `{` and `}` are escaped as `{{` and `}}`. Templates are validated against `args` keys at registry load.

### G15. 🟢 Workspace ↔ Import family ownership boundary

**Location:** lines 142–148.

The rules cover Call/Class/Proto/Type/Stdlib overlaps but not Workspace/Import. "Module not found" can be either:

- An import-form failure (`SIFR-IMPORT-*`) — bad import shape.
- A workspace resolution failure (`SIFR-WORKSPACE-*`) — module exists in source but workspace can't locate it.

These are user-visible as similar messages. Without an ownership rule, the registry will likely end up with codes in both families that overlap.

**Suggested fix:** Add to the family ownership rules:

> Module resolution diagnostics use `SIFR-IMPORT-*` when the failure is about the import statement form, the imported symbol, or the import policy. They use `SIFR-WORKSPACE-*` when the failure is about workspace/project layout, module graph construction, or filesystem discovery.

---

## 2. Things that look fine — explicit no-action

For traceability, the following were checked and found correctly aligned with the per-family namespace model and the strict no-fallback principle.

- **Decimal Code Migration table** (lines 184–190) — internally consistent, F1 from prior review is fixed.
- **Initial Code Examples table** (lines 736–758) — all local `000x` codes, internally consistent.
- **Numbering convention** (lines 135–140) — `0000` reserved per family, first active usually `0001`, retired codes remain as registry gaps. F2/F3 from prior review fixed.
- **Family ownership rules for overlaps** (lines 142–148) — sound except for the Workspace/Import boundary noted in G15.
- **Hard Rules** (lines 802–820) — comprehensive and consistent with the strict design principle. The internal-allocation policy (line 822–826) is correctly aligned to the per-family scheme.
- **Stability Policy** (line 798) — correctly distinguishes pre-1.0 and post-Phase-39 stability.
- **Documentation URL Policy** (line 172) — works because full code is globally unique. Only case sensitivity is unstated (G12).
- **Source Mapping Architecture** (lines 408–443) — `SourceId`/`SourceSpan` design is sound, fabricated-span prohibition is correctly aligned with `SIFR-INTERNAL-*` policy.
- **Span Policy** (lines 391–402) — internally consistent, correctly forbids spanless source diagnostics where AST has range.
- **JSON envelope and schema requirements** (lines 274–283) — versioned envelope is the only schema version; correct.
- **Severity::Error|Warning|Note (top-level) and ChildSeverity** — correctly partitioned except for top-level `Help` (G9).
- **Required Documentation Updates table** (lines 786–794) — comprehensive.
- **Phase Definition of Done** (lines 846–867) — comprehensive and aligned with the milestones (modulo the ownership gaps in G7).
- **Risk Register** (lines 870–879) — captures the right risks; the strictness of this proposal materially mitigates each.
- **`SIFR-INTERNAL-0001` as the catch-all** (line 824) — F2 from prior review is fixed; aligned with first-active-is-`0001` convention.
- **Stdlib sub-range example** (line 140) — F3 from prior review is fixed; matches 50-code policy.
- **`milestone_diag_2a` wording** (lines 478, 496) — F4 from prior review is fixed; uses "namespaces" and "per-family numbering convention."
- **`impl From<TypeError> for SifrDiagnostic` typo** (line 373) — F5 from prior review is fixed.

---

## 3. Implementation notes (informational, not findings)

These are forward-looking observations for whoever picks up `milestone_diag_1` / `diag_2`. Not blockers for the planning doc.

- **Code identity in Rust.** A typed enum with one variant per active code (with derived `family()` and `suffix()` accessors) makes the Hard Rule "do not construct codes with `format!`" enforceable at the type level rather than via lint. The previous review noted this and it remains the right call. Cost: the enum grows by one variant per active code, mitigated by per-family submodules.
- **Per-family submodules.** `crates/sifr_diagnostics/src/codes/{name,call,type,...}.rs` keeps file size bounded as families approach `9999`, matches AGENTS.md's no-monolithic-files rule, and makes per-family ownership reviews tractable.
- **Reserved family bases as registry rows.** Seventeen families × one reserved `0000` base each = 17 `state: Reserved` rows from `milestone_diag_2a`'s skeleton. Encodes the convention in the registry, not just in prose.
- **Stdlib sub-ranges as registry metadata.** Each `SIFR-STDLIB-*` row carries a `module: "math"` annotation; a top-level `stdlib_subranges` index maps module → range. The docs generator can render a per-module index without hand-writing it.
- **Schema regeneration check.** Use `schemars` (line 458) to derive the JSON schema from the Rust model. CI / local validation runs `cargo run -p sifr_diagnostics --bin gen-error-docs -- --check` and `git diff --exit-code docs/ schemas/` to detect drift.
- **e2e fixture grammar.** Validate fixture-asserted codes against the registry at test-harness load time, not via regex. The grammar `SIFR-<FAMILY>-dddd` (line 561) is registry-driven per Hard Rule line 815. Fail loudly with the unknown code printed and the closest match suggested.
- **Coverage report.** `scripts/check_diagnostic_code_coverage.py` should print which active registry codes have no fixture and exit non-zero. This is the only way the "every active code has fixture coverage" guarantee survives churn.

---

## 4. Recommendation

Address G1–G3 (🟠) before starting `milestone_diag_1`. They are the only items where leaving the wording as-is permits the implementation to drift away from the proposal's own strict principles:

- **G1**: rewrite the `SIFR-PARSE-0001` row to either retire it or pin it to a precise meaning + guardrail.
- **G2**: split `milestone_diag_4` into separate `4a` and `4b` sections with explicit Scope and DoD each.
- **G3**: add parser categorization to `diag_7` scope (or a new `diag_7.5`), or explicitly defer with a stated post-phase owner.

G4–G11 (🟡) should be addressed in the same documentation pass — most are one-line clarifications and one is a renumber (G6). G12–G15 (🟢) are polish that can ride along.

After those edits, the proposal is implementable as written. The per-family namespace design itself is the right call — strictly more elegant than any global-range allocation and fully consistent with the user's hard constraints. The remaining work is tightening the soft-fallback corners and assigning unambiguous milestone ownership for every concrete retirement.

The proposal does not require another design round. It requires one strict editing pass.
