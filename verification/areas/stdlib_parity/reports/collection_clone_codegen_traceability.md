# `implementation pass_clone_0` Codegen Baseline and Architecture Lock

Capability: `issues/ownership-aware-collection-lowering-and-clone-elision.md`
Execution ledger: `issues/ownership-aware-collection-lowering-and-clone-elision-execution.md`

## Objective

Capture a reproducible clone-heavy generated-code baseline and lock one canonical ownership-aware planning model before implementation passes (`implementation pass_clone_1+`) refactor lowering paths.

## Baseline Validation Snapshot (2026-03-21)

- validation profile: `scripts/run_all_tests.sh --profile create-pr`
- result: PASS
  - unit + non-pass e2e: PASS (`37` + `25`)
  - validation matrix (`frontend_mode_parity`, `graph_isolation_validation`): PASS (`7` rows)
  - create-pr e2e pass suite: PASS (`24` fixtures, report signature `e1bf653aaa770517`)

## Emit Inventory Inputs

Captured emit outputs from:

- `demos/iterators_and_comprehensions/main.sifr`
- `demos/ergonomics/main.sifr`
- `demos/safe_indexing/main.sifr`
- `demos/control_flow/main.sifr`

## Clone-Heavy Pattern Baseline (Capability 0 Entry)

| Pattern family | Baseline evidence (emit excerpt) | Interpretation |
| --- | --- | --- |
| temporary / named container cloning before iteration | `nums.clone().into_iter().map(...)`, `(unsorted).clone().into_iter()...`, `(bools).clone().into_iter().any(...)` (from `capability_generics_demo.sifr` emit) | ownership cases are collapsed to clone-heavy fallback instead of planner-derived preserve/consume decisions |
| borrowed copy iteration via clone path | `for x in Box::new(nums.iter().cloned())` (from `capability_generics_demo.sifr` emit) | copy-friendly iteration still routed through `cloned()` |
| safe indexing clone path | `__sifr_index_list.get(__sifr_index_norm).cloned()` (from `capability_safe_indexing_demo.sifr` and `capability_control_flow_demo.sifr` emit) | borrowed copy extraction uses clone path rather than copy-oriented extraction |
| boxed range clone path | `for i in Box::new((1 as i64..n + (1 as i64)).clone().into_iter())` (from `capability_control_flow_demo.sifr` emit) | structural range lowering emits ownership noise (`clone` + boxed iterator) |
| star-unpack whole-source clone | `let _star_tmp = items.clone(); let first = _star_tmp[0].clone();` (from `capability_ergonomics_demo.sifr` emit) | whole-source clone is used for unpack lowering convenience |

## Hotspot Ownership

| File | Capability-set-0 ownership reason |
| --- | --- |
| `crates/sifr_codegen/src/stmt_support_emitter.rs` | structural `for`/comprehension iteration source lowering currently emits clone-heavy iterator fallbacks for iterable/class/range paths |
| `crates/sifr_codegen/src/lower_expr.rs` | simple `map`/`filter` lowering paths clone iterables and filter items in generic fallback branches |
| `crates/sifr_codegen/src/lower_stmt.rs` | simple star-unpack and copy extraction paths clone source containers/elements for convenience |
| `crates/sifr_type_system/src/types.rs` | `Type::ownership()` already exposes `Copy` vs `Move`, but lowering paths do not centralize planner decisions around it |

## Locked Planner Rules (Capability 0)

This capability locks the planner axis model used by later implementation passes:

- `ValueCategory`: `Place | Temporary`
- `SourceAccessMode`: `Preserve | Consume`
- `YieldMode`: `Copy | Clone | Move | Borrow`

Decision inputs must include:

- value category (`Place` vs `Temporary`)
- element ownership (`Copy` vs `Move`) using `Type::ownership()`
- semantic source access rules (preserve vs consume)
- conservative generic handling (`TypeVar`, `Any`, unions with move members)

Implementation invariant:

- both structural lowering (`stmt_support_emitter.rs`) and simple lowering (`lower_expr.rs`, `lower_stmt.rs`) must derive collection/iterator decisions from one shared planner path.

## Capability-set-0 Lock Artifacts

- positive fixture: `crates/sifr/tests/e2e/pass/collection_cloning.sifr`
- demo: `demos/collection_cloning/main.sifr`
- ownership-safety negative anchor selected for later regression locking:
  - `crates/sifr/tests/e2e/fail/borrow_escape_store.sifr`

## CPython Family Mapping (Reference Only)

| CPython family | Direction in this capability |
| --- | --- |
| `Lib/test/test_iter.py` | adapted in implementation passes via iterator/comprehension ownership correction |
| `Lib/test/test_list.py` / `test_tuple.py` | adapted in implementation passes via indexing/slicing/unpack clone-elision |
| `Lib/test/test_dict.py` | adapted in implementation passes via safe indexing/get extraction ownership correction |
