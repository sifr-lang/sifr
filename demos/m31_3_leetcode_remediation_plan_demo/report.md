# Phase 31 Remediation Backlog

## Approval Process

- Owner sign-off: `yaseralnajjar`
- Reviewer sign-off: `external_phase_reviewer`
- Policy: Every backlog item must carry an owner, dependencies, acceptance criteria, and a target phase before it is considered approved for scheduling.

## Stale Blocker Policy

- Threshold: `14` days
- Applies to: `P1`
- Required action: reassign owner or record an explicit defer decision with rationale and target phase

## Ranked Items

### phase31-remediation-008: codegen.mutable_binding_emission
- Type: `bug`
- Priority: `P1`
- Effort: `small`
- Owner: `compiler/codegen`
- Bucket case count: `1`
- Dependencies: `[]`
- Wave candidate: `True`
- Proposed target phase: `phase31`
- Rationale: Small, isolated codegen bug with immediate end-to-end payoff and low regression radius.
- Acceptance criteria:
  - Tuple-destructured mutable locals emit `mut` where reassignment is required.
  - The `0069_sqrtx` repro runs end-to-end without Rust mutability errors.

### phase31-remediation-002: lowering.destructuring_target_support
- Type: `spec_gap`
- Priority: `P1`
- Effort: `medium`
- Owner: `compiler/lowering`
- Bucket case count: `7`
- Dependencies: `[]`
- Wave candidate: `True`
- Proposed target phase: `phase31`
- Rationale: High-frequency blocker across graph, heap, and string problems with low conceptual dependency depth.
- Acceptance criteria:
  - Tuple/loop destructuring targets used by the corpus lower deterministically.
  - Graph and heap corpus repros for the bucket move past the prior lowering failure.

### phase31-remediation-003: frontend.nested_function_annotation_support
- Type: `spec_gap`
- Priority: `P1`
- Effort: `medium`
- Owner: `compiler/frontend`
- Bucket case count: `6`
- Dependencies: `['lowering.unsupported_ast_shape']`
- Wave candidate: `False`
- Proposed target phase: `phase32`
- Rationale: Nested helper definitions block several backtracking and divide-and-conquer fixtures but should follow AST-shape enablement.
- Acceptance criteria:
  - Nested helper parameters and returns no longer require manual annotations in the covered corpus patterns.
  - Classifier bucket count drops for nested helper-driven failures without regressing diagnostics.

### phase31-remediation-004: stdlib.python_module_surface
- Type: `spec_gap`
- Priority: `P1`
- Effort: `medium`
- Owner: `compiler/stdlib`
- Bucket case count: `6`
- Dependencies: `[]`
- Wave candidate: `True`
- Proposed target phase: `phase31`
- Rationale: Set/defaultdict/deque/heapq parity gaps block multiple unrelated topics and are straightforward to measure.
- Acceptance criteria:
  - Corpus cases using `set`, `defaultdict`, `deque`, `heapq`, and equivalent module aliases no longer fail at symbol resolution.
  - Stdlib parity coverage is added for every newly exposed API surface.

### phase31-remediation-005: type_system.recursive_node_forward_reference
- Type: `spec_gap`
- Priority: `P1`
- Effort: `medium`
- Owner: `compiler/type_system`
- Bucket case count: `4`
- Dependencies: `[]`
- Wave candidate: `False`
- Proposed target phase: `phase32`
- Rationale: Tree problems remain structurally blocked until recursive node references resolve predictably in signatures.
- Acceptance criteria:
  - TreeNode/ListNode forward references resolve without manual reordering in covered corpus patterns.
  - Tree-focused corpus cases advance past unknown-type failures.

