# `implementation pass_clone_1` Iterator/Comprehension Ownership Traceability

Capability: `issues/ownership-aware-collection-lowering-and-clone-elision.md`

## Scope

`implementation pass_clone_1` implements shared ownership-aware iterator planning for:

- structural `for` / comprehension iterator sources
- simple-lowering `map` / `filter` / comprehension / generator paths
- registry-backed iterable-to-iterator lowering used by iterator builtins (for example `enumerate`)

Planner axes applied by lowering:

- `ValueCategory`: `Place | Temporary`
- `SourceAccessMode`: `Preserve | Consume`
- `YieldMode`: `Copy | Clone | Move | Borrow`

## Key implementation points

- shared planner introduced in `crates/sifr_codegen/src/helpers.rs`
  - `plan_iterator_ownership(...)`
  - `plan_iterator_ownership_with_element_hint(...)`
- structured lowering routes through planner-derived decisions in:
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs` (simple `for` iterator lowering)
- simple expression lowering routes through planner-derived decisions in:
  - `crates/sifr_codegen/src/lower_expr.rs`
- registry iterator conversion now uses planner-derived preserve/consume + copy/clone behavior in:
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`

## Evidence: clone-elision outcomes

### 1. Copy-element named collections now use copy-oriented iteration

Command:

- `cargo run -q -p sifr -- emit demos/control_flow/main.sifr`

Observed shape:

- `for n in nums.iter().copied()`

(Previously this path emitted `iter().cloned()`.)

### 2. Temporary containers avoid source-level clone-before-iteration in `map`

Command:

- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/iterator_pipeline_cloning.sifr`

Observed shape:

- `Box::new(vec![5 as i64, 6 as i64].into_iter().map(...))`

(no `clone().into_iter()` source pre-clone for this temporary path)

### 3. Structural range iteration no longer emits boxed clone noise

Command:

- `cargo run -q -p sifr -- emit demos/control_flow/main.sifr`

Observed shape:

- `for i in 1 as i64..n + (1 as i64)`

(no `Box::new((range).clone().into_iter())` in structural loop lowering)

### 4. Builtin enumerate path aligns with planner output

Command:

- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/iterator_pipeline_cloning.sifr`

Observed shape:

- `Box::new((nums).iter().copied().enumerate().map(...))`

(Previously this path used `(nums).clone().into_iter().enumerate()`.)

## Capability artifacts

- pass fixture: `crates/sifr/tests/e2e/pass/iterator_pipeline_cloning.sifr`
- demo: `demos/cloned_iterators/main.sifr`

## Validation snapshot

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/iterator_pipeline_cloning.sifr` -> PASS
- `cargo run -q -p sifr -- run demos/cloned_iterators/main.sifr` -> PASS
- `scripts/run_all_tests.sh --profile create-pr` -> PASS

Residual items intentionally deferred to later implementation passes:

- indexing/safe-indexing/slicing/star-unpack clone-elision (`implementation pass_clone_2`)
- generic-hardening and conservative parity readiness (`implementation pass_clone_3`)
