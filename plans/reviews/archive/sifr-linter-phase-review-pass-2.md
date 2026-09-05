# Ad Hoc Phase Review: Production-Grade Sifr Linter — Pass 2

Date: 2026-05-26
Reviewer: agent phase review
Phase: `issues/ad-hoc-production-grade-sifr-linter.md`
Execution: `issues/ad-hoc-production-grade-sifr-linter-execution.md`
Pass 1 blockers:
- C-1: No `check_linter_reuse_rules.py` validation script
- C-2: No explicit mechanical enforcement that parser-aware suppression blocks syntax/HIR rules

---

## Pass 1 Blocker Verdict

### C-1: `check_linter_reuse_rules.py` — STILL BLOCKING

The phase correctly identifies `verification/tooling/check_linter_reuse_rules.py` as a required M1 deliverable (lines 288–303). M1 scope explicitly calls for it. The validation bullets correctly reference it (lines 302–303).

However: the script **does not exist** in the codebase. `verification/tooling/` contains `check_tooling_dependency_boundaries.py` and `check_rule_suppression_rules.py` — these are different tools and do not cover M1's requirements. Specifically:

- `check_tooling_dependency_boundaries.py` checks for `ty_python_semantic`, `ty_project`, `ruff_server`, `pyright`, `pylsp`. It does **not** check for `ruff_linter::rules`, `ruff_linter::registry`, `ruff_linter::linter`, `ruff_linter::noqa`, Python `Rule` IDs, or `ruff_python_semantic` into `sifr_lint`.
- `check_rule_suppression_rules.py` tests runtime suppression behavior, not Rust Cargo-level dependency boundaries.

The M1 requirement is a **Rust crate boundary check** across Cargo.toml and transit crate scans. The existing tooling covers adjacent concerns but not this one.

**Verdict: C-1 remains a blocker.** The phase correctly describes what M1 needs; implementation has not started M1 yet.

---

### C-2: Mechanical suppression gate enforcement — PARTIALLY ADDRESSED

The pass-1 finding was: "nothing proves M5 syntax/HIR rules fail to compile or test if M3 hasn't merged first."

The phase was updated with:
- Quality contract line 61: "must either depend on the parser-aware suppression API at compile time or fail a dedicated guardrail"
- AC-13: "A mechanical gate prevents syntax, HIR, and workspace lint rules from shipping before parser-aware suppression support is enabled"
- M3 scope line 345: "update the suppression-gate manifest so syntax, HIR, and workspace lint-rule modules are allowed only after this milestone closes"
- M3 validation line 351: "guardrail proving syntax/HIR/workspace rules fail validation if they bypass the parser-aware suppression API"

This is **significantly more specific than before**, but two problems remain:

1. **"Compile-time dependency or dedicated guardrail" is still two options, not one.** The phrase "or fail a dedicated guardrail" keeps the mechanism ambiguous. AC-13 says the gate must be "mechanical" — but a compile-time dependency in Rust means a hard import error, while a "guardrail" could mean a Python script, a CI check, a feature flag, or a test. These have very different enforcement strengths.

2. **The suppression-gate manifest has no implementation path.** M1 requires creating it; M3 requires updating it; but there is no check that the manifest exists, is valid, or actually controls anything. No script in the phase validation plan checks the manifest state before M3 closes.

**Verdict: C-2 is partially addressed but still a blocker.** The intent is sound; the mechanism is underspecified enough to allow implementation that is still "advisory."

---

## New Blocking Issues

### D-1: Phase validation plan omits the new linter-specific checks

**Phase:** Validation Plan (lines 453–468)

The validation plan for phase closure references:
```bash
python3 verification/tooling/check_rule_suppression_rules.py
python3 verification/tooling/check_rule_suppression_rules.py --self-test
python3 verification/tooling/check_tooling_dependency_boundaries.py
```

It does **not** reference:
- `check_linter_reuse_rules.py` — the M1 delivery
- `check_linter_reuse_rules.py --self-test`
- the suppression-gate manifest existence check
- the guardrail proving syntax/HIR workspace bypass fails

Since the phase is planned (not yet implemented), `check_linter_reuse_rules.py` doesn't exist yet — this is expected. But the validation plan must be updated to include it once M1 ships, otherwise the closure gate will run old checks and miss the new dependency boundary enforcement.

