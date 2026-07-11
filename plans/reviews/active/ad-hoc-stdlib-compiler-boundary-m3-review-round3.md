I have enough evidence. Let me finalize the verdict.

# M3 Review — Round 3 Verdict

## Verdict: **SATISFIED**

The round-2 shadowing advisory has been closed correctly. The reconciliation is well-placed, cannot erase canonical source declarations, and prevents spelling-based dispatch after local shadowing.

## Reconciliation correctness (trace)

Placement (`crates/sifr_lowering/src/lower/mod_impl.rs:770-775`):

- Runs **after** every writer to `ctx.compiler_intrinsics` from imports:
  - Early import pass (`imports.rs:161`, at `mod_impl.rs:68`)
  - Sysroot `sifr.*` import pass (`mod_impl.rs:365-373`)
  - Private `_sifr.*` import pass (`private_stdlib_imports.rs:65-67`, entered from `mod_impl.rs:283`)
  - Local-module import pass (`mod_impl.rs:595-603`)
- Runs **before** every reader that matters:
  - `HirFunction.compiler_intrinsic` set in `annotations_and_function_lowering.rs:541` (second lowering pass)
  - Intrinsic dispatch in `expressions/regular_calls.rs:501`
  - First-class value rejection in `expressions/core_and_calls.rs:215`

Cannot erase canonical declarations:

- The snapshot at `mod_impl.rs:98-103` reads `ctx.compiler_intrinsics[function_name]` **immediately after** `register_declaration` for the same function; because `register_declaration` writes the canonical id last (`compiler_intrinsics.rs:49`), the snapshot captures the canonical id even if an early import had populated the same key first.
- The reconciliation removes each local function-name key once, then `extend`s the snapshot — canonical is always restored under its own name.
- Even under duplicate declarations (`note_module_decl` returns false for the dupe), `remove` is idempotent and the snapshot only ever contains the first (canonical) entry.

Prevents spelling-based dispatch after local shadowing:

- Trace for the new unit test `local_function_declaration_shadows_unaliased_imported_intrinsic_identity` (source has `from sifr.test import assert_eq` followed by local `def assert_eq`):
  1. Early import inserts `assert_eq → TestAssertEqual`.
  2. First function pass: `register_declaration` no-op (no decorator); snapshot skipped (`has_decorator_syntax=false`).
  3. Full import pass re-inserts `assert_eq → TestAssertEqual`.
  4. Reconciliation removes `assert_eq`; extend is empty. Map is empty.
  5. Second pass lowering `check()` sees `func_name="assert_eq"`, `ctx.compiler_intrinsics.get("assert_eq")=None` → emits `HirExpr::Call`, not `IntrinsicCall`. ✓ matches the test assertion at line 263-268.

Canonical case unaffected (`sifr.test` module lowering):

- 7 `@compiler_intrinsic`-decorated defs each populate the snapshot with `(name, canonical_id)`.
- Reconciliation clears all 15+ local function-name keys, then extend restores exactly the 7 canonical pairs.
- Helpers like `assert_vector_eq` (which call `assert_eq` internally) still see `ctx.compiler_intrinsics["assert_eq"]=TestAssertEqual` and lower to `IntrinsicCall`. ✓

## Regressions from round-1/round-2

- **Round-1 #1 doc drift** — resolved in round-2, still absent from HEAD (no `_sifr.task → task.sifr` row).
- **Round-1 #2 / round-2 #1 ranges** — still scaffolding; not a defect.
- **Round-1 #3 / round-2 #2 duplicate diagnostic on malformed `@compiler_intrinsic`** — cosmetic; `malformed_unknown_synthesized_and_runtime_body_declarations_are_rejected` still passes with `contains`-matcher.
- **Round-1 #4 prescan intrinsic-name path** — unchanged; still ordering-correct.
- **Round-2 #3 shadowing edge case (advisory)** — CLOSED by the reconciliation and the new unit test.
- **Round-2 #4 counter callables count** — informational only.

## Non-blocking notes

1. **Snapshot on error paths** (`mod_impl.rs:99-102`) — If `register_declaration` fails AND `has_decorator_syntax=true` AND an early import had populated `ctx.compiler_intrinsics[function_name]`, the snapshot captures the *imported* intrinsic id and the reconciliation later restores it under the erroring name. This is silently benign because `register_declaration` also pushes to `ctx.errors`, so lowering returns `Err` and the polluted map is discarded. Could tighten by only snapshotting when the local `register_declaration` actually succeeded (e.g., have `register_declaration` return the inserted id), but not a milestone-blocking defect.
2. Ranges captured but unread (carryover from round-2 #1) — scaffolding.

## Remaining validation gaps

- Focused `cargo test -p sifr_lowering --lib compiler_intrinsics --locked` re-verified locally: **7/7 pass**, including the new `local_function_declaration_shadows_unaliased_imported_intrinsic_identity`.
- Authoritative `scripts/run_all_tests.sh --profile create-pr` remains pending only because of the reported macOS `syspolicyd/fseventsd` saturation — environmental, not a waived gate. **M3 is ready to merge once the unchanged create-PR gate exits zero.**
