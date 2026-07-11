I've completed the review. Here's my assessment.

# Final Phase-Wide Closeout Review — Ad Hoc Stdlib and Compiler Boundary Rearchitecture

**Scope**: `origin/main..HEAD` (working tree) — five commits from `bf2ad72d0` through `da5dd9c27` plus the untracked closure delta: Cargo.toml (+4), verification/areas/sysroot_release/runner.py (+1), three doc updates, three review placeholders, and roadmap.

## 1 — Architecture and permanent guards

Every phase acceptance criterion holds under executable enforcement at HEAD. Independently re-executed:

- `python3 scripts/check_stdlib_native_adapter_reachability.py` → PASS: `public_adapters=403, active_rust_targets=399, cross_module_substrates=4`. Matches the `399 + 4` classification claimed in `plans/issues/active/ad-hoc-stdlib-compiler-boundary-m6-plan.md:44-46`.
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py` → PASS: `exact_intrinsics=17, registry_files=8, preamble_files=9, lowering_files=8, codegen_files=4, retained_direct_dependency_packages=6, direct_runtime_roots=2`. Matches m6 plan and `internal_docs/stdlib_retained_compiler_intrinsics.toml` surface enumeration exactly.
- `python3 scripts/check_stdlib_manifest_schema.py` → PASS: `surfaces=10, schema_version=2, final_state=retained-by-design`.

Typed HIR identity verified structurally:

- `crates/sifr_ir/src/hir_nodes.rs:118-136` — `CompilerIntrinsicId` has exactly 17 variants (7 test asserts + 2 open + 3 bytes constructors + 4 encoding + 1 task-context).
- `crates/sifr_ir/src/hir_nodes.rs:517-523` — `HirExpr::IntrinsicCall { intrinsic, args, ty, call_range, arg_ranges }`.
- `crates/sifr_codegen/src/intrinsics/registry.rs:25-69` — total match over all 17 variants with no `_ =>` fallback; no raw-name lookup.
- `crates/sifr_ir/src/hir_nodes.rs:107` — `HirFunction.compiler_intrinsic: Option<CompilerIntrinsicId>`; `FunctionType` remains signature-only (spot-checked).

Boundary residue verified absent:

- `rg 'sifr_retained_intrinsics' crates Cargo.toml Cargo.lock` → 0 hits.
- `rg 'infer_dependencies|infer_dependency' crates/ verification/ scripts/` → 0 hits.
- `rg 'intrinsic_io|retained-fallback-signature-glue|resolve_retained_fallback|re_export_intrinsic_fallbacks|get_intrinsic_module' crates/` → 0 hits.
- `rg 'counter_' crates/sifr_ir/src crates/sifr_codegen/src/intrinsics crates/sifr_stdlib_manifest/src` → 0 hits; residual matches under `crates/sifr_codegen/src/stmt_support_emitter/expr_call_and_literal_helpers.rs` are typed-defaultdict generated variable names (`__sifr_counter_chars`, etc.), not intrinsic identifiers, and are consistent with the retained `typed_defaultdict_language_semantics` surface.
- `stdlib/_sifr/{io,test,task}.sifr` — absent (verified via `ls stdlib/_sifr/`); `stdlib/_sifr/runtime.sifr` remains as the diagnostics bridge declaration.

## 2 — Post-gate corrections

### Round 4 — dev-profile `opt-level = 1` for `sifr_lowering` and `sifr_type_system`

`Cargo.toml:197-200` adds two per-package dev overrides. Confirmed:

- No override of `debug-assertions`, `overflow-checks`, `debug`, `lto`, or `codegen-units` for either package — workspace dev defaults (asserts on, overflow checks on, full debug info) are preserved.
- Follows the identical existing precedent for `ruff_python_parser` at `Cargo.toml:195-196` (the corresponding hot-crate on the CLI path). Not a novel mechanism.
- LLVM `-O1` (mem2reg, basic inlining, dead-code elim) is not a safety-off knob; it does not disable UB checks or reorder observable side effects. The `unsafe_code = warn` workspace lint at `Cargo.toml:151` and clippy pedantic already gate against latent UB.
- No performance-budget waiver in `verification/policy/` (spot-checked); representative performance evidence rows in the plan claim unchanged baselines and budgets.

Not a masking correction. Correct scope. No blocker.

### Round 5 — `timeout=1200` on `emit migrated stdlib smoke`

`verification/areas/sysroot_release/runner.py:325-333` adds `timeout=1200` to a single `run_checked(...)` call. Confirmed:

- Sits between two 1200 s peers (`runner.py:204` boundary equivalence build; `runner.py:419/428/438/492` heavy-installed check/emit/build/cargo-offline). It matches the existing bound already applied to comparable cold-bridge-probe-cache work in this same file.
- The smoke uses a fresh temp sysroot (`runner.py:273-294`) without a pre-warmed probe cache, so its first `emit` pays the full cache-population cost — the same cost the boundary equivalence lane already absorbs.
- Cheap adjacent installed calls retain their smaller budgets (sysroot JSON default 120 s; self/doctor helpers at defaults; LSP smoke at 60 s). Path-leakage scans at `runner.py:357,372` remain at 120 s — behavioral/path-leakage assertions are not weakened.
- The behavioral assertion at `runner.py:334-335` (`"sifr_stdlib" not in emit.stdout`) still fires, and a genuine hang would still trip at 1200 s (`exit 124`), preserving detection.

Bounded, narrowly targeted, and consistent with the file's existing policy. No blocker.

## 3 — Closure documentation

- `plans/issues/active/ad-hoc-stdlib-compiler-boundary-rearchitecture.md:5-66` updates status to "Completed and audited on 2026-07-11", records the merge-gate evidence table (3,416.62 s wall time; 375,729 ms boundary equivalence; 160,109 ms installed smoke; 650/650 E2E; 261 hardening variants), lists rounds 4 and 5 in the Implementation Closeout Review Record, and flips the final-gate checklist item.
- `plans/issues/active/ad-hoc-stdlib-compiler-boundary-m6-plan.md:38-86` flips the final-gate checkbox and appends the corrective evidence rows.
- `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:1-20` moves the native-boundary phase to completed/recertified status and cites PR #2928 as the closure record.
- `plans/roadmap.md` row 39.2 flipped to "Completed and audited".

Cross-references and counts internally consistent: rounds 1-3 (phase-wide) + rounds 4-5 (corrective) = 5 satisfied review rounds. Round 6 (this review) is called out as the empty file at `plans/reviews/active/ad-hoc-stdlib-compiler-boundary-phase-closeout-round6-final.md` (0 bytes) — it is the artifact this verdict populates.

## 4 — Risk, tests, maintainability

Public API additions in `crates/sifr_driver/src/lib.rs` remain additive (`generate_dependency_cargo_toml`, `sysroot_cargo_config_args`, `try_generate_standalone_dependency_plan`, `InteropBuildPlan`), and no user-triggerable panic surface was introduced in the M1 diagnostic bridge (`crates/sifr_stdlib/src/runtime_observability.rs`, previously verified panic-clean). File-size, HIR maintainability, bootstrap-ordering, format, and diff guards are all reported PASS in the plan's final merge-gate evidence and align with the working-tree state.

## Optional (non-blocking) observations

1. The 1200 s timeout constant is now repeated at 11 sites across `verification/areas/sysroot_release/runner.py`. Extracting a named constant like `INSTALLED_HEAVY_TIMEOUT_S` would improve future intent-preservation, but is out of scope for phase closure.
2. `plans/reviews/active/ad-hoc-stdlib-compiler-boundary-phase-closeout-round6-final.md` is a 0-byte placeholder — the file exists but this verdict is the artifact that populates it, so it should be filled with this review's content in the closure commit.
3. `plans/reviews/active/ad-hoc-stdlib-compiler-boundary-phase-closeout-round4-performance-corrective.md` opens with a top-level `## Round 4 corrective review` rather than a level-1 heading; rounds 1-3 use `#`. Cosmetic Markdown consistency, not a blocker.
4. The rearchitecture doc header now says "Completed and audited on 2026-07-11" and the roadmap says "Completed and audited" while this round-6 review is what confirms audit closure. The pattern is documented in the checklist row ("the identical merged `main` tree is revalidated after closure PR #2928 lands"), so this is anticipatory but honest — no blocker.

## Blockers

None. The two post-gate corrections (opt-level=1 for two hot compiler crates, +1200 s for the cache-cold installed emit) are the minimal, targeted, precedent-following responses to real signals surfaced by the exact `scripts/run_all_tests.sh` gate. Neither weakens safety, budgets, behavioral assertions, or path-leakage validation. Guard set-equality, adapter reachability, negative self-tests, installed/source-tree equivalence, and typed-dispatch exactness all hold at HEAD. The phase acceptance criteria are met with executable enforcement, not prose.

VERDICT: SATISFIED
