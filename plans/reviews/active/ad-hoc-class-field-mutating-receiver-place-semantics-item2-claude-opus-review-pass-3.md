## Item 2 independent review pass 3

Base: `b3495318dc59a79c678fe874619f993fed5deb4b`. I read the issue plan, architecture/diagnostic docs, passes 1–2, the full lowering/codegen/diagnostics/fixture delta, and ran read-only reproductions with the current `target/debug/sifr` (verified up to date).

### Pass-2 remediation: verified

| Pass-2 finding | Verdict | Evidence |
|---|---|---|
| 1. Shared receiver vs `mut` argument unchecked (leaked `E0502`) | **Fixed** | `validate_call_overlaps` now takes the receiver object and checks `SharedBorrow`/`Owned` receivers against mutable arg places (`method_receiver_places.rs:223-234`). Reproduced: `self.stock.observe(self.stock)` and `c.observe(c)` both → `SIFR-OWN-0002`; fixture `shared_receiver_mutable_argument_overlap.sifr` matches. |
| 2. Module-level roots silently lose mutation | **Fixed** | `BindingKind::ModuleConstant` (`scope.rs:20,204-206`) rejected at `method_receiver_places.rs:378`. Reproduced: `GLOBAL_HELPER.bump()` and `COUNTS.append(1)` → `SIFR-OWN-0014`. Shared-receiver module-constant reads still lower to `__const_X()` (verified via `emit`). |
| 3. Slice receivers | **Fixed** | `HirExpr::Slice` now an owned temporary (`method_receiver_places.rs:586`); `values[1:].pop()` runs correctly. Walrus rejected with fixture. |
| 4. Dead plain-dict `pop` arm + doc claim | **Not fixed** (see finding 4) | |
| 5. `body_contains_field_assign_codegen` | **Fixed** | Zero references anywhere in `crates/`, `docs/`, `internal_docs/`. Same for all three suppression helpers. |
| 6. Optimizer only in test path | **Fixed structurally** | Production assembly now runs it (`lib_modules_and_codegen.rs:628`), `RustStmt::Verbatim` conservatism restored (`mutability_and_clone_rewrites.rs:328-330`), compiler-generated table extracted (`ir_optimize/compiler_generated_mutating_methods.rs`). Residual risk in finding 5. |
| CFG panic attribution | **Supported** | The `cfg.rs` delta is test-constructor-only (`mutable_arg_places: Vec::new()` in 5 test literals), consistent with the pre-existing claim. |

Core defect closure reproduced end to end: direct, nested (`self.mid.inner.bump()`), `mut` param, `own mut` param, owned-local, inherited (`super().__init__`), and generic receivers all mutate original storage. All 22 new fail fixtures emit exactly their annotated codes; all 5 new pass fixtures plus the extended `class_method_mut_borrowed_field_argument.sifr` exit 0. `cargo fmt --check`, `git diff --check`, docs-error-link, HIR, and file-size guardrails pass; `cargo test -p sifr_lowering receiver` (33) and `-p sifr_codegen place` (10) pass. Diagnostic registry/catalog/baseline additions are mechanical registrations of the three new codes — no baseline, budget, or waiver loosening.

### Findings

**1. (High, blocking) Constructors can no longer call any mutating method on `self` or a `self` field.**
```
class Helper:
    items: list[int]
    def __init__(self):
        self.items = []
        self.items.append(1)      # error[SIFR-OWN-0014] ... self.items
        self.setup()              # error[SIFR-OWN-0014] ... self
```
Root cause: `class_body_lowering.rs:657` sets `HirFunction.receiver = None` for `__init__` (renamed `new`), so `persist_method_receiver` early-returns at `method_receiver_analysis.rs:313` and never patches the retained receiver fact — and its lookup key would be `Class.new` while `class_body_lowering.rs:446-449` registered `Class.__init__`. The `self` binding therefore keeps its declared `SharedBorrow`, and `prove_mutable_place` rejects the `BindingKind::Receiver` root at `method_receiver_places.rs:391-399`. Reproduced for `self.field.append(...)`, `self.field.bump()` (nested class field), and `self.method()`. This is a regression introduced by Item 2 — `SIFR-OWN-0014` does not exist on base — and it rejects one of the most common initialization idioms in the language. It contradicts the acceptance criterion *"Direct, nested, inherited, local, `self`, … field receiver mutations are observable after the call"*, and `__init__` appears nowhere in plan §3's deliberately-unsupported list.

