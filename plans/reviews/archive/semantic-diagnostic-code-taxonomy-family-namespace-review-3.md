# Review 3: Semantic Diagnostic Code Taxonomy — Final Verification After Patch Round

Reviewer: agent
Date: 2026-04-29
Source: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Repo state: branch `main`, head `c891177b`, proposal modified-uncommitted
Prior reviews:
- [reviews/semantic-diagnostic-code-taxonomy-proposal-review.md](semantic-diagnostic-code-taxonomy-proposal-review.md)
- [reviews/semantic-diagnostic-code-taxonomy-proposal-review-2.md](semantic-diagnostic-code-taxonomy-proposal-review-2.md)
- [reviews/semantic-diagnostic-code-taxonomy-proposal-review-3.md](semantic-diagnostic-code-taxonomy-proposal-review-3.md)
- [reviews/semantic-diagnostic-code-taxonomy-family-namespace-review.md](semantic-diagnostic-code-taxonomy-family-namespace-review.md)
- [reviews/semantic-diagnostic-code-taxonomy-family-namespace-review-2.md](semantic-diagnostic-code-taxonomy-family-namespace-review-2.md)

Lens: principal-engineer / compiler architecture. Final verification pass under hard constraints (no fallback compatibility, no historical alias layer, no message-embedded `[Edddd]`, no generic `SIFR-TYPE-0001` catch-all in final state, family-local NNNN identity inside `SIFR-<FAMILY>-NNNN`).

Severity: 🔴 blocker · 🟠 must-fix · 🟡 should-fix · 🟢 polish.

---

## Verdict: READY

Every finding G1–G15 from review-2 is resolved in the current proposal text. No 🔴 or 🟠 issues remain. The plan is implementable as written.

A short acknowledgement table follows so future readers can audit the patch round; no new findings are raised.

---

## 1. Resolution audit (G1–G15 from review-2)

| # | Severity | Status | Where in current proposal |
|---|---|---|---|
| G1 | 🟠 | ✅ Fixed | Line 163: `SIFR-PARSE-0001` is now "Reserved meaning only: opaque parser error with no upstream classification… guardrails must reject it as a default parser emission code." Hard rule line 858 explicitly forbids any `0001` code as a family-default catch-all unless guardrailed. Symmetric with `SIFR-TYPE-0001` retirement. |
| G2 | 🟠 | ✅ Fixed | The previously combined `milestone_diag_4` is now split into `milestone_diag_4a: Renderer Integration` (line 553) and `milestone_diag_4b: Phase-Mapping Retirement` (line 661). Each has its own Scope and DoD. The Mermaid sequencing diagram (line 757) and section headings now agree. |
| G3 | 🟠 | ✅ Fixed | `milestone_diag_7` (line 607) is now retitled "Parser, Name, Import, Type, and Call Diagnostics" and explicitly owns: mapping Ruff-fork parser categories to distinct `SIFR-PARSE-*` codes, replacing broad parser emission with category-specific codes from the inventory, and guardrailing `SIFR-PARSE-0001` against use as a default. DoD line 633 enforces it. |
| G4 | 🟡 | ✅ Fixed | `milestone_diag_2b` Scope (line 543) and DoD (line 551) explicitly own the `SIFR-WORKSPACE-0001..0103` review. Renumbering table (line 167) is also tightened: "Each existing code must be reviewed during registry population." |
| G5 | 🟡 | ✅ Fixed | Sequencing now reorders to `diag_4a → diag_6 → diag_5` (Mermaid line 763, prose line 772). `milestone_diag_5` opens with "This milestone lands after `milestone_diag_6`…" (line 576) and DoD line 589 forbids introducing transitional `SIFR-TYPE-0001` expectations. The `[Edddd]` audit/transition language is gone. |
| G6 | 🟡 | ✅ Fixed | The numbering `diag_2a → diag_3 → diag_2b → diag_4a → diag_6 → diag_5 → diag_7 → diag_8 → diag_4b → diag_9 → diag_10 → diag_11` is non-monotonic, but line 772 now makes the sequencing graph authoritative and explains both the `2a/3/2b` and `6/5` orderings. The review accepted "keep but explain" as a valid resolution. |
| G7 | 🟡 | ✅ Fixed | All three retirements have explicit milestone owners now: `LoweringError` is introduced as transitional plumbing in `milestone_diag_1` (line 474) and removed in `milestone_diag_4a` (line 563); `TypeError`/`TypeErrorKind` are retired in `milestone_diag_7` (line 628) with the short-lived adapter deleted in the same milestone; `CompileError` → structured wrapper conversion is in `milestone_diag_4b` (line 667). |
| G8 | 🟡 | ✅ Fixed | Lines 890–892 specify cap-overflow behavior: "rendering appends one structured `Severity::Note` summary such as `10 additional diagnostics omitted by recovery cap`. For `reveal_type(...)`, the summary must say how many explicit reveal results were omitted rather than silently dropping them." Locked by a `>50 reveal_type` fixture in `milestone_diag_10` (line 717). |
| G9 | 🟡 | ✅ Fixed | `milestone_diag_1` Scope (line 473) defines top-level `Severity` exactly as `Error | Warning | Note`. DoD line 486 forbids constructing top-level `Severity::Help`. Hard rule line 868 reinforces. The Severity enum sketch (line 253) shows only `ChildSeverity = Note | Help`. |
| G10 | 🟡 | ✅ Fixed | Validation focused-checks block (lines 808–817) now invokes `gen-error-docs --check`, `check_diagnostic_docs_sync.py`, and `check_diagnostic_code_coverage.py`. Pre-completion block (lines 821–829) repeats them. Line 831 requires wiring into `scripts/run_all_tests.sh` so CI mirrors local. |
| G11 | 🟡 | ✅ Fixed | Line 131 specifies family-name shape (uppercase ASCII, 3–12 chars, no digits, allowlist of abbreviations) and the registry-PR requirement (entry + reserved `0000` base + ≥1 active code with fixture). Family retirement: "Retired families remain documented in the registry; a retired family is never reused for a different domain." |
| G12 | 🟢 | ✅ Fixed | Line 178: "Documentation URLs and filenames use the canonical uppercase code form, for example `https://sifr.sh/docs/errors/SIFR-NAME-0001` and `docs/errors/SIFR-NAME-0001.md`. The URL is case-sensitive; generated filenames must match canonical code casing even on case-insensitive filesystems." |
| G13 | 🟢 | ✅ Fixed | `args` is now `BTreeMap<String, DiagnosticArg>` where `DiagnosticArg = String \| Signed(i64) \| Unsigned(u64) \| Float(f64) \| Bool(bool)` (lines 222, 245–251). Restricted to scalars; `serde_json::Value` is gone. |
| G14 | 🟢 | ✅ Fixed | Line 285: "Template syntax is intentionally small: a placeholder is `{<name>}` where `<name>` matches `[a-z][a-z0-9_]*`. Formatting specifiers, positional placeholders, nested placeholders, and whitespace inside braces are not supported. A name may appear multiple times. Registry loading validates that every placeholder has a matching scalar `args` key…" |
| G15 | 🟢 | ✅ Fixed | Line 150 adds the Workspace ↔ Import boundary rule explicitly: `SIFR-IMPORT-*` for import statement form, imported symbol selection, or import policy; `SIFR-WORKSPACE-*` for workspace/project layout, module graph construction, package roots, or filesystem discovery. |

