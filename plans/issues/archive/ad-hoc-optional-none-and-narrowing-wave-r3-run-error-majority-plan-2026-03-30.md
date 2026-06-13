# Ad Hoc Optional/None Closure: Wave-R3 Run-Error Majority Plan (2026-03-30)

Status: reviewer-pass1 adjustments applied (implementation gate pending)
Owning phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`
Probe baseline: `verification/leetcode/run_error_quartet_plus_baseline24_probe_20260330_wave_r2.json` (`RUN_ERROR=16`)
Reviewer artifact: `reviews/ad-hoc-optional-none-wave-r3-run-error-majority-review-pass1.md`

## Objective

Close the majority of remaining run-stage failures by fixing compiler root causes first, while keeping Sifr principles explicit:

- static typing at check stage,
- explicit Optional/None handling (no hidden auto-unwrapping),
- no Python-truthiness compatibility shortcuts,
- and no run-stage Rust type surprises for programs that passed `check`.

## Current Residual Cluster (Probe-24 after R2)

Remaining run errors:

- `0054`, `0071`, `0167`, `0187`, `0231`, `0349`, `0367`, `0416`, `0441`, `0459`, `0463`, `0763`, `0846`, `1461`, `1582`, `1905`

First-code distribution:

- `E0308=11`
- `E0277=2`
- `E0428=1`
- `E0425=1`
- `E0369=1`

## Root-Cause Buckets

### Bucket A: Empty-collection element type remains `Any` (`Vec<Box<dyn Any>>` emission)

Fixtures:

- `0054`, `0071`, `0349`, `0763`

Signal:

- local `res = []` / `stack = []` inferred as `Any`-container in generated Rust (`Vec<Box<dyn Any>>`), then concrete `int`/`str` appends fail.

Owning layer:

- HIR/type-system local collection specialization for empty literals under dominated method writes (`append` / `push`-equivalent paths), not fixture hacks.

### Bucket B: Run-stage type failures that should be check-stage semantics diagnostics

Fixtures:

- `0167`, `0367`, `0463` (missing guaranteed return),
- `0416`, `0846` (numeric-in-condition bool contract violations under Sifr semantics),
- `0231` (duplicate function definitions in same module).

Signal:

- `check` currently passes, but Rust compile fails (`expected bool`, `expected return type`, redefinition).

Owning layer:

- HIR semantic validation completeness (`condition must be bool`, `module symbol redefinition diagnostics`, `non-None function must return on all paths`).

### Bucket C: Structured/codegen parity defects

Fixtures:

- `0187` (`list(set_obj)` lowering yields `Vec<&String>` instead of `Vec<String>`),
- `0459` (`contains` pattern borrow level mismatch `&&String`),
- `1461` (`set(generator)` emitted as unresolved `set(...)` call),
- `1582` (`[0] * n` list-repeat lowered to invalid Rust vec multiplication),
- `0463` (`+=` rendered as `+==` in one lowering path),
- `0846` (heapq helper call-name mismatch: emitted defs `heapify/heappop` vs calls `__compat_sifr_heapq_*`),
- `1905` (Optional index expression survives condition lowering as `Option<i64>` in bool context).

Owning layer:

- `crates/sifr_codegen` structured lowering/render parity and helper naming consistency.

### Bucket C1 (explicit split): AugAssign render contract defect

Fixtures:

- `0463`

Signal:

- generated Rust emits `+==` in one lowering path, proving mixed normalized/raw `AugAssign` op strings are not normalized at final render boundary.

Owning layer:

- codegen render contract hardening (`RustStmt::AugAssign` should render correctly for both normalized and raw op representations).

### Bucket D: Optional-boundary + guarded string index in loops

Fixtures:

- `0763`, `1905`

Signal:

- guarded index/value paths still materialize as `Option[...]` inside bool/key contexts where dominated guards should allow non-optional use or require explicit check-stage diagnostics.

Owning layer:

- HIR narrowing and condition lowering contracts across `while`-dominated index expressions and boolop composition.

## Fixture-to-Bucket Ownership Map

- `0054`: A + D
- `0071`: A
- `0167`: B
- `0187`: C
- `0231`: B
- `0349`: A
- `0367`: B
- `0416`: B
- `0441`: C
- `0459`: C
- `0463`: B + C1
- `0763`: A + D
- `0846`: B + C
- `1461`: C
- `1582`: C
- `1905`: C + D

## Proposed Wave Execution Order

### Wave-R3a (semantic correctness gate)

Goal:

- move Bucket B failures from run-stage surprises to deterministic check-stage diagnostics or valid lowering where semantics are explicit.

Scope:

- HIR/module duplicate-definition detection for same function name in one module,
- condition type enforcement for `if/while` (bool-only),
- return-path completeness diagnostics for non-`None` returns.

Expected effect:

- eliminates a large class of `RUN_ERROR` states that violate Sifr static-contract expectations.

### Wave-R3b (codegen parity corrections, split by risk)

Goal:

- eliminate clear codegen defects independent of frontend semantics.

Scope (R3b1: low-risk correctness hardening):

- augassign render normalization (`+=` / `+==` boundary fix),
- heapq helper call-name consistency,
- borrow-normalized `contains` lowering.

Scope (R3b2: data-shape lowering correctness):

- list-repeat lowering (`[x] * n`),
- set(generator) lowering,
- iterator collect cloning for owned `Vec<T>` from iterator of refs,
- float/int compare typing parity where HIR admits the expression (e.g. `0441`).

### Wave-R3c (container/optional stabilization follow-up)

Goal:

- close Bucket A + D residuals with principled HIR narrowing/inference (empty-literal specialization + guarded index typing).

Scope:

- empty-list local specialization from append-dominated evidence,
- guarded string/list index non-optional typing under proven in-range facts,
- optional-in-condition normalization only where semantically explicit.

## Validation Plan Per Wave

- targeted `cargo test -q -p sifr_hir ...` / `-p sifr_codegen ...` for each rule,
- targeted `target/release/sifr check/run` on owning fixture subset,
- `scripts/run_all_tests.sh --profile quick`,
- refresh run-error probe artifact for the baseline-24 cohort after each wave.

Guardrails required by reviewer pass-1:

- each wave must include fixture-id ownership list in PR body,
- each wave must include one direct regression test for its primary lowering rule,
- no fixture canonicalization in R3a/R3b unless reviewer file explicitly marks fixture as outside Sifr semantics.

## Reviewer Questions

1. Is Wave-R3a (semantic-gate first) the right ordering under Sifr principles, or should we prioritize codegen parity (R3b) first?
2. For non-bool conditions (`if sum(nums) % 2`), should we strictly reject at check stage (preferred by principle) instead of adding implicit truthiness lowering for arbitrary numeric expressions?
3. For missing non-`None` returns, do we enforce check-stage diagnostics only, or also add a defensive codegen guard to avoid Rust run-stage fallout if frontend misses a case?
4. Is empty-collection specialization from append-dominance acceptable as principled inference, or should such locals require explicit annotations to remain strictly explicit?
