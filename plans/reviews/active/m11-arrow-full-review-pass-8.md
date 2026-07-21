All verification complete and scratch probes removed. Here is the full report.

---

# M11 Arrow C Data Interface — full milestone review, pass 8 (`main...HEAD` at d727cd692)

Full-milestone review of the complete `main...HEAD` diff (142 files, +7,943/−1,126 over 8 commits, merge base 61155e200), the M11 plan section and acceptance criteria, and all prior review artifacts (passes 1, 2, 3b, 4, 5, 6, 7). Constraints honored: no worktree created or entered, no repository file modified (`plans/reviews/active/m11-arrow-full-review-pass-8.md` exists as an empty untracked stub and was left untouched; this report is stdout-only), `scripts/run_all_tests.sh` **not** run in either profile per instruction, scratch probes written to `/tmp/m11p8` (outside the repo) and removed after the review. All probes ran against a freshly built release compiler at d727cd692.

## Context: why pass 8 exists

Pass 7 (at 368b23606) was **non-final**: its Validation section shipped with an unresolved `MERGE-GATE-RESULT` placeholder (pass-7 artifact line 63), and the actual default merge gate at 368b23606 subsequently **failed** `test_emit_pass_fixtures_do_not_include_unwrap_or_expect` — the class-method borrowed-escape diagnostics introduced in 47a6c71d8/554afe692 rejected `crates/sifr/tests/e2e/pass/nominal_identity_alias_paths.sifr`, whose `JsonValue.__add__(self, other: StdJsonValue) -> StdJsonValue: return other` returns the borrowed non-affine operator RHS. Pass 7's SATISFIED verdict was therefore invalidated by the gate it left unresolved. Commit d727cd692 is the remediation; it is the entire delta since pass 7 (`git diff 368b23606..HEAD` touches exactly three files: `class_body_lowering.rs`, `parameter_conventions.rs`, `python_arrow_contract_tests.rs`, +12/−3). Every other pass-7 area finding — runtime/ABI/certification, evidence, docs, CLI, driver — is content-identical at d727cd692 and its audit conclusions carry forward, re-evidenced here by fresh suite runs at HEAD.

## The fix, mechanism-verified

`prepare_class_method_param_ownership` → `prepare_method_param_ownership(params, method_name, …)` (`parameter_conventions.rs:136-154`), with one added condition at line 149: a borrowed Move-ownership parameter is registered in `ctx.borrowed_params` only if `!is_operator_dunder(method_name) || param.ty.contains_affine_resource()`. Sole call site updated (`class_body_lowering.rs:548`). A regression pin was added: `borrowed_non_affine_operator_parameters_preserve_return_semantics` (`python_arrow_contract_tests.rs:513-519`), which fails if the fixture shape ever re-rejects.

Why this is sound, verified structurally and empirically:

