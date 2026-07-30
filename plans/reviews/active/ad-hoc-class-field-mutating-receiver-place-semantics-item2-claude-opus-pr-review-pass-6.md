# Item 2 PR Review — Class-Field Mutating Receiver Place Semantics (PR #3082)

**Range reviewed:** `f1c34cf9aaabadda546e670fca190decc580c935` → `ea4790f72aa2fafcc50125d6a3755c25d70fd5ed` (182 files, +5524/−1060). Working tree at exact head, clean except my own untracked review placeholder. No files modified, no git/PR state changed.

I read the phase contract, all five prior Item 2 artifacts, the upstream LRU corpus review, the full implementation diff, and ran read-only probes with `target/debug/sifr` (verified current: `cargo build` no-op at head).

---

## 1. Core defect closure — verified by execution

The originating defect is fixed, not papered over.

| Probe | Result |
|---|---|
| `self.helper.bump()` ×2 + `self.mid.inner.bump()` + `mut box: Owner` field receiver | prints `3` / `2`; emitted Rust is `self.helper.bump(); self.mid.inner.bump(); r#box.helper.bump();` — **zero** `.clone()`/`take()` between root and call |
| Same program with `box: Owner` (no `mut`) | `SIFR-OWN-0005` at `a.sifr:29:5` — the previously silent-loss shape is now rejected |
| 3-level inheritance (`C(B(A))`, field declared in `A`) | prints `2`; parent re-rooting composes with place emission |
| `self.a.absorb(self.b)` (mutable receiver + `mut` sibling argument) | receiver and argument mutations both observable (`1`/`1`), return `2` |
| `free_fill(self.stock)` and `self.sink.absorb(self.stock)` | both mutate real storage |
| Nested `def` capturing `self` and mutating a field | prints `1` |
| Value-read preservation: `copy_items = self.items.copy(); copy_items.append(100)` | `5` vs `4` — ordinary field-read clone semantics intact |
| Read-only `self.right.peek()` (fixture) and `emit` of `class_field_mutating_receiver_places.sifr` | `fn read_only(&self)`, `fn bump(&mut self)`, `fn mutate(&mut self)`, no `clone` anywhere in emitted output |

Ambient state is gone: `grep` for `pending_self_field_clone_suppression`, `method_call_needs_field_clone_suppression`, `method_mut_arg_needs_field_clone_suppression`, `body_contains_field_assign_codegen` across `crates/` returns **0** hits. `lib_emitter_state.rs:89-90` replaces the counter with `protected_mutable_place_roots`; `field_and_stdlib_rewrites.rs:111` now computes `needs_clone` with no suppression term.

## 2. Contract conformance — spot checks that held

- **Place model / root eligibility** (`method_receiver_places.rs:351-390`): `BindingKind::{Local, Parameter(mut|own mut), Receiver(MutableBorrow)}` accepted; `ModuleConstant`, `EphemeralLocal`, immutable params, optional/recursive/callable roots rejected. Verified live: loop element → `OWN-0014` (`b3.sifr:10:9`), module constants → `OWN-0014` ×2, `with`/tuple-unpack/chained-assignment/except locals accepted (fixture `class_field_mutating_receiver_contexts.sifr` + my `pair_a, pair_b` probe).
- **Overlap** (`places_overlap`, `:350`): prefix rule with `BindingId` roots. `self.helper.absorb(self.helper)` → `OWN-0002` at the argument; `self.helper.read(self.other)` accepted (`no errors found`). All 24 new/changed fail fixtures emit exactly their annotated code (checked each with `sifr check`).
- **Fixed Rust traits** (`method_receiver_analysis.rs:191-231`): `__eq__` field-assign, `__str__` builtin mutation, `__repr__` delegating to a mutating method, `__add__`, `__getitem__`, and `__eq__` mutating from a *nested `def`* all produce `PROTO-0006`. Transitive detection works (pass-1 finding closed).
- **Protocol variance**: `PROTO-0005` fires for mutable impl of a shared protocol method. I confirmed this is necessary rather than over-strict: Sifr generates `impl Reader for Unrelated { fn run(&self) }` for structural conformance (`emit d2.sifr` → line 28), so a `&mut self` inherent body genuinely cannot conform.
- **Owned temporaries** (`is_owned_temporary`, `:566-611`): exhaustive compiler-enforced match; `IfExpr` requires *both* branches proven, `Name`/`Index`/`FieldAccess`/`BoolOp`/`WalrusExpr` excluded — matching the `conditional_storage_*` and `walrus_*` fail fixtures.
- **Optimizer** (`mutability_and_clone_rewrites.rs:72-76`, `compiler_generated_mutating_methods.rs`): protected-root set threaded through every recursion arm; the string table is now scoped to compiler-generated spellings; `RustStmt::Verbatim` is treated conservatively (`:322-330`). The pass now runs in production assembly too (`lib_modules_and_codegen.rs:628-631`) alongside test assembly (`entrypoints.rs:159`), as the contract requires.
- **Invariant verifier** (`method_call_verifier.rs:139-190`) rejects a `MutableBorrow` source call with no target, a target whose shape does not match the HIR, and missing argument proof slots. It runs only when `ctx.errors.is_empty()` (`mod_impl.rs:821-843`), so diagnostics are not double-reported — confirmed: `OWN-0014` fixtures emit no ICE. This verifier is what makes the `Ok(None)` returns in `place_emitter.rs` unreachable rather than a silent clone fallback, which I consider the load-bearing guarantee here.
- **No unrouted receiver emitter**: the remaining `try_lower_simple_method_call_expr` family (`lower_expr/iterators_and_callables.rs:364+`) only handles `__sifr_*` task/JoinSet/TaskScope methods through `try_lower_leaf_or_name_expr`, which does **not** accept `FieldAccess` (`leaves_and_plain_calls.rs:582-590`), so field receivers cannot reach it. `class_method_receiver_analysis.rs` survives only as a called-method-name collector for generic bounds/operator requirements — no receiver-mutability decision.

