# Sifr M8 First-Class Flow Graph — Pass-3 Review

## Scope of pass-2 follow-up evaluation

The pass-2 review identified actionable gaps between the reviewer's hypothetical API surface (e.g., `Effect::None`, `RefinedToNone`, `FlowFacts::debug` with a 16-predecessor cap, `Effect::Spawned`, `cfg_view`) and the actual current diff. The pass-2 closeout explicitly disambiguated the real implementation surface and added targeted regression tests and documentation to lock the semantics in. This pass-3 review re-evaluates the implementation against that clarified surface.

## Findings

### 1. Test coverage matches the real effect vocabulary

The new test `graph_fingerprint_includes_effect_payloads` exercises the actual `FlowEffect` variants and verifies that the graph fingerprint incorporates effect-payload labels (not just node identity). This is the correct semantic anchor: the prior pass-2 hypothesis that fingerprinting would be tied to a generic `Effect::None`/`Effect::Spawned` enum was incorrect. Fingerprint stability now rests on the concrete `FlowEffect` payload (e.g., `Narrow`, `ClearNarrowing`, `Mutate`, `Move`, `Escape`), which is what real consumers will hash on. No drift between the test name and the asserted behavior.

### 2. Snapshot-scoped flow effect exposure is correctly narrowed

`lowering_result_exposes_snapshot_scoped_flow_effects` now asserts the realistic set of post-lowering observable effects for the covered program: `Narrow { narrowed_type: Type::None }`, `ClearNarrowing` (for both the variable and a collection's items), and `Mutate`. This correctly captures what the current diff actually emits at the lowering boundary, rather than asserting a hypothetical `Effect::None`/`RefinedToNone`/`Effect::Spawned` triple. The narrow-to-`Type::None` assertion pairs with the doc clarification that "narrowing to None is `FlowEffect::Narrow { narrowed_type: Type::None }`" — the two reinforce each other and rule out the ambiguous interpretation flagged in pass-2.

### 3. Await + task-handle move semantics are pinned

`await_task_handle_records_move_effect` codifies the move-tracking behavior for awaiting a task handle. This is the load-bearing case for the first-class flow graph: the move effect must be observable so downstream passes (escape analysis, ownership tracking) can consume it. Asserting it directly prevents a regression where the await site silently drops the move event.

### 4. Documentation is honest about scope

The added doc notes accomplish three things at once:

- They disambiguate the `Narrow → None` representation, preventing a future contributor from inventing a parallel `RefinedToNone` variant.
- They document that the fingerprint deliberately includes effect-payload labels, which is a stability contract: changing a payload label is now an observable change to the graph fingerprint, which is exactly the property consumers want.
- They defer LSP exposure of the flow graph to later tracing/debug milestones, which is the right call — exposing it now would entrench a public API before the underlying representation has stabilized through internal use.

The deferral is explicit, which is what the pass-2 review asked for: it is no longer implicit that "LSP will get this eventually" — the doc now states that it is intentionally out of scope for this milestone.

### 5. No new advisories introduced

The pass-2 follow-up is additive (new tests, new doc paragraphs) and does not touch the lowering, CFG construction, or fingerprint algorithm. There is no risk surface expansion in the production code path as a result of the closeout.

## Validation

- `cargo fmt --check` — PASS
- `cargo test -p sifr_hir flow_graph -- --nocapture` — PASS (7 tests, including the 4 new follow-up tests)
- `cargo clippy -p sifr_hir -- -D warnings` — PASS
- Prior full-suite validation (driver, frontend, analysis, LSP, package/file guardrails, TypeScript-Go M1 guardrails, end-to-end excluding `test_e2e_pass`, workspace clippy, `scripts/run_all_tests.sh --profile quick`, 306.27s wall time with report at `target/validation_lane_reports/quick.latest.json`) — PASS, advisories unchanged.

No new advisories surfaced from the follow-up additions.

## Cross-checks against the original M8 review

- The fingerprint contract is now both implemented and tested under the real effect vocabulary, not a hypothetical one.
- Snapshot-scoped effect exposure is asserted at the granularity the diff actually produces.
- Move effects at await sites are regression-locked.
- The deferral of LSP exposure is documented, not assumed.

All four open items from the pass-2 actionable list are closed.

## Verdict

**SATISFIED**
