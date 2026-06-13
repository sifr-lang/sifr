# TypeScript-Go Architecture Transfer M8 First-Class Flow Graph

status: M8 merged

M8 adds a first-class HIR flow graph alongside the existing control-flow graph.
The CFG remains responsible for structural reachability and return facts. The
new flow graph records data-flow effects that later cache and query milestones
can fingerprint, trace, and reuse without re-deriving them from ad hoc lowering
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
as `FlowEffect::Narrow` with `narrowed_type = Type::None`; M8 does not introduce
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

The current graph is a companion graph, not a replacement for all narrowing and
ownership state. M8 moves the facts onto a stable graph surface while preserving
the existing lowering semantics; later query/cache milestones can key and reuse
those facts through the graph fingerprint and debug trace.

## Driver Propagation

Project lowering stores a `FlowGraph` for each lowered module and preserves the
main module graph when constructing single-entry build plans. This keeps graph
facts attached to the same HIR snapshot that codegen and analysis inspect.

M8 stops at HIR and driver propagation. Analysis/LSP display of flow graph
fingerprints and traces is intentionally left to the later tracing and debug
status milestones so editor APIs do not expose an unstable graph contract early.

## Validation

M8 focused validation so far:

- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr_lowering flow_graph -- --nocapture`
- `cargo test -p sifr_lowering`
- `cargo test -p sifr_driver`
- `cargo test -p sifr_frontend -p sifr_analysis -p sifr_lsp`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_package_manager_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report
  `target/validation_lane_reports/create-pr.latest.json`, wall time 306.27s,
  advisories: warm wall-time budget exceeded; group skew is high
- Claude reviewer pass 3 -> SATISFIED
- Loop-else follow-up validation: `cargo fmt --check`,
  `cargo test -p sifr_lowering flow_graph -- --nocapture`,
  `cargo clippy -p sifr_lowering -- -D warnings`
- Claude reviewer loop-else follow-up pass 1 -> SATISFIED
- M8 closeout create-pr validation: `scripts/run_all_tests.sh --profile create-pr` ->
  PASS, report `target/validation_lane_reports/create-pr.latest.json`, wall time
  280.58s, advisory: group skew is high
- Claude reviewer closeout pass 1 -> SATISFIED
- M8 loop-else follow-up: `cargo fmt --check`,
  `cargo test -p sifr_lowering flow_graph -- --nocapture`,
  `cargo clippy -p sifr_lowering -- -D warnings`
- Claude reviewer loop-else follow-up pass 1 -> SATISFIED