## 3. Independent gate re-runs at exact head

| Check | Result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo clippy --workspace -- -D warnings` | pass (exit 0) |
| `python3 scripts/check_file_size_guardrails.py` | pass (3053 files, limit 900) |
| `check_hir_maintainability_guardrails.py` / `check_docs_error_code_links.py` | pass |
| `git diff --check` | clean |
| Submodule pointer | `7772857c6f` = upstream `Snapshot LRU head before mutable receiver calls (#40)`, clean |
| `cargo test -p sifr_lowering --lib` | **918 passed, 1 ignored** |
| `cargo test -p sifr_codegen --lib` | **941 passed** |
| E2E merge profile | started cold; still running past my review window, so **680/680 + signature `8871ba51135353a4` is not independently re-confirmed by me** (pass 5 did reproduce it on this tree). I terminated the job to avoid leaving a multi-core run contending with your machine. |

## 4. Verdict on the split performance evidence — **acceptable, no required correction**

- `--allow-subset` is a first-class flag of the repository's own gate (`verification/areas/performance/check_budgets.py:44,58,85`), also used by `runner.py`; it is not an ad-hoc bypass.
- The committed baselines themselves use `sample_count: 5`, below `MIN_P95_SAMPLE_COUNT = 20` (`check_budgets.py:27,315`), so p95 is *never* enforced in this repo's normal runs. A short uncontended remeasurement is therefore methodologically equivalent to standard practice, not a weakened variant.
- The quoted thresholds match `data/budgets.json` exactly and unmodified: `perf.check.project.project_graph` 1357.524, `perf.check.single.arithmetic` 1334.139, `perf.diagnostic.json_diagnostic_schema` 1335.954. `git diff` shows no budget/baseline/waiver/manifest change.
- Scheduler contention inflates measurements, i.e. biases against the change; a pass after removing contention is the sound direction.

One factual caveat worth recording (not blocking): `check-single-file-001-arithmetic` at 1328.513 ms sits 0.42 % under budget and ~9.5 % above its recorded baseline of 1212.854 ms, against a ~10 % budget headroom. Given that this diff adds a lowering fixed point, place validation traversals, and a *new* production-side IR mutability pass, that case has almost no remaining margin. I would record the measured medians and sample counts in the ledger so the next change to this area starts from an auditable number.

## 5. Explicitly checked and *not* counted against this PR

- `values.append(len(values))`, `self.insertAfter(node, self.head)`, and `_sift_down(heap, 0, len(heap))` are now `SIFR-OWN-0002`. This is a real source-compatibility narrowing versus Rust two-phase borrows, and it is why `stdlib/sifr/heapq.sifr` and the LRU corpus fixture needed snapshot rewrites. It is **mandated** by the approved contract (§4: "intentionally rejects … rather than depending on … Rust two-phase-borrow behavior"; `self` overlaps every `self.*`), it is documented with the snapshot fix in `docs/errors/SIFR-OWN-0002.mdx`, and it is pinned by fixture `mutable_receiver_overlapping_shared_read.sifr`. Conformant, deliberate, not a defect.
- Constructor rejections (`self.total = 0` after a self use, `self.b = self.a + 1`, tuple-unpack field init) are new `OWN-0014` errors, but base partitioned field assignments out of source order (`git show f1c34cf9a:…/class_method_emitter.rs:319-325`), so these shapes leaked rustc `E0424`/`E0063` before. Diagnosing them is an improvement.
- Generic class instance as a *field* (`ints: Bag[int]`) resolves to `Bag[T] | Bag[int]`, breaking even a plain field read (`len(self.ints.items)` → `list[T] | list[int]`). Pre-existing generic-field typing gap, independent of receivers; this diff does not expand it.
- `_ = f.write(...)` inside a `with` in `main` leaks rustc `E0277`; reproduced with a class-free program, so unrelated.

---

## Actionable findings

**1 — Low/Medium (evidence integrity). The head commit's validation ledger misstates the unit-test counts, reproducibly.**
`plans/issues/active/ad-hoc-class-field-mutating-receiver-place-semantics.md:777-778` records "full lowering tests pass (`889 passed`, `1 ignored`), full codegen tests pass (`933 passed`)". At exact head, `cargo test -p sifr_lowering --lib` yields **918 passed / 1 ignored** and `cargo test -p sifr_codegen --lib` yields **941 passed** (identical with `--no-default-features`). These are not filter artifacts: `#[test]` counts are 919/945 at head and 909/943 at base `f1c34cf9a`, so 889/933 corresponds to no commit in this range — they appear carried over from a pre-final remediation snapshot. The docs commit `ea4790f72` exists solely to record this evidence, and the merge decision cites it.
*Required correction:* update the ledger to the head-reproducible counts (918 + 1 ignored / 941), and while there, record the three remeasured performance medians with their sample counts.

**2 — Low (diagnostic contract + internal-name leak). The constructor `SIFR-OWN-0014` path puts prose, including a compiler-synthesized identifier, into the structured `place` argument and anchors to the wrong span.**
`class_body_lowering.rs:615-621` passes `"`self` before constructor storage initialization (missing {})"` as the registry's `place` argument, whose contract is the canonical place display (`internal_docs/diagnostic_codes.md`, `place (message+json)`). Reproduced: a subclass `__init__` that omits `super().__init__()` reports `missing __sifr_parent` (`k3.sifr:8:9`) — naming a compiler-internal field instead of telling the user to call `super().__init__`, and emitting that string into the machine-readable `place` argument consumed by JSON diagnostics/tooling. `self.b = self.a + 1` reports `missing b`, i.e. the field being assigned rather than the mid-initialization read that caused it, and the range is `func.name.range()` (`def __init__`) rather than the offending statement. The docs page carries correct guidance, so this is message/argument quality, not semantics — but it is a stable diagnostic introduced by this diff.
*Required correction:* report the constructor case with a place-shaped `place` argument plus a cause-specific message (super-call vs. named field), map `__sifr_parent` to user-facing "inherited storage via `super().__init__`", and anchor to the first offending statement.

**3 — Low (over-rejection inconsistency in new check). Writing the explicit field assignment makes a constructor fail that succeeds when it is omitted.**
`class_semantics.rs:106-131` excludes any field appearing in `explicit_initializers` from parameter seeding unconditionally, while codegen's `constructor_instance_fields` will happily source that field from the same-named parameter. Reproduced at head: `def __init__(self, a: int): self.items = []; self.items.append(1); self.a = a` → `SIFR-OWN-0014 (missing a)`, while deleting the redundant `self.a = a` compiles and prints `3`. Conservative (no miscompile), and base leaked `E0424` on the same input, but "adding the explicit assignment breaks it" is a surprising boundary for a stable error.
*Required correction:* seed a field from a same-named parameter even when an explicit initializer exists later, or reject with a message that names the actual restriction.

---

I found no silent clone, no unchecked mutating receiver path, no root-only over-rejection, no missing-metadata codegen fallback, no optimizer demotion of a proven root, no panic risk, and no fixture/diagnostic/guardrail gap. Findings 2 and 3 are quality defects in code this PR introduces; finding 1 is a factual defect in the evidence record the PR commits. All three are small and localized.

**NOT SATISFIED**
