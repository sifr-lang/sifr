I read the issue, the architecture/diagnostic docs, all four prior Item 2 review artifacts, the LRU corpus milestone review, and the complete uncommitted delta against `b3495318dc59a79c678fe874619f993fed5deb4b`, then ran read-only reproductions with the up-to-date `target/debug/sifr`.

## Pass-3 remediation verification

| Claim | Verdict | Evidence |
|---|---|---|
| 1. Constructor `self` as fresh `MutableBorrow` root; partitioned constructor codegen | **Partially fixed — see findings 1 and 2** | The place proof, synthetic instance, and static `new` are all correct (`class_body_lowering.rs:440-455`, `class_method_emitter.rs:513-549`). `emit` of the new fixture shows `let mut __sifr_self = Self {...}` → `__sifr_self.items.push(1_i64)` → `__sifr_self`, inherited re-rooting as `__sifr_self.baseowner.helper.bump()`, no clone/take, `fn new()` still static. Overlap checking works inside `__init__` (`self.a.absorb(self.a)` → `SIFR-OWN-0002`). The statement **partition itself is defective**. |
| 2. LRU migration committed, merged, clean pointer | **Fixed** | `git submodule status` → `+7772857c6…` with no `-dirty`, on `remotes/origin/HEAD`; inner log head is `7772857 Snapshot LRU head before mutable receiver calls (#40)`; inner `git status --porcelain` empty. Parent diff records only the pointer bump `a20d9d5 → 7772857`. Representative algorithmic suite re-run here: **12/12, 0 failures**, including `0146_lru_cache`. |
| 3. `append(len(...))` boundary pinned + user guidance | **Fixed** | `crates/sifr/tests/e2e/fail/mutable_receiver_overlapping_shared_read.sifr` pins the exact `values.append(len(values))` shape at `col=19` with `SIFR-OWN-0002`; both `error_page_examples/SIFR-OWN-0002.md` and `docs/errors/SIFR-OWN-0002.mdx` now document the overlap scope and the snapshot rewrite. `heapq.sifr:190-193,235-239` migrated consistently. |
| 4. Plain-dict indexed arm live; `key_guard_token` StringLiteral gap fixed | **Fixed** | `sequence_guards.rs:258` adds the `Expr::StringLiteral` token. Reproduced end to end: literal-key `table["a"].append(3)`/`.pop()` under `if "a" in table` and the variable-key form both check clean; `guarded_index.rs:430-440` unit added. The pass-3 reproducer's failure was indeed the missing literal token, not a dead arm. |
| 5. Compiler-generated optimizer fallback restored to full union | **Fixed** | `compiler_generated_mutating_methods.rs` is a strict superset of base `MUTATING_METHODS` (adds `retain`, `__sifr_add_task`); nothing dropped. `optimization_helpers.rs:171-215` proves unprotected compiler `write`/`append` locals keep `mut` and that a source-only name (`source_mutation`) is demoted unless in `protected_mutable_place_roots`. Both production paths pass the protected set (`lib_modules_and_codegen.rs:628-630`, `entrypoints.rs:159`). |

Also confirmed: zero remaining references to `pending_self_field_clone_suppression`, `method_call_needs_field_clone_suppression`, `method_mut_arg_needs_field_clone_suppression`, `body_contains_field_assign_codegen` anywhere in `crates/`, `docs/`, `internal_docs/`. Diagnostic registry/catalog/baseline/manifest additions for `SIFR-OWN-0014`/`PROTO-0005`/`PROTO-0006` are mechanical registrations — no budget, baseline, sample-count, or waiver change.

## Findings

**1. (High, blocking) The constructor statement partition silently reorders source-visible effects.**
`class_method_emitter.rs:497-508` classifies each non-field constructor statement by whether it mentions `self`; `append_constructor_result` (`:513-549`) emits every self-independent statement *before* the struct init and every self-referencing statement *after*. Relative order between the two groups is discarded, so data and effect dependencies crossing the split break.