**Verdict: D-1 is a blocker for phase closure, not for M1 start.** It must be fixed before phase closure, but does not prevent the phase from starting.

---

### D-2: Suppression-gate manifest structure is undefined

**Phase:** M1 suppression-gate manifest creation (line 295)

The M1 scope says "create a suppression-gate manifest that records whether parser-aware suppression is enabled and which rule families may compile while it is disabled."

Several questions are unanswered in the phase:

1. **File path**: No path is specified (`verification/tooling/linter_manifests/suppression_gate.json`? inline in the execution doc? elsewhere?)
2. **Schema**: The manifest needs at minimum: gate state (open/closed), which rule families are allowed at gate-open, and timestamp/version. No schema is defined.
3. **Validation**: No existing tool checks this manifest. M3 scope says "update the suppression-gate manifest" but M3's validation says "suppression contract checks and self-tests" — it does not say "verify the gate manifest says gate is open/closed."
4. **Gate closing trigger**: What marks M3 as closed? Is it a commit, a version bump in the manifest, a feature flag? The manifest update in M3 (line 345) is described as a side effect, not a mechanical gate.

Without a defined manifest path and a script that verifies its state, the suppression gate cannot be enforced.

**Verdict: D-2 is a critical drafting gap.** The phase describes the manifest but does not define it. This must be resolved before M1 closes.

---

### D-3: M5 non-physical-line rule blocking mechanism incompletely specified

**Phase:** M5 (`sifr_policy_rule_families`) interaction with M3 gate

M1 creates the suppression-gate manifest "which rule families may compile while it is disabled" (line 295).
M3 "updates the suppression-gate manifest so syntax, HIR, and workspace lint-rule modules are allowed only after this milestone closes" (line 345).
M5 adds "representative syntax rules" and "representative HIR/frontend policy rules" (lines 385–386).

The logical chain is: M5 code is blocked from compiling/testing until M3 closes. But **how**?

Options implied by the phase:
a) **Rust compile-time dependency**: syntax/HIR rule modules `use` the parser-aware suppression API; without it they fail to compile.
b) **Manifest-based CI check**: a Python script verifies the gate manifest shows M3 is closed before allowing M5 tests to pass.
c) **Test-level guardrail**: M5 tests include a check that the gate API is present; if absent, tests fail.

The phase says "compile time or dedicated guardrail" (line 61) but does not commit to one. Option (a) is the strongest (compile-time). Option (b) requires a manifest path and a CI runner condition. Option (c) is the weakest (tests can be removed or skipped).

Without committing to one option, implementers will choose the easiest path (e.g., `#[cfg(feature = "parser_aware_suppression")]`) that still allows M5 to ship without M3.

**Verdict: D-3 is a blocker.** The phase must specify exactly **one**enforcement mechanism for the M3→M5 dependency.

---

## Additional Findings (Non-Blocking but Requiring Attention)

### H-1: `unsafe-fixes = "hint"` TOML value is defined but its enforcement semantics are not in the quality controls

**Phase:** Config ownership (lines 192, 215)

The phase introduces `unsafe-fixes = "hint"` semantics (line 215: "surfaced as unavailable/user-confirmation-required suggestions but are not applied automatically"). This is correct and matches the formatter phase pattern.

However, there is no acceptance criterion for unsafe-fix behavior:
- AC-6: "hard vs. policy diagnostic class is present in analysis/LSP"
- But no AC covers: "unsafe fixes are never auto-applied regardless of config value"

The "Fixes" section (lines 247–256) is correct and complete. This should be captured as an AC sub-item or cross-reference.

**Conclusion: not blocking.** The behavior is well-specified in prose. Should be captured explicitly.

---

### H-2: M6a/M6b split is described but there is no explicit minimum-gate/milestone closeout definition

**Phase:** M6 (lines 404–416)

M6 is split into M6a (synchronous fix applicability) and M6b (deferred resolution + workspace edit tracking). This is a good structure. However:

- The phase does not define what "closing M6a" means before M6b starts.
- No acceptance criterion distinguishes M6a-close vs. M6b-close.
- M5 (rule families) is gated on M3; M6 has no analogous milestone split gate.

This could lead to M6a being declared "done" while M6b is deferred indefinitely, or M6b being added as scope creep to M6a.

**Conclusion: not blocking but scope-risk.** M6a/M6b is a good split. An acceptance criterion or explicit scope note distinguishing M6a minimum from M6b enhanced would clarify milestone boundaries.

