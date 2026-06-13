SATISFIED. The branch is ready for PR.

Evidence:
- `crates/sifr_hir/src/flow_graph.rs` (519 lines) + `flow_graph/effects.rs` (529 lines) + `flow_graph/tests.rs` introduce a snapshot-scoped graph with `FlowNode`, `FlowEdge`, `FlowEffect` (Define/Assign/ClearNarrowing/Narrow/Move/ResetMove/Borrow/Mutation/Call/Exit/Join/Unreachable), plus `shape_fingerprint()` and `debug_trace()` surfaces — all under the 900-line cap.
- `LoweringResult` now carries `flow_graph`; `ProjectLowering` propagates per-module graphs; `entrypoint.rs:323` retrieves the main graph with a panic diagnostic fallback (`INTERNAL_COMPILER_PANIC`).
- `FlowFacts` exposes `flow_graph()`, `flow_graph_fingerprint()`, `flow_graph_debug_trace()` (`cfg.rs:266-275`).
- Lowering effects are recorded through `LowerCtx` helpers (`record_flow_effect`, `narrow_var_with_flow`, `clear_narrowing_with_flow`, `mark_moved_with_flow`, `reset_moved_with_flow`); existing `scope.*` calls are preserved at every site I checked (async_await, defaultdict/empty_collection/generic_receiver refinement, regular_calls, control_flow, patterns_and_assignments, task/task_handle/timeout, tuple_unpack, if_expression), so narrowing/ownership behavior is unchanged.
- `apply_narrowing` (`narrowing.rs:140-162`) now routes narrowing through the flow graph and re-applies the existing `narrow_type` as a fallback.
- Tests in `flow_graph/tests.rs` cover statement-graph branches/loops/mutations/exits and end-to-end lowering effects (`narrow v`, `clear-narrowing v`, `mutate items via method pop`).
- `internal_docs/architecture.md` records the M8 surface and points to `typescript_go_architecture_transfer_m8_first_class_flow_graph.md`.

Minor follow-ups (not blockers):
- `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md` has two M8 table rows and two "M8 local validation so far" blocks; the older draft should be collapsed.
- `flow_graph.rs` is missing a blank line before `#[cfg(test)] mod tests;` (cargo fmt may or may not catch this; worth running locally).
- `build_loop` (`flow_graph.rs:463-477`) creates a `join` node that is unused when `else_body` is `Some`; harmless but should be removed.
- `narrowing.rs` calls `narrowing_effects_for_condition` only to extract a narrowed type; the per-subcondition effects are discarded and `narrow_var_with_flow` records a single effect. Consider recording the full effect vec directly, or removing the helper.
- The two flow-graph tests do not directly assert borrow/move traces; worth a focused test once the next milestone consumes them.
