## Review: Wave 4 — `defaultdict(int)` augassign key specialization (PR #3079, head `1bcbd54c1`, base `9934b72e5`)

I read the full crate + plan diff, the patch/refinement/isolation machinery and every caller, then built three compilers (head `1bcbd54c1`, pass-2 head `8acf7ad01`, base `9934b72e5`) and ran differential `check`/`emit`/`run` probes, plus the repo's own gates. No files were modified (`git status` unchanged; the two temporary worktrees I created were removed).

### Actionable findings

**None.**

### Verification performed

Recorded evidence reproduced independently at head:
- `cargo test -p sifr_lowering` → **908 passed, 0 failed, 1 ignored**; `-p sifr_codegen` → **939 passed**; focused new suites **10/10** lowering and **5/5** codegen.
- Full native e2e: **678 pass tests completed (678 passed, 0 failed)**, `test_e2e_pass ... ok` in 586 s (`verification/runner/e2e/run_e2e_pass.sh`).
- `cargo fmt --check` clean; `check_hir_maintainability_guardrails.py` PASS; `check_file_size_guardrails.py` PASS (3019 files, limit 900); clippy on `sifr_lowering`/`sifr_codegen` shows the same 2 pre-existing test-file warnings as base (`python_buffer_contract_tests.rs:742`, `empty_plain_dict_inference.rs:3`) — zero new.
- New e2e fixture `crates/sifr/tests/e2e/pass/defaultdict_int_augassign_key_refinement.sifr` runs, exit 0.
- All four affected corpus fixtures (`0350`, `0621`, `0767`, `1481`) `check` clean and `run` exit 0; `0621`'s only output is the pre-existing `SIFR-TYPE-0901` overflow warning.

Behavioral validation of the specific fix (each verified head vs `8acf7ad01` vs base):
- **Child patches applied inside the child**: `lower_function_stmts` (`statement_dispatch.rs:812`) runs between the `mem::take` (`:707`) and the restore (`:816`), and `lower_stmts` applies patches per statement (`:185`), so the child's declaration is patched inside the child. Verified with 3-level nesting + a sibling nested function: outer `HashMap<String, i64>`, middle `HashMap<i64, i64>`, inner `HashMap<String, i64>`, sibling `HashMap<i64, i64>` — all four correct, program builds and runs (base emits four bare `HashMap::new()`).
- **Enclosing patch restored without staleness**: pass-2's F1 shape (shadow *after* the refining augassign, inside the same `if` body) now emits `let mut counts: HashMap<String, i64>` outer and `HashMap<i64, i64>` inner (`8acf7ad01` lost the outer; base lost both), and runs. There is no restore-staleness window in practice: a pending patch is consumed by the next `apply` in the declaring block, so it only survives while lowering inner blocks. The one way a child can invalidate it — `nonlocal` rebinding in the same inner block — was probed both compatibly (compiles and runs correctly at head, base loses the annotation) and conflictingly (head rejects with `SIFR-TYPE-0002` at the offending key; base's `check` passes and then fails in rustc `E0308`).
- **Same-scope rebinding still clears patches**: `patterns_and_assignments.rs:288/468/566-606` remove the entry on any rebinding, which also makes cross-function leftover leaks unreachable — I tried three leak constructions (tuple-unpack binding, aliased binding `counts = base`, module-global) and head emitted byte-identical Rust to base in each.
- **Either shadow source order**: shadow-before (pass-1 scenarios 1 and 2) and shadow-after (pass-2 F1) both correct; the scalar shadow emits `let counts: i64 = 7_i64` with no `HashMap` annotation.
- **Alias / missing-key / codegen preserved**: `defaultdict(list)`/`defaultdict(set)` emit identically to base; missing-key read stays `*counts.entry(...).or_insert(0)` and returns 0 at runtime; `entry(k).or_insert(0)` + `*__elem += 1_i64` unchanged.
- **Removed compound-statement recursion in `patch_stmt_container_specialization`**: declarations inside `while`/`for`/`if`/`try` bodies are still annotated at head (each inner block applies its own patches), confirmed by emit for all four block kinds.

### Non-blocking observations

- **N1** — The isolation intentionally discards patches a nested function discovers for an *enclosing* declaration, so the captured/`nonlocal` counter shape (`counts = defaultdict(int)` in the parent, `counts[w] += 1` only inside a nested function) reverts to base's bare `let mut counts = HashMap::new();` where `8acf7ad01` annotated it. Head == base here and both build and run (rustc infers `K`/`V` from the closure body), so this is an unfixed shape rather than a regression — but the ledger's "patches both declaration and constructor-call HIR" (`plans/issues/active/…-preexisting-failures.md:313`) does not hold for it and is worth qualifying.
- **N2** — Direct-only patching loses one pre-existing (non-defaultdict) target: a declaration inside a `try` body with the refining evidence *after* the block. `try: items = [] / finally: pass / items.append(n)` emits `let mut items: Vec<i64> = { … }` at base but `let mut items: Vec<Box<dyn ::std::any::Any>> = vec![];` at head and `8acf7ad01`. Not reachable today — `try`-body bindings never survive into valid generated Rust (both head and base fail with `E0425 cannot find value 'items'` for `try/except`, and `E0425 cannot find type 'Error'` for `try/finally`), and every other escaping-block shape is rejected at lowering (`SIFR-NAME-0001`). It becomes visible only if that `try`-scoping hole is later fixed.
- **N3** — Several shapes that base accepted at `check` and then failed inside rustc are now rejected with a proper `SIFR-TYPE-0002`: same-scope rebinding to a different key type, conflicting keys across `if`/`else` branches, and assigning a `dict[int, int]` to a refined counter. These match the pre-existing rule for plain dicts (identical diagnostic on both compilers for `counts = {}` shapes), so they are improvements to the "if it compiles, it works" guarantee, not regressions — but they do shrink the accepted-program set relative to base and aren't mentioned in the ledger.
- **N4** — Pass-1's N2/N3/N5 remain open (alias `type_args` preserved in the new refiner, discarded at `defaultdict_refinement.rs:106`; no `resolve_alias()` on the `Any`/`Unknown` guards at `:26,:30`; `entry(item.clone())` clones a `Copy` key). Pass-2's N2 (the shadowed `name` binding in the `container_literal_specialization.rs:298-303` guard) also stands. All cosmetic.
- **N5** — `plans/reviews/active/…-wave-4-claude-opus-review-pass-4.md` and its `.claude.log` exist as zero-byte artifacts from the timed-out run; they should be discarded or overwritten rather than left as apparent evidence.

APPROVED