---

### H-3: Traversal negative tests are mentioned in M2 but not cross-referenced with performance budgets

**Phase:** M2 (line 320)

M2 scope includes "negative fixtures for deep directory traversal, ignored directories, symlink loops or cycles, and pathological file counts within the local validation budget."

The pass-1 review raised `collect_sifr_files_inner` as H-4 (uses `fs::read_dir` without depth/budget limits). M2 correctly scopes the negative fixture requirement. However:

- No performance budget is defined in the phase (e.g., "discovery must complete within X seconds for Y files")
- The negative fixture requirement is qualitative ("within the local validation budget") without a defined budget number
- The guardrail from pass-1 (H-4) is not cited or cross-referenced

The phase is correct to scope the negative fixtures. The budget number is an implementation detail. But if the budget is "unlimited," the negative fixture tests don't constrain the behavior.

**Conclusion: not blocking.** Implementation will define the budget. Cross-reference pass-1 H-4.

---

### H-4: No script verifies the suppression-gate manifest or its state transitions

**Phase:** M3 gate closure

M3 validation says (line 351): "guardrail proving syntax/HIR/workspace rules fail validation if they bypass the parser-aware suppression API."

But there is no script or code that checks this. The guardrail must be:
1. A Python script or Rust compile-time check
2. Running in CI or as a validation command
3. Referenced in the validation plan

None of these are specified.

**Conclusion: not blocking for phase start, but blocking for M3 closure.** Must be specified in M3 scope before M1 closes.

---

## Cross-Check: New Additions Around Pass-1 Concerns

### Unsafe fixes (lines 215, 247–256)
Correct. `unsafe-fixes = "hint"` semantics are well-specified. Hard diagnostics are never fixed. Policy diagnostics follow applicability.

### Import organization boundary (lines 237–243)
Correct. Explicit list of required definitions before any Sifr import-organization rule ships. No isort-style rule can be added to M5.

### Traversal negative tests (line 320)
Correct. Required in M2 scope. Adequately scoped.

### Rule-count threshold (line 389)
Explicit: "exceeds 50" is the trigger. Correct.

### M6a/M6b fix tracks (lines 410–415)
Clear split. M6a is the milestone minimum gate. M6b is enhanced. No structural issues.

---

## Summary

The phase correctly addressed the **intent** of pass-1 blockers C-1 and C-2, adding named guardrails, explicit scope updates, and AC-13. However:

**C-1** (`check_linter_reuse_rules.py`) still has no implementation path. The code does not exist. M1 must create it.

**C-2** (suppression gate enforcement) is partially addressed: the prose is now stronger and more specific. But it remains underspecified in two ways that still allow the gate to be implemented as "advisory":
- Two enforcement options remain ("compile-time dependency or dedicated guardrail") without committing to one
- The suppression-gate manifest has a creation requirement (M1) and an update requirement (M3), but no path, schema, or validation check

**D-2** (suppression-gate manifest structure is undefined) and **D-3** (M5 blocking mechanism is underspecified) are the root causes.

---

## Recommendation: Three Additions to Resolve Blockers

1. **Define the suppression-gate manifest path and schema** in M1 scope:
   - File path: `verification/tooling/linter_manifests/suppression_gate.json`
   - Required fields: `schema`, `gate_state` (`open` | `closed`), `allowed_rule_families`, `updated_by_milestone`
   - M1 creates with `gate_state = "open"` and `allowed_rule_families = ["physical-line"]`
   - M3 updates to `gate_state = "closed"` and adds `syntax`, `HIR`, `workspace`

2. **Specify a single enforcement mechanism** for the M3→M5 dependency:
   - "Compile-time: syntax/HIR/workspace rule modules must `use` the parser-aware suppression API. If absent, the module fails to compile."
   - Remove the "or dedicated guardrail" option. Let the Rust type system be the gate.

3. **Add the manifest/state transition check** to M3 validation:
   - M1 adds a Python script that verifies the gate manifest is parsable
   - M3 validation requires the manifest to show `gate_state = "closed"` before M5 may proceed
   - The same script or an additional one closes the loop

With these three additions, the phase is implementation-ready. The remaining H-1 through H-4 findings are implementation details to address within PRs.

---

**The phase is NOT yet implementation-ready.** Three blocking drafting gaps (D-2, D-3, and the unresolved half of C-2) must be resolved before M1 starts.