No finding is downgraded, deferred, or partially resolved.

---

## 2. Independent re-check against the hard constraints

I re-read the proposal end-to-end against the user's four invariants and confirm:

1. **Global numeric range 1..10,000 is respected.** Identity is the full string `SIFR-<FAMILY>-NNNN`; numeric suffixes are family-local (line 133). Adding new families consumes no global numeric space (line 129). The proposal does not introduce any code that would collide with the existing 1..10,000 range. Existing workspace codes such as `SIFR-WORKSPACE-0101` keep their numbers (line 135) without renumbering for global-range fitness.

2. **No fallback compatibility.** The Non-Goals section (lines 96–102), Hard Rules (lines 853–869), and renumbering table (lines 161–168) collectively prohibit `SIFR-TYPE-0001` compatibility, message-embedded `[E25xx]`, string-prefix-to-code classifiers, compatibility aliases, old baselines, and phase-derived public diagnostic identity. The hard rule on line 858 closes the "any 0001 code as family-default catch-all" hole.

3. **No historical alias layer.** Type System Integration (line 386) explicitly forbids `impl From<TypeError> for SifrDiagnostic` as a long-term design and bounds any short-lived adapter to a single migration PR; `milestone_diag_7` Scope (line 628) requires the adapter's deletion in the same milestone. Hard rule line 866 forbids `expect-error` annotations using codes absent from the registry.

4. **No generic `SIFR-TYPE-0001` catch-all survives the final phase.** Renumbering table line 164 retires it permanently. Phase DoD line 900 confirms "No e2e fail fixture expects `SIFR-TYPE-0001` as a catch-all." `milestone_diag_5` DoD line 589 prevents reintroduction during the test-harness cleanup. `milestone_diag_11` guardrail line 732 enforces it permanently.

All four hard constraints are honored by the plan as written.

---

## 3. Sequencing sanity check

The numeric labels are non-monotonic, but the dependency graph is consistent. Tracing it:

```
diag_1 (shared diagnostic model + LoweringOutcome introduced alongside LoweringError)
  → diag_2a (registry skeleton + docs generator + drift check)
  → diag_3 (emission inventory)
  → diag_2b (registry population from inventory; workspace 0001..0103 review)
  → diag_4a (renderers consume SifrDiagnostic; LoweringError retired in HIR;
             parser/workspace/codegen/build/test-runner transport migration;
             workspace message-prefix inference deleted)
  → diag_6 (decimal pseudo-codes [E25xx] retired; SIFR-DECIMAL-000x emission)
  → diag_5 (e2e harness rejects [Edddd]; registry-validated expectation grammar)
  → diag_7 (parser/name/import/type/call code migration; TypeError retirement)
  → diag_8 (ownership/flow/match/class/protocol/result/stdlib code migration)
  → diag_4b (CompilePhase retired; CompileError → structured wrapper;
             sifr_driver re-exports removed; residual cleanup)
  → diag_9 (span completion + related spans)
  → diag_10 (recovery semantics, error tainting, reveal_type cap fixture)
  → diag_11 (final guardrails + baselines)
```

