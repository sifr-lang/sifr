## Verdict: READY

## Blocking Findings

None.

## Non-Blocking Concerns

- **FlowSummary's handling of `Break`/`Continue` is implicit, not explicit** (`crates/sifr_codegen/src/hir_analysis/queries/queries_impl.rs:208`). `stmt_summary` routes them through the `_ => FlowSummary::fallthrough()` arm. This is functionally equivalent to the prior CFG-based result in the public queries (loop union with the empty `else` always produces `falls_through: true`, matching the conservative CFG model), and all `cfg`/`flow_graph` tests still pass — but the implicit fallthrough hides intent. A brief comment or explicit `HirStmt::Break | HirStmt::Continue => FlowSummary::fallthrough()` arm would document the invariant for future readers.

- **`pub use sifr_ir::hir_nodes::*;` in `crates/sifr_hir/src/hir_nodes.rs`** is a wide re-export retained for sifr_hir's internal lowering modules. Acceptable for M3 backward-compat; M2 (`sifr_lowering` rename) will tighten this. Worth tracking so it doesn't ossify.

- **`LoweringResult` lives in `sifr_ir` but is constructed only by the lowering crate.** The struct itself is pure data and the contract excerpt explicitly lists "public lowering outcome/result data" in `sifr_ir` scope, so this is on-spec. Flagging only because it puts `num-bigint` in `sifr_ir`'s dependency closure — small surface, but worth keeping minimal as IR consumers grow.

- **`scripts/run_all_tests.sh --profile create-pr` warm wall-time advisory** (155.82s) is consistent with the M3 ledger entry. Not a blocker, but `sifr_ir` introduces another compilation unit in the critical path; M3-M4 should keep an eye on whether this drifts further.

- **54 codegen expectation failures pre-existed M2 and were re-verified in M3** with no new failures introduced. The ledger correctly identifies this as the documented baseline and notes M3 query tests pass within that run.

## Review Round Needed

No further round needed for M3 itself. The boundary changes match the contract:

- `sifr_codegen/Cargo.toml`: `sifr_hir` moved to `[dev-dependencies]`; normal deps go through `sifr_ir`.
- `sifr_lint/Cargo.toml`: direct `sifr_hir` dep replaced with `sifr_ir`.
- `cfg::flow_facts` and the CFG/flow-graph builders remain in `sifr_hir`, satisfying the "construction stays in lowering crate" gate.
- `cargo tree` validations on both codegen and lint confirm no direct or transitive (through IR) `sifr_hir` edge on the normal graph.

Proceed to open the M3 PR and then move to `milestone_ir_boundary_2` (rename to `sifr_lowering`), where the `pub use sifr_ir::hir_nodes::*;` shim and the remaining test-only `sifr_hir` dev-dep references can be cleaned up under the new crate name.
