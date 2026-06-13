Reviewed the diff and ran targeted validation on the affected crate.

**Diff scope matches the request exactly:**
- `crates/sifr_hir/src/flow_graph.rs:483-493` — `build_loop` now creates `join` only inside the `else` branch; the `Some(else_body)` branch returns the else-body frontier directly, so the prior unused `join` allocation is gone. Semantic-preserving: same frontier outputs and same edge set where they matter.
- `crates/sifr_hir/src/flow_graph.rs:518-519` — blank line added before `#[cfg(test)] mod tests;`.
- `internal_docs/...m8_first_class_flow_graph.md:3` — status flipped from "M8 implementation review" to "M8 merged".
- `issues/ad-hoc-...-architecture-transfer.md:17` — M8 row in the execution tracker flipped from "in progress" to "merged" (PR #2243). Only one M8 row exists (the pass-1 follow-up about collapsing duplicate rows was already addressed in earlier PRs); the single M8 local validation block at line 121 is the only one.

**Correctness check on the `build_loop` change:** in the `Some(else_body)` branch, the call `self.build_stmt_list(vec![node], FlowEdgeKind::False, else_body, false)` already returns a frontier, which `build_stmt` propagates as the loop's continuation. The previously allocated `join` was indeed dead — nothing read it. Moving `join` into the `None` branch is a pure dead-code removal with no behavioral change.

**Validation run (all green):**
- `cargo check -p sifr_hir` — PASS
- `cargo test -p sifr_hir flow_graph` — PASS (7/7, including the 4 closeout regression tests)
- `cargo fmt --check` — PASS
- `cargo clippy -p sifr_hir -- -D warnings` — PASS
- `scripts/check_hir_maintainability_guardrails.py` — PASS
- `scripts/check_file_size_guardrails.py` — PASS (flow_graph.rs at 520 lines, well under 900)

Tracker/docs accurately reflect M8: PR #2243 corresponds to commit `a7211f4d4 Add first-class HIR flow graph`, which is the head commit on the branch — "merged" is the correct state.

**SATISFIED.**
