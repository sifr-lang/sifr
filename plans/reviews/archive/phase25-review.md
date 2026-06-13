# Phase 25 Code Review: CFG/Flow Analysis Activation

## Executive Summary

**Status**: Completed and merged (PRs #883-#887)

Phase 25 successfully activates the Control Flow Graph (CFG) as the canonical source for correctness-critical flow facts. The implementation replaces legacy tree-walk fallbacks with CFG-backed queries, enforces validity invariants, integrates with diagnostics, and provides comprehensive regression coverage.

**Verdict**: APPROVED - Implementation is sound, well-tested, and follows the canonical architecture contract.

---

## Architecture Overview

### Ownership Model

The phase establishes a clear ownership boundary:

| Module | Responsibility |
|--------|---------------|
| `sifr_hir::cfg` | Owns CFG truth - builds, validates, and provides query entrypoints |
| `sifr_codegen::hir_analysis::queries` | Consumes CFG via canonical API - does not build its own CFG |
| `sifr_hir::lower` | Uses CFG for unreachable statement detection during lowering |

### Core Data Structures

The CFG implementation in `crates/sifr_hir/src/cfg.rs` defines:

- **CfgBlockId**: Block identifier (usize)
- **CfgBlockLabel**: Entry, Exit, Statement(&'static str), Synthetic
- **CfgTerminator**: Goto, Branch, Return, Raise, Exit
- **CfgBlock**: id, label, top_level_stmt_index, terminator
- **ControlFlowGraph**: blocks, entry, exit, top_level_stmt_nodes
- **FlowFacts**: exit_effect, reachable/unreachable stmt indices, return types

---

## Milestone-by-Milestone Review

### milestone_25_1: CFG Integration Contract

**Objective**: Wire existing CFG subsystem into active compiler analysis flow.

**Implementation**:

1. Added canonical entrypoints in `sifr_hir::cfg`:
   - `build_control_flow_graph(stmts: &[HirStmt]) -> ControlFlowGraph`
   - `flow_facts(stmts: &[HirStmt]) -> FlowFacts`

2. Integrated into lowering (`crates/sifr_hir/src/lower/statements.rs:6-11`):
   ```rust
   if crate::cfg::flow_facts(&result).always_exits() {
       ctx.warn(format!(
           "unreachable statement at block index {index} was ignored"
       ));
   }
   ```

3. Added demo: `demos/m25_1_cfg_integration_contract_demo/main.sifr`

**Verification**:
- Demo runs correctly: outputs `41` and `0` for `pick_value(41)` and `pick_value(None)`
- CFG tests pass: 7 tests in `cfg::tests`
- Full test suite passes

**Strengths**:
- Clear ownership boundary established
- CFG is built during lowering, not lazily
- Immediate fail-fast on CFG construction errors

**Concerns**: None

---

### milestone_25_2: CFG Validity Invariants

**Objective**: Define and enforce CFG invariants with fail-fast validation.

**Implementation** (`cfg.rs:145-214`):

The `validate()` method checks:
1. Non-empty blocks
2. Entry/exit block IDs within bounds
3. Block ID matches index
4. Branch terminators have at least 2 targets
5. All successors are valid block IDs
6. Top-level statement mappings are correct and unique

**Validation at build time** (`cfg.rs:520-528`):
```rust
pub fn build_control_flow_graph(stmts: &[HirStmt]) -> ControlFlowGraph {
    let cfg = builder.finish(root_entry);
    if let Err(err) = cfg.validate() {
        panic!("internal compiler error: invalid control-flow graph: {err}");
    }
    cfg
}
```

**Determinism** (`cfg.rs:108-143`):
- `shape_fingerprint()` generates deterministic string representation
- `reachable_blocks()` uses iterative DFS with stack (consistent ordering)

**Verification**:
- Demo runs correctly: outputs `4` for `compute(8)`
- Determinism test passes: `cfg_repeat_run_matrix_is_deterministic`

**Strengths**:
- Comprehensive invariant checking at CFG construction time
- Clear error messages for invariant violations
- Deterministic CFG shape ensures repeat-run stability

**Concerns**: None

---

### milestone_25_3: Canonical Flow Truth Queries

**Objective**: Replace tree-walk fallback logic with CFG-backed queries.

**Implementation** (`crates/sifr_codegen/src/hir_analysis/queries.rs:50-72`):

All critical flow queries now use CFG:
```rust
pub(crate) fn block_control_flow_effect(stmts: &[HirStmt]) -> ControlFlowEffect {
    ControlFlowEffect::from(cfg::flow_facts(stmts).exit_effect())
}

pub(crate) fn reachable_top_level_stmt_indices(stmts: &[HirStmt]) -> Vec<usize> {
    cfg::flow_facts(stmts).reachable_top_level_stmt_indices().to_vec()
}

pub(crate) fn unreachable_top_level_stmt_indices(stmts: &[HirStmt]) -> Vec<usize> {
    cfg::flow_facts(stmts).unreachable_top_level_stmt_indices().to_vec()
}

pub(crate) fn body_contains_return(stmts: &[HirStmt]) -> bool {
    cfg::flow_facts(stmts).has_reachable_return()
}

pub(crate) fn try_body_has_value_return(stmts: &[HirStmt]) -> bool {
    cfg::flow_facts(stmts).has_reachable_value_return()
}
```

**Verification**:
- Demo runs correctly: outputs `5` and `77` for `classify(True)` and `classify(False)`
- Query tests pass

**Strengths**:
- No fallback code - direct CFG consumption
- Queries are correctness-critical (affect type inference, code generation)
- Consistent canonical truth source

**Concerns**: None

---

### milestone_25_4: Diagnostics and Consumer Integration

**Objective**: Integrate CFG flow facts into lowering/codegen and ensure deterministic diagnostics.

**Implementation**:

1. **Unreachable statement warnings** (`lower/statements.rs:6-11`):
   - Detects unreachable statements after always-exiting code paths
   - Uses CFG reachability for accuracy

2. **Return type inference**:
   - Ignores unreachable return statements when inferring return types
   - Demo shows unreachable `return 'never'` after `return 2` is correctly handled

3. **Deterministic diagnostics**:
   - CFG shape is deterministic
   - No ordering dependencies in diagnostic generation

**Verification**:
- Demo outputs `2` and `3` (correct inference ignoring unreachable return)
- Warning message is deterministic: "unreachable statement at block index 2 was ignored"

**Strengths**:
- Diagnostics are now based on precise CFG analysis
- Deterministic output across check/build/run modes

**Concerns**: None

---

### milestone_25_5: Regression and Determinism Matrix

**Objective**: Comprehensive regression coverage and deterministic repeat-run checks.

**Implementation** (`scripts/run_phase25_cfg_flow_activation_matrix.sh`):

Matrix covers:

| Row | Test |
|------|------|
| `canonical_query_paths` | m25_3 demo runs correctly |
| `diagnostics_consumer_cfg_integration` | m25_4 demo with unreachable warning |
| `matrix_fixture_full_modes` | m25_5 demo in check/build/run/test modes |
| `cfg_shape_and_query_repeat_determinism` | CFG shape deterministic across rebuilds |
| `negative_reachable_type_error_parity` | Identical diagnostics across modes |
| `negative_diagnostic_stability` | Repeat-run diagnostics byte-identical |

**Verification**:
- All 6 matrix rows pass
- Negative case (type error) produces identical diagnostics across modes

**Strengths**:
- Comprehensive coverage of control flow patterns
- Automated regression matrix wired into validation
- Determinism guarantees

**Concerns**: None

---

## Test Coverage Analysis

### Unit Tests in cfg.rs

| Test | Purpose |
|------|---------|
| `flow_facts_reports_always_raises_for_raise_only_branch` | Flow analysis for raise in all branches |
| `flow_facts_marks_trailing_stmt_unreachable_after_return` | Unreachable statement detection |
| `flow_facts_collects_reachable_return_types_only` | Return type filtering |
| `control_flow_graph_validate_accepts_valid_graph` | Positive validation |
| `control_flow_graph_validate_rejects_invalid_edge` | Negative validation |
| `control_flow_graph_shape_is_deterministic_across_rebuilds` | Determinism |
| `cfg_repeat_run_matrix_is_deterministic` | Extended determinism matrix |

### Unit Tests in queries.rs

| Test | Purpose |
|------|---------|
| `block_control_flow_effect_reports_always_returns_for_exhaustive_if` | Exhaustive if detection |
| `block_control_flow_effect_reports_fallthrough_for_non_exhaustive_if` | Fallthrough detection |
| `block_control_flow_effect_reports_always_exits_for_mixed_return_raise` | Mixed exit paths |
| `reachable_stmt_indices_omit_unreachable_tail_after_return` | Reachability |
| `body_contains_return_ignores_unreachable_return` | Unreachable return filtering |
| `try_body_has_value_return_ignores_unreachable_value_return` | Try body analysis |

---

## Correctness Analysis

### CFG Construction Correctness

The CFG builder handles all HIR statement types:

- **Control flow**: If, While, For, Match, TryExcept, With
- **Transfers**: Return, Raise, Break, Continue
- **Statements**: Let, Assign, Expr, etc.

**Correctness verification**:
- Comprehensive `validate()` catches structural errors
- All 39 tests in sifr_hir pass
- Demo scenarios execute correctly

### Flow Analysis Correctness

Flow exit effect computation (`cfg.rs:565-574`):

```rust
let exit_effect = if falls_through {
    FlowExitEffect::FallsThrough
} else if has_reachable_return && !has_reachable_raise {
    FlowExitEffect::AlwaysReturns
} else if has_reachable_raise && !has_reachable_return {
    FlowExitEffect::AlwaysRaises
} else {
    FlowExitEffect::AlwaysExits
};
```

This correctly handles:
- Exhaustive if with returns in all branches
- Mixed return/raise in try/except
- Unreachable code after early exits

### Reachability Correctness

The reachability algorithm (`cfg.rs:91-106`) uses iterative DFS:

```rust
pub fn reachable_blocks(&self) -> Vec<bool> {
    let mut reachable = vec![false; self.blocks.len()];
    let mut stack = vec![self.entry];
    while let Some(block_id) = stack.pop() {
        if reachable[block_id] { continue; }
        reachable[block_id] = true;
        for &next in self.blocks[block_id].terminator.successors().iter().rev() {
            if !reachable[next] { stack.push(next); }
        }
    }
    reachable
}
```

The `.rev()` ensures consistent ordering across iterations, contributing to determinism.

---

## Determinism Verification

### CFG Shape Determinism

- Block IDs assigned sequentially during construction
- Terminator successors processed in reverse for consistent stack behavior
- `shape_fingerprint()` provides verifiable deterministic representation

### Diagnostic Determinism

The matrix validates:
- `negative_reachable_type_error_parity`: check/build/run produce identical errors
- `negative_diagnostic_stability`: repeated runs produce byte-identical diagnostics

---

## Regression Coverage

### Control Flow Patterns Covered

| Pattern | Demo Coverage |
|---------|---------------|
| Nested branching | m25_1: if inside if |
| Loop exits | m25_2: for with break/continue |
| Early return/raise | m25_3: try/except with return/raise |
| Unreachable tails | m25_4: return after return |
| Complex combinations | m25_5: nested loops + try/except + unreachable |

---

## Quality Contract Compliance

| Requirement | Status |
|-------------|--------|
| No fallback/migration code | ✅ Direct CFG implementation |
| Root cause resolution | ✅ CFG is canonical truth source |
| Production-grade code | ✅ Strict typing, explicit invariants |
| Comprehensive validation | ✅ Positive + negative test cases |
| Deterministic behavior | ✅ Verified by matrix |
| Clear ownership | ✅ sifr_hir owns CFG, sifr_codegen consumes |

---

## Findings and Recommendations

### Strengths

1. **Clean Architecture**: Clear separation between CFG ownership (sifr_hir) and consumption (sifr_codegen)
2. **Fail-Fast Validation**: CFG invariants validated at construction time
3. **Comprehensive Testing**: Unit tests + integration demos + regression matrix
4. **Deterministic Behavior**: Verified across rebuilds and execution modes
5. **No Legacy Code**: Direct implementation without fallbacks

### Potential Improvements (Future Work)

1. **Lazy CFG Construction**: Currently CFG is rebuilt on every query call. Could cache for repeated queries on same statements.
2. **CFG Visualization**: Debugging tool to dump CFG structure for complex functions.
3. **Extended Reachability**: Could add dataflow analysis (live variables) on top of CFG.

### Minor Observations

1. The CFG builder creates synthetic blocks for elif conditions (`CfgBlockLabel::Synthetic`). This is correct but could be documented.
2. `shape_fingerprint()` includes type information which could vary if types are resolved differently across compilation phases. Currently this is mitigated by constructing CFG after type resolution.

---

## Conclusion

Phase 25 is a well-executed implementation that successfully activates CFG-backed control flow analysis. The implementation:

- ✅ Correctly builds CFG for all HIR statement types
- ✅ Enforces validity invariants with fail-fast validation
- ✅ Provides canonical flow truth queries
- ✅ Integrates with diagnostics for unreachable code detection
- ✅ Maintains deterministic behavior across modes and runs
- ✅ Has comprehensive regression coverage

**Recommendation**: APPROVED for production use.

---

## Appendix: Validation Evidence

### Test Results

```
$ bash scripts/run_phase25_cfg_flow_activation_matrix.sh
Phase 25 CFG/flow activation regression matrix: PASS

$ cargo test -q -p sifr_hir cfg::tests
test result: ok. 7 passed; 0 failed

$ cargo test -q -p sifr_hir
test result: ok. 39 passed; 0 failed
```

### Demo Outputs

| Demo | Expected Output | Actual Output |
|------|-----------------|---------------|
| m25_1 | 41, 0 | 41, 0 ✅ |
| m25_2 | 4 | 4 ✅ |
| m25_3 | 5, 77 | 5, 77 ✅ |
| m25_4 | 2, 3 | 2, 3 ✅ |
| m25_5 | 8, 42, 9 | 8, 42, 9 ✅ |

### PR Links

- Part 1: https://github.com/sifr-lang/sifr/pull/883
- Part 2: https://github.com/sifr-lang/sifr/pull/884
- Part 3: https://github.com/sifr-lang/sifr/pull/885
- Part 4: https://github.com/sifr-lang/sifr/pull/886
- Part 5: https://github.com/sifr-lang/sifr/pull/887