**2. (High, blocking) The LRU compatibility fix is uncommitted inside a submodule and cannot ship with this PR.**
`verification/areas/algorithmic_compatibility/corpora/leetcode` is a submodule (`.gitmodules:9-12`); the parent diff records only `a20d9d5…-dirty`. The `self.head` snapshot edits exist solely as uncommitted working-tree changes in `src/0146_lru_cache.sifr` inside that submodule. `algorithmic_compatibility_checks` is a merge-gate lane (`verification/runner/sifr_verify/profile_runner.py:88,596-602`), so merging Item 2 as-is re-breaks that lane. Concrete requirement: commit the corpus change upstream in `sifr-lang/leetcode` and bump the submodule pointer in this PR.

**3. (Medium) The same-call rule rejects Copy-value and receiver-overlapping reads that rustc accepts, with real churn and no coverage.**
`arr.append(len(arr))`, `self.items.append(len(self.items))` → `SIFR-OWN-0002` (`method_receiver_places.rs:209-214` + `collect_footprint`). The emitted Rust (`arr.push(arr.len() as i64)`) is legal under two-phase borrows, so for a method receiver the rejection is stricter than necessary; the argument-position case (`_sift_down(heap, 0, len(heap))`) genuinely needs it. The cost is measurable: `stdlib/sifr/heapq.sifr:190-193,235-239` had to be rewritten and the LeetCode corpus had to be migrated. Plan §4 does sanction this ("intentionally rejects … rather than depending on … two-phase-borrow behavior"), so I do not treat it as a contract violation — but there is no pass/fail fixture pinning the `x.append(len(x))` boundary, and no user-facing migration note; the `SIFR-OWN-0002` page (`docs/errors/SIFR-OWN-0002.mdx`) only shows the owned-move class.

**4. (Low) Dead plain-dict arm retained and `architecture.md` still documents it as supported.**
`method_receiver_places.rs:414-423` accepts `append`/`pop` on plain `Type::Dict` with a list value, but plain dict indexing types as `T | None`, so even the guarded form fails earlier — reproduced with `d: dict[str, list[int]]; d["a"] = [1,2]; if "a" in d: d["a"].append(3)` → `SIFR-STDLIB-0001` for both `append` and `pop`. `internal_docs/architecture.md:470-476` asserts "the existing guarded/narrowed `dict[K, list[V]]` `bucket[key].append(...)` and zero-argument `bucket[key].pop()` lowering … are compiler-owned exceptions"; only the `defaultdict` aliases actually work (`collections_boundary_ownership.sifr:17-18`). Pass-2 finding 4 was narrowed but not resolved.

**5. (Low) Optimizer table narrowing dropped names codegen still emits, in a pass that is newly live in production.**
`COMPILER_GENERATED_MUTATING_METHODS` drops `append`, `write`, `reverse`, `set`, `writerows`, `writeln`, `kill`, `setstate`, `set_level`, … relative to the base `MUTATING_METHODS`, and codegen still emits `.reverse()` (`methods/list.rs:134,157`), `.write(` (`preamble/io_file_handles.rs:486`, `render/render_helpers.rs:467`), and `.append(` literals. I could not reproduce a failure — `x.reverse()`, `y.sort()`, and the `open()`/`write` fixture all keep `let mut` because those roots are in `protected_mutable_place_roots` — so this is residual risk only. A focused test for one *unprotected* compiler-synthesized `write`/`append` local would close it.

### Evidence sufficiency

The interleaved base-vs-tree timing comparison, the isolated 8/8 representative performance run, and the exact-base CFG reproduction are credible and correctly scoped; the unchanged budgets/baselines check out. What remains missing is not statistical: closure still needs (a) finding 1 fixed with a constructor-mutation pass fixture, (b) the leetcode submodule change committed and its pointer bumped so the algorithmic lane is actually green from a clean checkout, and (c) one uncontended end-to-end `scripts/run_all_tests.sh` — which by the issue's own account has never completed.

NOT SATISFIED
