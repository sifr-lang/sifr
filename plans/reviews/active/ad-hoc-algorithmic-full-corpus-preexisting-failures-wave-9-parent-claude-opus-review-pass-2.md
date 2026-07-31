## Review — PR #3091 (Wave 9), head `22111f3f05fbcd89f1216ac80326f76240fa66a7`

Scope verified: base `44ab8ad38`, 9 files, +271/−7. No source file outside `crates/sifr_lowering/src/lower/statements/{nested_function_state.rs,statement_dispatch.rs}` changes behavior. Corpus gitlink unchanged at `9d715953`; the dirty submodule markers and the untracked closeout demo are untouched. Working tree ended byte-identical to the starting snapshot.

### Root cause and fix
Confirmed by `git log -S`: the unconditional `std::mem::take` / whole-map restore was introduced by `66cdaf108` ("preserve enclosing container patches"), merged in `f1c34cf9a` = **PR #3079 = Wave 4**. The ledger's attribution is exact.

The fix's lifecycle reasoning holds up:
- `restore_container_specialization_patches` runs at `statement_dispatch.rs:749`, i.e. *inside* the enclosing block's statement loop, so the `nested_function_captures` push from `statement_dispatch.rs:71` is still active and `get(func.name)` resolves to the correct entry. Nested-block pushes are restored at `:129` before returning, so a same-named inner `def` cannot corrupt the lookup (verified with a two-level same-name probe).
- Filtering by capture name gives isolation for free: a name that is local to the nested function is excluded by `capture_collection.rs:52`, so its patch is dropped exactly as before; a `nonlocal`-declared name is retained by the same line, so it propagates — which is correct.
- Multilevel relay works because the patch survives one lexical level at a time: it is not consumed at the intermediate level (`patch_stmt_container_specialization` only removes on a matching `Let`), then re-propagates because the intermediate function transitively captures the name.

### Independent verification (head vs. a freshly built base worktree)
| probe | base | head |
|---|---|---|
| direct capture (`t1`) / multilevel (`t2`) | `Vec<Box<dyn Any>>` | `Vec<String>` |
| unmodified `0022_generate_parentheses` | 4 errors | builds + runs, exit 0 |
| captured dict + set | `HashMap<Box<dyn Any>, …>` | `HashMap<String, i64>` |
| 3-level list + dict | `E0277`/`E0599` | builds + runs |
| `def` inside a `for` body | `Vec<Box<dyn Any>>` | `Vec<String>` |
| `nonlocal` + `append` | `E0308` ×2 | builds + runs |
| method-local capture in a class | — | builds + runs |
| param shadowing captured name | `Vec<i64>` | `Vec<i64>` (isolation held) |

Nothing regressed. Three probes fail identically on both sides and are pre-existing, unrelated to patch propagation (each also fails with explicit annotations or with 2 errors at base vs 1 at head): conflicting sibling appends into one captured empty list, an intermediate-level shadowed empty list, `nonlocal` *rebinding* (not appending) an empty list, and same-name nested functions at two levels (`E0061`).

Differential native build over all 17 corpus fixtures matching the "nested `def` + unannotated empty container" shape: only `0022` changed (4 errors → 0). `0106` and `0698` fail identically on both sides (helper-import artifacts of building outside the package root).

Regression sensitivity: both refinement codegen tests' `!contains("Vec<Box<dyn")` assertions fail at base; the shadow-isolation test correctly pins preserved behavior.

Gates re-run at exact head: lowering **944 passed / 1 ignored**, codegen **967/967**, full native e2e **687/687** with signature `d61c30dde1d7fc1c` (matching the claim exactly), clippy clean, `cargo fmt --check` clean, HIR maintainability guardrails PASS, `statement_dispatch.rs` at 833 lines.

### Actionable finding

**LOW — ledger accuracy.** `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:340`, Wave 9 row: "complete lowering passes 941/941". 941 is the *base* count (`44ab8ad38` measures 941 passed / 1 ignored); head measures **944 passed / 1 ignored**. The figure contradicts the same sentence's claim to add "three focused HIR tests", and it drops the ignored test that prior wave rows (e.g. Wave 5's "920 passed with one ignored") do record. The adjacent codegen figure `967/967` and the e2e `687/687` + signature are both accurate. Suggested wording: "complete lowering passes 944 with one ignored".

Everything else in that row checks out, including the reconciliation pass 1 asked for: the original 411-record inventory (20 `CHECK_FAIL` / 23 `BUILD_FAIL` / 368 `BUILD_PASS`, `0022` in the pass set) is consistent with the closeout measurement of 410 pass + 1 build failure once `0022` is named a Wave-4 regression rather than pre-existing residue, and the corpus-gate build-coverage gap pass 1 raised is now carried explicitly by the closeout row.

**Verdict:** approve on correction of the single low-severity ledger figure at line 340. No compiler, test, or scope changes required.