1. **`ctx.borrowed_params` is diagnostics-only in lowering.** Exhaustive consumer sweep: `return_lowering.rs:64` (non-affine return escape, OWN-0003) and `:142` (affine return escape, PYZC-0001); `patterns_and_assignments.rs:448` (store escape); `tuple_unpack.rs:113-133` (unpack escape); `task_scope_calls.rs:165` (task-boundary escape); `core_and_calls.rs:564/:598` (affine consume/owned-argument escape — both **affine-gated**); `python_arrow_methods.rs:52` / `python_buffer_methods.rs:96` (explicit release of borrowed resource — affine types only). Removing a name from the set can only suppress rejections; it never alters emitted code. The codegen-side `borrowed_params` (`lib_emitter_state.rs:77`, populated independently from parameter conventions at `class_method_emitter.rs:613`) is untouched.
2. **The exemption is exact main parity.** On main, class methods never populated `ctx.borrowed_params` at all (grep of main's `class_body_lowering.rs`: zero hits); `return_lowering.rs` and the store-escape check pre-existed unchanged (branch diff of `return_lowering.rs` is empty; `patterns_and_assignments.rs` differs only by a message string at `:529`). The branch *added* class-method tracking; the fix carves out exactly the non-affine operator-dunder subset, restoring main's behavior for that subset only.
3. **The exemption is coextensive with the clone-on-return emission path.** The identical `is_operator_dunder` predicate (`diagnostics.rs:261-262`, file byte-identical to main; list excludes `__pow__`/`__getitem__`) routes those methods to `operator_impls` (`class_body_lowering.rs:625`), emitted by `operator_protocol_emitters.rs` (zero branch diff), whose Return arm wraps borrowed non-Copy returned names in `RustExpr::Clone` (`operator_protocol_emitters.rs:448-455`). Confirmed in emitted output: `impl Add<&Value> for &Identity { fn add(self, other: &Value) -> Value { other.clone() } }`.
4. **Affine resources can never traverse the exempted path.** The exemption requires `!contains_affine_resource()`; owned affine operator params are declaration-rejected before any body lowering (`parameter_conventions.rs:90-101`); borrowed affine operator params remain tracked and every escape position rejects (probes below). No move/copy of an Arrow or Buffer resource is reachable through this change — the only behavioral loosening is on types with no release semantics, and its worst case (a store that codegen moves out of a borrow) fails loudly at rustc, producing no binary.

## Explicitly requested verifications (all live at d727cd692)

| Requested check | Result | Evidence |
|---|---|---|
| Ordinary `def echo(self, value: str) -> str: return value` still rejects | **PASS** | `SIFR-OWN-0003: cannot return borrowed parameter 'value'` at check |
| Borrowed Arrow operator return rejects before codegen | **PASS** | `__add__(self, other: python.ArrowArray) -> python.ArrowArray: return other` → `SIFR-PYZC-0001` at `check`; `emit` refuses (diagnostic, no Rust output) |
| Borrowed Arrow operator *store* escape rejects | **PASS** | `keep = other` in `__add__` body → `SIFR-PYZC-0001` ("cannot pass as an owned argument borrowed affine…") |
| Borrowed Buffer operator escape rejects | **PASS** | `python.Buffer[uint8]` store variant → `SIFR-PYZC-0001` |
| Owned affine operators remain declaration-rejected | **PASS** | `def __add__(self, own other: python.ArrowArray)` → `SIFR-PYZC-0001` "operator 'Wrap.__add__' cannot consume affine parameter"; unit matrix `affine_consuming_operator_parameters_fail_closed` (×Arrow/Buffer × `__add__`/`__pow__`/`__getitem__`) green |
| Nested affine shapes remain rejected | **PASS** | `own other: list[python.ArrowArray]` → declaration `PYZC-0001`; borrowed `other: list[python.ArrowArray]` return → escape `PYZC-0001` |
| Ordinary-method borrowed *affine* return rejects | **PASS** | `def take(self, value: python.ArrowArray) -> python.ArrowArray: return value` → `PYZC-0001` |
| `nominal_identity_alias_paths.sifr` emits | **PASS** | `emit` exit 0, zero `unwrap(`/`expect(` occurrences in output; fixture also **runs** end-to-end (all asserts, exit 0) |
| Full emitted-code safety test passes | **PASS** | `test_emit_pass_fixtures_do_not_include_unwrap_or_expect`: ok, all 675 pass fixtures compiled and scanned, 16.05s |
| No unsound move/copy path opened | **PASS** | Structural argument (points 1–4 above) plus adversarial probe: store-then-return of the borrowed non-affine RHS check-passes but fails **loudly** at build (`SIFR-BUILD-0005`, rustc move-out-of-borrow, no binary) — exact main parity, never a miscompile, unreachable for affine types |

Additional adversarial probes: `__getitem__(self, key: str) -> str: return key` correctly still rejects OWN-0003 (`__getitem__`/`__pow__` are not in `OPERATOR_DUNDERS`, are emitted as inherent methods without clone-on-return, and are correctly *not* exempted — the exemption tracks the emission routing exactly). The exempted operator run end-to-end via `i + v` returned the cloned RHS with correct field values.

## Pass-7 and carried finding dispositions

- **Pass-7 artifact itself** — non-final: unresolved `MERGE-GATE-RESULT` placeholder; the merge gate it claimed to have run actually failed at 368b23606. Its technical audit content (runtime/ABI/certification, evidence/docs, async ownership matrices) remains valid — those surfaces are byte-identical at d727cd692 — but its verdict is superseded by this pass. The failure it missed is fixed and regression-pinned.
- **Pass-7 N1** (while-*condition* affine move escapes the loop check; pre-existing on main, loud E0382/BUILD-0005) — **carried, unchanged**; `statements/control_flow.rs` untouched by the delta. Non-gating.
- **Pass-7 N2** (protocol-class declarations bypass the owned-affine operator guard; soundness-inert) — **carried, re-reproduced live this pass**: `class P(Protocol): def __add__(self, own value: python.ArrowArray) -> int: ...` still checks clean. All escalation paths remain independently rejected (per pass-7 verification, surfaces unchanged). Non-gating; follow-up recommendation stands.
- **Pass-7 N3** (plan header claims the waves "pass the authoritative create-PR and merge gates", `ad-hoc-declaration-first-python-interop.md:42-43`) — **carried and sharpened**: the claim was factually false at 368b23606 (merge gate failed there) and at d727cd692 the merge gate has not yet run. Must be resolved by the fresh gate run plus the planned merge-time header rewrite. Tracking-only; non-gating.
- **Pass-7 advisories A1–A11** — all carried unchanged (none of their surfaces are in the delta). A11 note refreshed: `class_body_lowering.rs` is now 896 lines (was 896-class; still under the 900 cap; file-size guardrail PASS over 2,744 files).
- **Pass-6 carried minors** (imported-class dunder class-style calls; latent instance-method-export gaps masked fail-closed; cross-module inherited-not-redeclared CLASS-0004; `class_method_origins` not exported; genexpr-call-iter E0425; driver `>= 2` OWN-count assertion; near-cap files; dirty `third_party/ruff`) — carried unchanged. `third_party/ruff`: still a single unstaged local modification, gitlink identical to main (`git diff main HEAD -- third_party/ruff` empty) — must not be staged at merge.
- **Earlier closures** (pass 1 B1–B7, pass 2 NB-1/NB-2/M-1–M-4, pass 3b B-1–B-3, pass 4 NB-A–C/M-A–D, pass 5 NB-D/M-E/M-F/file splits, pass 6 F1) — stand; every file they touch is unchanged since pass 7's re-audit, and the crate suites covering them pass at d727cd692 (below).

## Acceptance criteria

All three criteria were independently re-verified by pass 7 at content identical to d727cd692 for every cited runtime/certification/evidence path; this pass re-confirms the static-ownership criterion with fresh probes and re-runs the covering suites at HEAD:

- **No copy switch / never certify uncertain copying — MET** (decorator grammar `python_interop/arrow.rs:157-193`; four-layer copy-evidence rejection `arrow_certification.rs:144-166`, `python_cli.rs:294-379`, `build/python_interop.rs:98-135`, `arrow_ops.rs:193-211` — all unchanged since pass 7; `sifr_runtime --features python` 221/221 at HEAD).
- **Ownership transfers exactly once, stays moved on consumer failure — MET** (static matrices: `sifr_lowering` 812/812 including the full arrow contract suite and the new pin; the ten negative probes above; runtime commit-before-call and error-preserving reconcile unchanged, covered by the passing runtime suite).
- **Unconsumed resources release exactly once — MET** (capsule destructor single release point, `SifrPythonClosedArrowCapsule` on double release, instrumented exact-count tests — all in the 221 passing runtime tests at HEAD; evidence-side exactly-one-release gate unchanged).

## New findings

**Blockers** — none. **Majors** — none.

**Minors**

- **P8-1 — the exemption is parameter-scoped, not return-position-scoped, so operator bodies lose the new *store*-escape diagnostic for non-affine params (main parity; loud; never unsound).** `def __add__(self, other: Value) -> Value: keep = other; return keep` checks clean and fails at build with `SIFR-BUILD-0005` (rustc move-out-of-borrow; no binary produced). This is byte-for-byte main behavior (main never tracked class-method borrowed params; `patterns_and_assignments.rs:448` pre-existed for functions only), affects only non-affine types (the affine branch of the condition keeps Arrow/Buffer tracked — probed), and can never miscompile. Recommended follow-up, not gating: scope the exemption to the return position (the only position operator codegen clones), keeping `borrowed_parameter_store_escape` active inside operator bodies, with fixture-corpus validation.

**Advisories**

- **P8-A1** — `test_emit_pass_fixtures_do_not_include_unwrap_or_expect` asserts `total > 0` before reporting accumulated failures (`e2e_entrypoints.rs:176`), so an environment-level sysroot-resolution failure surfaces as the misleading "no pass fixtures were checked" instead of the real per-fixture errors (reproduced: without `SIFR_SYSROOT` in a bare shell the test fails in 0.01s with that message). Pre-existing on main (file has zero branch changes); fail-closed either way.
- **P8-A2** — `cargo clippy -p sifr_lowering --all-targets -- -D warnings` fails with 27 test-cfg pedantic lints (e.g. `semicolon_if_nothing_returned`), all in test files untouched by this branch; lib-target clippy (the documented gate form) is clean. Pre-existing, non-gating.

## Validation performed

- Built the branch compiler at d727cd692 (release, clean). Eleven live probes through `check`/`emit`/`run` covering the full requested matrix plus adversarial extensions (store-position escape, `__getitem__` non-exemption, protocol carried gap, exempted-shape runtime execution, fixture end-to-end run).
- Focused suites at HEAD: `sifr` e2e-entrypoint `test_emit_pass_fixtures_do_not_include_unwrap_or_expect` **ok** (675 fixtures — the exact test that failed the pass-7 merge gate); `sifr_lowering` **812/812** (includes the new regression pin and the full arrow contract suite); `sifr_codegen` **878/878**; `sifr_runtime --features python` **221/221**.
- Guardrails: `check_hir_maintainability_guardrails.py` **PASS**; `check_file_size_guardrails.py` **PASS** (2,744 files); `cargo fmt --check -p sifr_lowering` clean; `cargo clippy -p sifr_lowering -- -D warnings` clean.
- Provenance checks: delta since pass 7 is exactly 3 files; `diagnostics.rs`, `return_lowering.rs`, `operator_protocol_emitters.rs` byte-identical to main; ruff gitlink unchanged vs main with one unstaged local hunk.
- **Merge-gate status: pending a fresh post-review run.** Per instruction, neither `scripts/run_all_tests.sh` profile was executed in this pass, and no completed gate evidence exists at d727cd692 (the last known runs are create-pr **pass** and merge **fail** at 368b23606). The single merge-gate failure cause is verified fixed here; nothing else in the gate's input set changed since the passing create-pr run except the three delta files, all covered green above.

## Merge-time bookkeeping (not gating)

At merge: run and record the fresh default merge gate at d727cd692 (or the then-HEAD); check the milestone-review box (plan `:1665-1667`) and the top-level M11 box (`:180`); rewrite the header narrative (`:42-45`) to cite the gate runs that actually completed (resolves N3); commit the pass-7 artifact only with its placeholder corrected to the true 368b23606 result, alongside this pass-8 report; do not stage the dirty `third_party/ruff` submodule.

## Bottom line

The single commit since pass 7 fixes the true merge-gate failure at its root: the new class-method borrowed-escape tracking was over-broad for operator dunders, whose emission path borrows the RHS and clones non-Copy borrowed returns, making the escape rejection a false positive there. The exemption is precisely scoped — same predicate as the emission routing, affine-containing parameters explicitly excluded, ordinary methods and `__getitem__`/`__pow__` unaffected — and every consequence was verified live: the previously failing fixture emits and runs, the failing gate test passes over all 675 fixtures, all ten requested negative shapes still reject with the right codes before codegen, and the only permissiveness introduced is exact main parity on non-affine types with a loud build-time backstop. No blocker or major exists; carried findings are all pre-existing, inert, or tracking-only. M11 is ready for the fresh default merge gate.

VERDICT: SATISFIED
