# Optional/None Closure: Full Approach Review Before Next Decision

Date: 2026-03-30  
Checkpoint artifact: `verification/leetcode/full_corpus_current_results_20260329_live_after_optional_wave9e.json`

## Review Question

Given Sifr language perfectness goals, should we continue fixture-heavy closure waves, or switch to compiler-first closure for the remaining failures?

## Current State

- Full corpus summary:
  - `PASS=135`
  - `CHECK_ERROR=252`
  - `RUN_ERROR=24`
- Optional/None phase bucket (remaining from this phase): `61` (`CHECK_ERROR` only).

## Findings (Severity Ordered)

### Critical: `RUN_ERROR` lane is predominantly compiler/runtime-pipeline debt

- Remaining `RUN_ERROR=24` split:
  - `15` with Rust `E0308` type mismatches in generated Rust.
  - `4` codegen panics: `structured statement emission missing for production path`.
  - `2` Rust `E0277` trait/pattern generation gaps.
  - `1` Rust `E0369` operator/type mismatch.
  - `1` Rust `E0425` unresolved symbol (`set`) stdlib surface gap.
  - `1` Rust `E0428` duplicate definition surfaced only at Rust build stage.
- Interpretation:
  - `22/24` are compiler/codegen correctness issues, not fixture semantics issues.
  - Check-pass but run-build-fail violates Sifr’s “if it compiles, it works” contract.

### High: Optional phase bucket majority is arithmetic over Optional-contaminated flows, but mixed

- Remaining phase bucket: `61`.
- Largest cluster: `optional arithmetic/reduction` (`30/61`).
- However, a large subset co-occurs with non-core issues (undefined vars/surface syntax artifacts), so not all `30` should drive compiler-rule expansion directly.

### High: Broad `CHECK_ERROR` population still includes non-Optional language-surface gaps

- Top first diagnostics in remaining `CHECK_ERROR=252` include:
  - attribute expression surface gaps (`.next`, `.left`, `.val`)
  - `Any` indexing/len/iterability propagation failures
  - unsupported assignment shapes (augmented subscript target, tuple target shapes)
  - explicit mutability boundary violations (`mut` missing)
- Interpretation:
  - Optional-phase closure should stay scoped; many failures belong to separate language lanes.

## Reviewer-Aligned Recommendation

### Decision

Adopt a **compiler-first strategy for all remaining `RUN_ERROR`**, then continue phase closure on remaining Optional check failures with strict Sifr-principled triage.

### Why this is the best path for Sifr perfectness

- Run-stage compiler failures indicate core language pipeline breakage and must be fixed in compiler/codegen.
- Fixture rewrites must not be used to hide check→run pipeline defects.
- Optional semantics must remain explicit; no hidden unwrap/coercion should be introduced as a shortcut.

## Execution Guardrails

1. No further fixture rewrites for `RUN_ERROR` unless the fixture is invalid Sifr syntax/contract and the compiler already emits an explicit check-stage diagnostic.
2. Add/strengthen a contract lane: check-pass fixture must rust-compile and run; regressions fail fast.
3. For Optional `CHECK_ERROR`, classify per fixture:
   - valid explicit Sifr program rejected => compiler fix
   - fixture uses unstated/non-Sifr assumptions => canonical fixture rewrite allowed
4. Every wave must include:
   - focused root-cause inventory
   - reviewer pass before implementation
   - targeted + quick + full-corpus validation deltas

## Proposed Next Priority Order

1. `RUN_ERROR` wave-R1: structured statement emission panic closure (`4` fixtures).
2. `RUN_ERROR` wave-R2: generated Rust `E0308` type-invariant closure (`15` fixtures).
3. `RUN_ERROR` wave-R3: trait/pattern/operator/surface closure (`E0277`, `E0369`, `E0425`, `E0428`).
4. Return to Optional `CHECK_ERROR` phase lane with compiler-vs-fixture decision per fixture under the same reviewer gate.
