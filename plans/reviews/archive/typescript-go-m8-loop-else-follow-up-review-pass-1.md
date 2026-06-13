# Sifr M8 First-Class Flow Graph — Loop-Else Join Cleanup Follow-up Review (Pass 1)

## Scope

Diff vs `origin/main` on `wave_tsgo_m8_followup_cleanup`:
- `crates/sifr_hir/src/flow_graph.rs` — move `let join = self.join_node("loop");` from the top of `build_loop` into the no-else branch; add blank line before `#[cfg(test)] mod tests;`.
- `internal_docs/typescript_go_architecture_transfer_m8_first_class_flow_graph.md` — status line bumped to "M8 merged".
- `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md` — M8 row marked merged.

This is a targeted follow-up to the pass-2 actionable item: "`build_loop` (`flow_graph.rs:463-477`) creates a `join` node that is unused when `else_body` is `Some`; harmless but should be removed."

## Findings

### 1. The dead `join` node is correctly removed for the with-else case

In the original code (main), `build_loop` always called `self.join_node("loop")` upfront, but the returned `Vec<FlowNodeId>` from the `if let Some(else_body)` branch discarded the join. The join node was pushed into `self.nodes` with a `FlowEffect::Join` effect, yet had no incoming or outgoing edges — a true dead node, leaking into the `FlowGraph`'s node list, `effects()` iterator, `shape_fingerprint()`, and `debug_trace()`.

The fix (`crates/sifr_hir/src/flow_graph.rs:478-493`) moves the `join_node` call inside the `else` arm, so it is only created when the frontier is consumed. With-else loops now produce a graph with the body and else-body nodes but no synthetic `Join { label: "loop" }` placeholder. The with-else semantics are unchanged: the else body is still built from `node` on `FlowEdgeKind::False`, and its frontier is the loop's exit frontier.

### 2. No-else branch is semantically equivalent but the join node id shifts

In the no-else case, the join is now created after `build_stmt_list(body)` runs, so its `FlowNodeId` is the last index in `self.nodes` rather than an early one. The graph structure (one `Join { label: "loop" }`, one `node -> join` edge of kind `False`, plus the body and its `LoopBack` edges to `node`) is identical to the prior shape. The only observable change is the integer id of the join node.

Because `shape_fingerprint()` (`flow_graph.rs:177-191`) embeds each node's `id` and `kind`, the fingerprint string for any program that contains a no-else loop will change. This is not a determinism violation — the same HIR input still produces the same fingerprint across rebuilds — but it is an intentional fingerprint break that consumers will see.

This is the right tradeoff for a follow-up that fixes a graph-shape bug, but it should be called out because:
- The pass-3 review doc claims "`shape_fingerprint()` includes effect payload labels as well as node/edge shape, so structurally identical graphs with different calls, mutations, or narrowed types have different fingerprints" — by that same logic, a graph-shape fixup is also a fingerprint change. Worth one sentence in the M8 doc.
- No existing test pins the post-cleanup fingerprint, so the cleaner shape is not yet regression-locked.

### 3. Loop semantics for both branches are preserved

Walked the two branches against `build_stmt` (`flow_graph.rs:392-440`) and `build_stmt_list` (`flow_graph.rs:353-390`):

- **With `else_body`**: The `if let Some(else_body) = else_body` branch returns `build_stmt_list(vec![node], FlowEdgeKind::False, else_body, false)` — this is the frontier of the else body, which becomes the new frontier for whatever follows the loop. The new code keeps this exactly as before, minus the dead join. The body's `LoopBack` edges back to `node` are unaffected.
- **Without `else_body`**: The `else` branch creates `join` and adds `node -> join` with `FlowEdgeKind::False`, returning `vec![join]`. The body and its `LoopBack` edges are unchanged. The only delta is construction order.

Both branches correctly return the post-loop frontier that callers in `build_stmt` use to chain subsequent statements.

### 4. Blank line + status doc updates

The added blank line before `#[cfg(test)] mod tests;` resolves the pass-2 nit. `cargo fmt --check` and `cargo clippy --workspace -- -D warnings` both pass.

The M8 doc status bump and tracker row update are consistent with the merged state. The pass-2 review also flagged a duplicate M8 row / duplicate "M8 local validation so far" block in `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md`; this follow-up does not collapse those, but that work is out of scope for the loop-else cleanup.

## Test gaps

The cleanup is not regression-locked. `crates/sifr_hir/src/flow_graph/tests.rs` has no test that builds a `HirStmt::While` / `HirStmt::For` / `HirStmt::AsyncFor`, with or without `else_body`, so nothing in the test suite would have caught the original dead-node bug, and nothing in the test suite confirms the post-cleanup shape.

Recommended additions (small, focused, follow-up scope):

1. A `build_loop_with_else_omits_loop_join` test that lowers a loop with an `else_body` and asserts `debug_trace()` does **not** contain `Join { label: "loop" }` (i.e., no dead join) and that the else body frontier feeds the post-loop frontier. Pairs naturally with the existing `statement_graph_tracks_branches_loops_mutations_and_exits` style.
2. A `build_loop_without_else_emits_single_loop_join` test that lowers a loop without an `else_body` and asserts exactly one `Join { label: "loop" }` node plus one `False` edge from the loop condition node to it. Pins the no-else shape so a future "move join to top" regression is caught.
3. Optionally, an end-to-end lowering assertion (`lower_source` of a `while ... : ... else: ...` program) that calls `result.flow_graph.shape_fingerprint()` and compares against a stored golden value, so the fingerprint break is documented as a one-time event rather than silently re-broken.

## Doc gaps

- The M8 doc (`internal_docs/typescript_go_architecture_transfer_m8_first_class_flow_graph.md`) does not note that the `loop` join is now created lazily only when there is no `else_body`. Worth a one-line clarification under "Graph Model" so consumers reading the graph know to expect a `Join { label: "loop" }` only in the no-else path.
- The "Graph Model" section could also mention that the no-else case is the canonical "loop with a join" — the with-else case's exit frontier is the else body's frontier, with no synthetic join — so downstream fingerprint consumers know the structural difference.
- The pre-existing pass-3 review file does not list this follow-up under validation; either add a closing note here or in a pass-4 review when the regression tests are in.

## Verdict

**SATISFIED** for the focused loop-else cleanup. The fix removes the dead `Join { label: "loop" }` node from the with-else branch, preserves semantics in both branches, and is a strict improvement to the graph shape. Local validation (`cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr_hir flow_graph`, `cargo test -p sifr_hir`, `cargo test -p sifr_driver`) all pass.

The fingerprint for no-else loops changes as a side effect of construction-order reshuffling. This is acceptable for a graph-shape fixup but is not regression-locked — see the test gaps above. None of those gaps are blockers for this follow-up; they should land before the next M8-tracing milestone that consumes `shape_fingerprint()` for real.