Silent wrong output (compiles, exit 0):
```python
class C:
    items: list[int]
    def __init__(self):
        self.items = []
        self.items.append(1)
        print(len(self.items))   # self -> post-init
        print("done")            # no self -> pre-init
```
prints `done` then `1`; source order requires `1` then `done`. Second instance:
```python
    def __init__(self, flag: bool):
        self.items = []
        n: int = 0
        if flag:                 # self -> post-init
            self.items.append(1)
            n = n + 1
        print(n)                 # no self -> pre-init
```
prints `0`; source requires `1`.

Leaked rustc error for the reverse dependency:
```python
    def __init__(self, x: int):
        self.total = x
        doubled: int = self.total * 2   # post-init
        print(doubled)                  # pre-init
```
→ `error[SIFR-BUILD-0005]: … error[E0425]: cannot find value 'doubled' in this scope`.

All three are rejected by the exact base (which emitted every non-field statement pre-init, where any `self` reference failed), so these are **newly accepted programs that silently produce wrong observable behavior** — the same guarantee class ("compiles and silently loses a source-visible effect") this issue exists to close. Nothing covers it: `constructor_mutating_receiver_places.sifr` and `receiver_codegen_tests.rs:167-208` only use constructors whose post-init statements are *all* self-referencing, and `internal_docs/architecture.md:453-457` documents the mechanism without stating any ordering caveat.

**2. (Medium) The self-reference probe and the `__sifr_self` rename miss `self` carried in statement `object: String` fields, so nested constructor field assignment still emits illegal `self`.**
`constructor_stmt_references_self` and the rename both run through `visit_hir_function_exprs_mut`, which only reaches `HirExpr` nodes. `HirStmt::NestedFieldAssign`/`FieldAssign`/`SubscriptAssign` carry their receiver as `object: String` (`crates/sifr_ir/src/hir_nodes.rs:437-451`), so:
```python
class Owner:
    helper: Helper
    def __init__(self):
        self.helper = Helper()
        self.helper.n = 7
```
is classified as *not* referencing `self`, lands pre-init, and emits
```rust
fn new() -> Self {
    self.helper.n = 7_i64;
    Self { helper: Helper::new() }
}
```
→ `SIFR-BUILD-0005` wrapping `error[E0424]`. Same for a field assignment nested inside a conditional (`if flag: self.a = 5` → E0424 + E0063). This shape is pre-existing on base, so it is not a regression — but remediation claim 1 asserts "nested class-field mutation" coverage, and the fixture only exercises nested *method* calls (`self.helper.bump()`). The remediation is narrower than claimed, and the two probe/rename sites need the same string-object treatment the place emitter already applies.

## Validation reproduced on this tree

`cargo test -p sifr_lowering receiver` 33 pass; `-p sifr_lowering place` 8 pass; `-p sifr_lowering guard` 68 pass; `-p sifr_codegen receiver` 30 pass; `-p sifr_codegen place` 10 pass; `sifr_lowering --lib` 888 passed / 1 ignored (889 total); `sifr_codegen --lib` 932 passed; `cargo test -p sifr -- --skip test_e2e_pass` all lanes green (116/12/36/6/1/3 tests, includes the annotated fail corpus and emit checks); all 6 new pass fixtures plus the extended `class_method_mut_borrowed_field_argument.sifr` exit 0; representative algorithmic subset 12/12; `cargo fmt --check`, `cargo clippy --workspace -- -D warnings` (the AGENTS.md gate command), `check_hir_maintainability_guardrails.py`, `check_file_size_guardrails.py` (3006 files, largest new file 876 lines), `check_docs_error_code_links.py`, and `git diff --check` all pass. I did not run the 680-fixture E2E pass corpus or the full merge gate. `cargo clippy --all-targets` fails, but every location is in files untouched by this diff (`sifr_stdlib_imports`, `sifr_ipc`, `sifr_runtime`, `rust_interop_direct`, and untouched lines of test modules) — pre-existing, outside the documented gate. Repository state is unchanged (167 entries, identical to the starting snapshot).

Four of the five pass-3 remediation items are fully closed. Item 1's place proof and codegen shape are right, but the partition it depends on is unsound: it admits programs whose constructor effects execute out of source order with no diagnostic, and does not cover the nested-field-assignment shape it claims.

NOT SATISFIED
