# Phase 43: Interoperability

> Note: Needs more planning before execution (interop model, safety contract depth, and release gating are still draft-level).

## Objective
Deliver interoperability capabilities after typed model, package, tooling, and web foundations are stable.

## Depends on
- Phase 42

## Milestones

### milestone_43_1: Interoperability (FFI)
- Scope:
  - Rust/C FFI boundary model, safety constraints, and diagnostics.
  - Interop notes must build on the extended bytes-foundation contract (locked by `wave_psp_bytes_5`):
    - `bytes` is the canonical owned immutable read-only byte buffer,
    - mutable/output byte-buffer interop remains deferred until explicit mutable/view semantics exist,
    - fixed-width integer families are an explicit interoperability design topic and must not be assumed to exist implicitly because raw-byte-backed `bytes` exists.
- Definition of done:
  - Interop workflows are documented, test-covered, and safe-gated.

## Quality Contract
- Entry criteria: Phase 42 is completed and existing quality gates remain green.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Interoperability is stable and governed by existing quality gates.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_43_1` (Interoperability (FFI)): validation goals cover: Rust/C FFI boundary model, safety constraints, and diagnostics. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Interoperability is stable and governed by existing quality gates.

## Exit Gate
- Interoperability is stable and governed by existing quality gates.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