Verified properties:

- `LoweringError` is private transitional plumbing from `diag_1` (line 474) and is removed from user-facing paths in `diag_4a` (line 563). No milestone between them is starved of HIR diagnostic transport.
- `diag_4a` migrates the renderers and HIR transport before family-specific code migration. During the `diag_4a → diag_6` window, HIR may emit `SifrDiagnostic` values whose code is still `SIFR-TYPE-0001` for not-yet-migrated emission sites; this is an intermediate, not a final, state and is bounded by the migration milestones that follow. The phase DoD applies only at completion, so this is consistent.
- `diag_5` only lands after `diag_6` retires `[E25xx]`, so the harness can reject `[Edddd]` expectations without any transitional fixture state. Line 589 explicitly forbids replacing decimal pseudo-code expectations with `SIFR-TYPE-0001` expectations.
- `diag_4b` lands after the family migrations (`diag_6/5/7/8`) so retiring `CompilePhase` does not strand any emission surface still depending on phase-derived codes. Line 669 confirms "this milestone is residual cleanup only; new family migrations must not be deferred here."
- `diag_9` follows code migration, since spans are populated for diagnostics whose codes are already final.
- `diag_10` and `diag_11` close out recovery semantics and guardrails over the now-stable code identity.

No sequencing impossibility. No milestone depends on a deliverable that lands later in the graph.

---

## 4. Validation guardrail completeness check

The set of guardrails enumerated in `milestone_diag_11` (lines 730–746), the Hard Rules (lines 853–869), and the Phase DoD (lines 898–920) collectively cover:

- No `SIFR-TYPE-0001` catch-all in fixtures or emission.
- No `[Edddd]` message-embedded codes.
- No emission of unregistered codes.
- No raw `ctx.error(String)` user-facing emission.
- No message-prefix code inference.
- No fully-rendered-message grouping (must use `message_template`).
- No spanless source diagnostics where AST has range.
- No `[Edddd]` expectations in e2e fixtures.
- No surviving `is_message_error_code` / `diagnostic_error_code` helpers.
- No emission code constructed via `format!`/raw string at call site.
- No top-level `Severity::Help` or child `Severity::Error`.
- Active codes have fixture coverage and generated docs pages.
- JSON schema is checked in and synchronized with the Rust model.
- Docs and code-coverage drift checks run in `scripts/run_all_tests.sh`.

This set closes the obvious drift channels. No further guardrail is required for the planning doc to be complete.

---

## 5. Items still worth noting (informational, not findings)

These are observations for the implementer of `milestone_diag_1` / `milestone_diag_2a`. They are not gaps in the plan and do not affect READY status.

- **Intermediate-state ergonomics during `diag_4a → diag_6`.** HIR emitting `SifrDiagnostic` values stamped `SIFR-TYPE-0001` is unavoidable for unmigrated families until each family's migration milestone lands. This is fine for the plan (no Phase DoD assertion fires until `diag_11`), but the inventory from `diag_3` should clearly mark which call sites are still using transitional codes between `diag_4a` and the milestone that retires that family. The proposal already says "any still-unmigrated legacy path is explicitly temporary, tracked by the inventory, and blocked from gaining new emission sites" (line 559); a one-line note in `diag_3` DoD that the inventory carries this `transitional-code-until-milestone-X` field would make the bookkeeping explicit. Not blocking.
- **`diag_4a` Scope is large.** Renderers + HIR `LoweringError` retirement + four upstream transport migrations + workspace message-prefix-inference deletion is a lot for one milestone. This is implementation pacing rather than a planning gap. If it turns out to be one PR's worth of work that lands in two passes, that is an internal tactical decision the milestone allows.
- **Active code count vs. enum size.** A typed `DiagnosticCode` enum with one variant per active code (preferred over a stringly-typed identifier per Hard Rule line 866) will grow as families fill. `crates/sifr_diagnostics/src/codes/{parse,name,...}.rs` per-family submodules keep file sizes within AGENTS.md's "no monolithic files" guidance. The proposal does not need to prescribe this; it is an implementer choice that the architecture already permits.
- **e2e harness registry validation.** `milestone_diag_5` DoD line 582 says "Validate fixture-asserted codes against the registry at harness load time, not by regex alone. Unknown codes fail loudly with the unknown code and a closest-match hint." This is the correct enforcement layer, but the implementer should double-check that fixture loading happens before any test setup that could swallow the error message. Not a planning gap.

None of these is a finding.

---

## 6. Recommendation

The proposal is READY for implementation. Begin `milestone_diag_1` directly.

The per-family-namespace design is sound, the hard constraints are honored end-to-end, every prior finding is resolved, and the milestone graph contains no sequencing impossibility. The remaining items are informational notes for the implementer, not gaps the planning doc needs to address.

No further design or planning round is required.
