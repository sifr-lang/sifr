# Phase 40: Typed Data Model and Validation (Pydantic-Parity Track)

> Note: Needs more planning before execution (which pydantic subset to target, scope boundaries, parity target depth, and acceptance gates are still draft-level).

## Objective
Introduce a dedicated typed model layer with serialization and validation semantics, stable error behavior, and explicit pydantic-parity boundaries.

## Depends on
- Phase 39

**Scope note:** This phase owns the typed serialization surface (`Serialize`/`Deserialize`, `dumps`/`loads`, and typed JSON roundtrips). Phase 41 is the downstream consumer via typed extractors, not a dependency of this phase.

## Milestones

### milestone_40_1: Typed Model Core and Serialization
- Scope:
  - Class-to-model mapping with field metadata and defaults.
  - Auto-derive `Serialize`/`Deserialize` for model-backed classes without manual annotation.
  - Optional/union/list/dict model handling.
  - Baseline serialization/deserialization (`dumps`/`loads`) with typed JSON roundtrips independent of web extractors.
- Definition of done:
  - Typed model core is usable independent of async/web runtime concerns.
  - `dumps(obj)` serializes model-backed classes to JSON strings.
  - `loads(s, T)` deserializes JSON strings to typed models and returns `Result[T, JSONDecodeError]`.
  - Nested models, lists, dicts, optionals, and unions serialize and deserialize correctly.

### milestone_40_2: Validation Engine
- Scope:
  - Strict vs coercion modes.
  - Nested model validation and collection constraints.
  - Field/model validator hooks with deterministic order.
- Definition of done:
  - Validation behavior is deterministic, testable, and documented.

### milestone_40_3: Error Model and Diagnostics Contract
- Scope:
  - Structured validation errors (path, code, message, context).
  - Stable parse/validation error-code contract.
- Definition of done:
  - Validation failures produce stable, structured, and actionable errors.

### milestone_40_4: Parity and Compatibility Matrix
- Scope:
  - Feature matrix per capability: `parity`, `intentional-diff`, `unsupported`.
  - Port representative pydantic behavior tests.
- Definition of done:
  - Target pydantic subset is explicit and regression-locked.

## Quality Contract
- Entry criteria: Phase 39 is completed and release governance is active.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Typed model + validation layer is stable, test-covered, and consumable by web extractors without redesign.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_40_1` (Typed Model Core and Serialization): validation goals cover: Class-to-model mapping with field metadata and defaults; Auto-derive `Serialize`/`Deserialize`; Optional/union/list/dict model handling; Baseline serialization/deserialization (`dumps`/`loads`) and typed JSON roundtrip behavior. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_40_2` (Validation Engine): validation goals cover: Strict vs coercion modes; Nested model validation and collection constraints; Field/model validator hooks with deterministic order. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_40_3` (Error Model and Diagnostics Contract): validation goals cover: Structured validation errors (path, code, message, context); Stable parse/validation error-code contract. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_40_4` (Parity and Compatibility Matrix): validation goals cover: Feature matrix per capability: `parity`, `intentional-diff`, `unsupported`; Port representative pydantic behavior tests. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Typed model + validation layer is stable, test-covered, and consumable by web extractors without redesign.

## Exit Gate
- Typed model + validation layer is stable, test-covered, and consumable by web extractors without redesign.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