### phase31-remediation-001: type_system.optional_narrowing_and_union_ops
- Type: `spec_gap`
- Priority: `P1`
- Effort: `large`
- Owner: `compiler/type_system`
- Bucket case count: `16`
- Dependencies: `[]`
- Wave candidate: `True`
- Proposed target phase: `phase31`
- Rationale: Largest bucket in the seed baseline with multi-topic impact and direct unblock potential.
- Acceptance criteria:
  - Optional/union arithmetic, indexing, and return-value paths used by the seed corpus type-check successfully.
  - New regression coverage locks the repaired operator and narrowing behaviors.

### phase31-remediation-012: stdlib.python_builtin_signature_surface
- Type: `spec_gap`
- Priority: `P2`
- Effort: `small`
- Owner: `compiler/stdlib`
- Bucket case count: `1`
- Dependencies: `[]`
- Wave candidate: `False`
- Proposed target phase: `phase32`
- Rationale: Builtin signature parity issues are low-volume but easy to close after the higher-leverage buckets land.
- Acceptance criteria:
  - Builtin signatures used by the corpus match documented Sifr semantics or are explicitly documented as divergences.
  - The `2235_add_two_integers` repro leaves the builtin-signature bucket.

### phase31-remediation-006: frontend.generic_check_failure
- Type: `bug`
- Priority: `P2`
- Effort: `medium`
- Owner: `compiler/frontend`
- Bucket case count: `3`
- Dependencies: `[]`
- Wave candidate: `False`
- Proposed target phase: `phase32`
- Rationale: Residual generic failures need concrete decomposition after the dominant bucket fixes land.
- Acceptance criteria:
  - Each case in the generic bucket is either reclassified into a specific bucket or fixed.
  - The generic bucket reaches zero before phase closeout.

### phase31-remediation-007: codegen.generic_run_failure
- Type: `bug`
- Priority: `P2`
- Effort: `medium`
- Owner: `compiler/codegen`
- Bucket case count: `1`
- Dependencies: `[]`
- Wave candidate: `True`
- Proposed target phase: `phase31`
- Rationale: Run-stage Rust build failures should be decomposed quickly because they already pass frontend validation.
- Acceptance criteria:
  - The seed run no longer contains generic run failures; remaining run failures map to specific buckets.
  - Rust build diagnostics for the repro case are covered by a regression test.

### phase31-remediation-009: lowering.attribute_expression_support
- Type: `spec_gap`
- Priority: `P2`
- Effort: `medium`
- Owner: `compiler/lowering`
- Bucket case count: `1`
- Dependencies: `['type_system.recursive_node_forward_reference']`
- Wave candidate: `False`
- Proposed target phase: `phase32`
- Rationale: Tree field access remains blocked after type resolution and should be sequenced behind recursive node support.
- Acceptance criteria:
  - Attribute reads on supported class/recursive-node values lower without expression-shape rejection.
  - Tree corpus cases advance beyond attribute-expression diagnostics.

### phase31-remediation-010: lowering.unsupported_ast_shape
- Type: `spec_gap`
- Priority: `P2`
- Effort: `medium`
- Owner: `compiler/lowering`
- Bucket case count: `1`
- Dependencies: `[]`
- Wave candidate: `False`
- Proposed target phase: `phase32`
- Rationale: Nested function definitions and related AST shapes should be enabled before deeper nested-helper inference work.
- Acceptance criteria:
  - Previously unsupported nested function statement shapes lower into HIR with stable diagnostics.
  - The `0052_n_queens_ii` repro no longer fails with `unsupported statement type`.

### phase31-remediation-011: ownership.borrowed_return_surface
- Type: `intentional_divergence`
- Priority: `P3`
- Effort: `small`
- Owner: `language/ownership`
- Bucket case count: `1`
- Dependencies: `[]`
- Wave candidate: `False`
- Proposed target phase: `deferred`
- Rationale: Borrow-by-default return semantics may remain intentionally explicit rather than silently cloning or changing ownership rules.
- Acceptance criteria:
  - Phase documentation explicitly records the divergence and the required `own`/`.clone()` escape hatch.
  - No remediation work starts unless product direction changes the ownership contract.
