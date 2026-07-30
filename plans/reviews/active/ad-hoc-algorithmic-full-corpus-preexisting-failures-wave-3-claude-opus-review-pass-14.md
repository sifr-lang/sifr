## Wave 3 review — pass 14 (exact head `7dbe8bd36` vs base `ca7731aa8`)

### Merge integrity — verified not evil

- `7dbe8bd36` has parents `efb3eba15` (branch) + `ca7731aa8` (main). Both directions of the merge are exactly the respective side's changes, which is the definitive non-evil check:
  - `git diff ca7731aa8...7dbe8bd36` = 25 files, all Wave 3 (`crates/**` + `plans/**`), 1072/69.
  - `git diff efb3eba15 7dbe8bd36` = 59 files, all main's Rust-backend-ecosystem certification (`sifr_driver/src/build/rust_interop*`, `rust_interop_plan.rs`, `sifr_package` interop tests, `verification/areas/rust_interop/**`, interop docs).
- **Zero file overlap** between the two sides. The only shared crate is `sifr_codegen`, and main's touch there is `rust_interop_plan.rs` adding a `sqlx_offline_metadata_digest` field to the cargo-inputs plan digest — no contact with `function_emitter/**` or any typing path. Current-main changes cannot perturb the empty-plain-dictionary mechanism.
- Scope hygiene: `git diff --name-only` outside `crates/`+`plans/` is empty; `git diff --check` clean; no `.gitmodules`, submodule pointer, compatibility-matrix, stable-claim, or profile changes — consistent with the issue's constraint at `plans/issues/.../ad-hoc-algorithmic-full-corpus-preexisting-failures.md:301-303`.

### Prior approved findings remain resolved (own runs at this head)

- Pass 1 §1/§2 leakage: `distinct_scoped_maps` / `loop_scoped_maps` shapes check, build, and run; the two codegen tests pin `HashMap<String, i64>` and `HashMap<i64, i64>` side by side.
- Pass 4 widening: assignable-but-unequal writes still gated by `statement_dispatch.rs:128-131` exact-shape `retain`; `SIFR-TYPE-0008` preserved.
- Pass 6 augassign: `disqualify_exact_dict_writes` (`state_collection.rs:766-770` call site) sticky and merged through `merge_exact_dict_writes`; missing-key loop still `SIFR-TYPE-0005`.
- Pass 12 §1 self-label: `…pass-11.md:3` reads "pass 11". Pass 13 approved that correction and its report is preserved verbatim.
- Nearest-declaration patching (`container_literal_specialization.rs:273-278`) is sound: patches drain after every statement (`statement_dispatch.rs:184-188`), so reverse-iterate-and-`remove` always resolves the innermost/most-recent `Let`. I could not construct a mis-patch across sibling `if`, loop, `try`, `match`, or nested-function bodies.
- Mechanism probes I ran fresh (all correct): `match`-arm writes, `while` with `str` values, `del`, sibling nested-function scope, dict-of-list values, 2-let ineligibility (`count != 1` → base path), if/elif/else uniform shapes, read-before-write two-sum.

### Independent validation at `7dbe8bd36`

| Check | Result |
|---|---|
| `cargo test -p sifr_lowering --lib` | 898 passed, 1 ignored — matches ledger exactly |
| `cargo test -p sifr_codegen --lib` | 934 passed — matches ledger exactly |
| `cargo clippy --workspace -- -D warnings` | exit 0 |
| `cargo fmt --check` | clean |
| `check_hir_maintainability_guardrails.py` | PASS |
| Touched-file sizes | 890 / 866 / 858 / 735 / 350 / 162 / 109 / 47 — all < 900 |
| `empty_plain_dict_write_inference.sifr` | build + run exit 0 |
| `0001_two_sum.sifr` (corpus) | build + run exit 0 |
| Working tree | byte-identical to session start; no files modified |

### Actionable findings

None.

### Non-blocking observations

1. **Ledger currency.** `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:317-328` still names `ec5aab945` as "Wave 3's exact prospective merge" and `ea119724e` as "current `main`"; the actual prospective merge is `7dbe8bd36` over main `ca7731aa8`, and neither pass 13's approval nor the create-PR revalidation on this head is recorded. Classified non-blocking for the same reason pass 11 filed its observation #6 that way — and because requiring the ledger to name its own head commit is self-invalidating. Fold into the same edit that flips the row to `merged`.
2. **Pre-existing: `empty_dict_specializations` is never scoped per function**, so a stale entry produces a nonsensical diagnostic in an unrelated function. Probe: `def a(): data = {}; data[1] = 2` followed by `def b(mut data: dict[str,int]): data[1] = 5` reports `SIFR-TYPE-0008 empty literal type conflict for 'data': expected key 'int' and value 'int', got key 'int' and value 'int'`, where the same `b` alone correctly reports `SIFR-TYPE-0002 … key type 'int' … not compatible with … 'str'`. The insertion site responsible (`container_literal_specialization.rs:90`) is untouched by this diff and reaches the identical state on base for the same program, so the wave's new insert at `control_flow.rs:440` neither introduces nor widens it. Follow-up issue material.
3. **Pre-existing: module-level empty dict.** `table = {}` at module scope with `table[1] = 2` inside a function emits `fn __const_table() -> HashMap<Box<dyn Any>, Box<dyn Any>>` → three rustc errors from a check-clean program (and each call returns a *fresh* map, a separate semantic bug). Ineligible for the new path by construction, so unchanged; worth its own issue.
4. **Pre-existing: nested empty-dict value.** `data[1] = {}` emits `HashMap<i64, HashMap<Box<dyn Any>, Box<dyn Any>>>` → E0277. Adoption is correctly refused (hint contains `Any`), so this is the untouched specialization path.
5. **Pre-existing: captured-dict mutation.** `data = {}` + nested reader + later write yields E0502; reproduces identically with a fully annotated `data: dict[int, int] = {1: 2}`, so it is orthogonal to this wave.
6. **`statement_dispatch.rs` is at 890/900 lines** after +11. Compliant, but the next touch to that file will likely need the split.
7. **`local_binding_registry.rs:8-14`'s ambiguous-name drop** also shrinks `shadowed_module_bindings` passed to `register_sifr_int_forced_local_bindings` (`scope_and_function_types.rs:96`). Only reachable when one name has two `Let`s of differing types — precisely the shape where the old first-wins map was already wrong — and every consumer degrades to `expr.ty()`. Strictly conservative.
8. `cargo clippy --workspace --all-targets -- -D warnings` fails in `sifr_ipc` (lib test, `expect_used` ×3). Untouched crate, and not the project's canonical command; the canonical invocation is clean.
9. A tracked 3-byte junk file `crates/sifr/tests/e2e/pass/Untitled` exists from main (`c9e5aba729`), unrelated to this PR.

The merge is clean and mechanically inert with respect to the empty-plain-dictionary path, all previously approved corrections hold at this exact head, and every lane I re-ran matches the claimed numbers.

APPROVED
