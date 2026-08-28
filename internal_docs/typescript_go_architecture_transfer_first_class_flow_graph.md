# TypeScript-Go Architecture Transfer: Flow-Graph Snapshot

status: retained snapshot artifact

This work adds an HIR flow graph alongside the existing control-flow graph.
The CFG remains responsible for structural reachability and return facts. The
new flow graph records data-flow effects that later cache and query surfaces
can fingerprint, trace, and reuse without re-deriving them from one-off lowering
state.

## Graph Model

`sifr_lowering::flow_graph` defines snapshot-scoped flow nodes, edges, and effects.
Nodes represent entries, exits, statements, conditions, loops, joins, and
unreachable statements. Effects cover definitions, assignments, narrowing,
narrowing clears, moves, move resets, borrows, mutations, calls, exits, joins,
and unreachable paths.

Each `FlowGraph` exposes:

- ordered nodes and edges;
- an entry and exit node id;
- an effect iterator;
- `shape_fingerprint()` for stable snapshot identity;
- `debug_trace()` for reviewable trace output.

Effects use the existing type model directly. A narrowing to `None` is recorded
as `FlowEffect::Narrow` with `narrowed_type = Type::None`; flow graph does not introduce
a parallel `RefinedToNone` vocabulary.

Loop graphs emit a synthetic `Join { label: "loop" }` for the no-`else` exit
frontier. Loops with an `else` body use the else-body frontier as the loop exit
instead, so they do not add an unused loop join node.

`shape_fingerprint()` includes effect payload labels as well as node/edge shape,
so structurally identical graphs with different calls, mutations, or narrowed
types have different fingerprints.

## Lowering Integration

`LowerCtx` now records flow effects when lowering performs narrowing, clears
narrowing, marks ownership moves, resets move state, records borrows, or
invalidates mutation-sensitive facts. `LoweringResult` carries the resulting
module flow graph next to the lowered HIR module.

The graph is a companion graph. It does not replace narrowing or ownership
state. Lowering records the facts on a stable graph surface. Cache and query
code can use the graph fingerprint and debug trace.

## Driver Propagation

Project lowering stores a `FlowGraph` for each lowered module and preserves the
main module graph when constructing single-entry build plans. This keeps graph
facts attached to the same HIR snapshot that codegen and analysis inspect.

This work stops at HIR and frontend propagation. Analysis and LSP code do not
read the graph or show its fingerprint. Editor APIs therefore do not expose the
graph rules.

## M12 Retention Decision

Decision: **keep the graph as a deterministic snapshot artifact**.

The graph has these current consumers:

- `sifr_ir::ControlFlowFacts` retains the graph and exposes its fingerprint and
  debug trace.
- `sifr_lowering` builds statement graphs for CFG facts and builds one module
  graph from the HIR plus recorded lowering effects.
- `sifr_frontend::ProjectCompilation` retains one graph for each compiled
  module.
- The frontend cache identity includes `FLOW_GRAPH_POLICY_VERSION`. A graph
  rules change must therefore invalidate the affected cached product.

The graph has no current analysis or LSP consumer. This absence is explicit.
The decision does not claim future editor use.

Removing the graph would delete deterministic CFG evidence and change the
canonical project compilation product. Refactoring it into a second semantic
authority would duplicate narrowing and ownership rules. The retained companion
model avoids both outcomes. Lowering remains the semantic authority and records
each narrowing effect once through `LowerCtx::narrow_var_with_flow`.

Revisit this decision only if measurements show that project-level graph
retention has material cost, or if a consumer needs graph queries that the
current immutable snapshot cannot supply.

## Validation

Initial flow-graph validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr_lowering flow_graph -- --nocapture`
- `cargo test -p sifr_lowering`
- `cargo test -p sifr_driver`
- `cargo test -p sifr_frontend -p sifr_analysis -p sifr_lsp`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 verification/areas/package_management/tools/check_package_manager_guardrails.py`
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py`
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py --self-test`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report
  `target/validation_lane_reports/create-pr.latest.json`, wall time 306.27s,
  advisories: warm wall-time budget exceeded; group skew is high
- Claude reviewer pass 3 -> SATISFIED
- Loop-else validation: `cargo fmt --check`,
  `cargo test -p sifr_lowering flow_graph -- --nocapture`,
  `cargo clippy -p sifr_lowering -- -D warnings`
- Claude reviewer loop-else pass 1 -> SATISFIED
- flow graph readiness create-pr validation: `scripts/run_all_tests.sh --profile create-pr` ->
  PASS, report `target/validation_lane_reports/create-pr.latest.json`, wall time
  280.58s, advisory: group skew is high
- Claude reviewer readiness pass 1 -> SATISFIED
- flow graph loop-else validation: `cargo fmt --check`,
  `cargo test -p sifr_lowering flow_graph -- --nocapture`,
  `cargo clippy -p sifr_lowering -- -D warnings`
- Claude reviewer loop-else pass 1 -> SATISFIED
