# Rust-Interop `certification_9` (`native_build_script`) — Exact-Head Review

**Head:** `b5497901d4d7c7d90a65d03402708f6642e913ea` · **Base:** `origin/main` = `f188986482e` (the `certification_8` merge, PR #3067) · **PR:** #3069 (`DRAFT`) · 2 commits, 36 files, +1865/−98.

Scope note: the local `main` ref was stale by one merge commit; `git diff main...HEAD` therefore misleadingly shows 65 files including all of `certification_8`. All judgements below use the true merge base `f18898648` / `gh pr diff 3069`. Shared-worktree changes outside the commits (`editor_integrations`, leetcode corpus, `.cert5probe/`, `.agent/`, stray `*.webp`, `plans/phases/43_interoperability.md`, the untracked round-3 placeholder) are excluded as specified.

## Independent verification performed

**Out-of-scope backend hunk is excluded — confirmed two ways.**
- The PR's only hunk in `rust_interop_compatibility_matrix.json` is the `native_build_script` row (`:422-432`): `category` `future-owned-by-separate-phase` → `supported`, `future_owner` dropped, both evidence directions `planned` → `passing`, note rewritten. The worktree's unstaged promotion of `ecosystem_backend_certification` (`:396-400`) is absent from the diff; `grep ecosystem_backend` over `gh pr diff` hits only prose, review-file text, and unchanged checker context — no data-row change.
- Behavioural proof, not just textual: running `check_compatibility_matrix.main()` with `COMPATIBILITY_MATRIX_PATH` monkeypatched to the committed HEAD blob (no repo file touched) → `rust interop compatibility matrix ok: rows=36 fixture_rows=36 categories=4`, exit `0`. The same checker against the worktree fails with `ecosystem_backend_certification: supported rows require passing positive and negative fixture evidence`. The full area run reproduces `variants=10, failures=1, blocking_failures=1` with that single failure and every other case green (`fixtures=36 diagnostics=10 crates=44 package_examples=60 scenario_examples=18`, matrix self-test `cases=166`, `tiers=5 fixtures=36`, stale-draft `cases=20`, `claims=32` + `cases=33`). The PR itself is all-green for that variant; the failure is entirely attributable to the excluded hunk.

**Inventory recomputed from committed data** (not from the plan's prose): 36 compatibility rows, 36 fixture rows, 32 structured claims; categories 19 `supported` / 12 `supported-through-bridge` / 4 `future-owned-by-separate-phase` / 1 `unsupported-by-design`; execution kinds 13 `cargo-probe` / 10 `contract-only` / 9 `runtime-observed` / 4 `compiler-diagnostic`; evidence 64 `passing` / 8 `planned`. Exactly matches the plan's "Expected post-item inventory" (`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:1053-1064`) and the sysroot-doc counts (`internal_docs/sifr_sysroot_and_stdlib_architecture.md:155-158`, `18→19` supported / `5→4` future-owned, with `native_build_script` correctly removed from the separately-owned list). `stable_support_claims.json` and the generated `docs/rust-interop.mdx:245` table row agree (`supported` / `cargo-probe`).

**Post-round-2 commit `b5497901d` is accurate.** Its two hunks are: the status row → `in review` with the PR #3069 link, and one new validation paragraph. Each claim checked:
- "all 19 Python-interop variants" — `verification/profiles/create-pr.json` `selected_areas` lists exactly 19 `python_interop` suites (counted programmatically).
- "passed every step before Rust interop … stopped only on that same mismatch" — ordering verified in `verification/runner/sifr_verify/profile_runner.py:74-91`: `python_interop` is step 5, `rust_interop_checks` step 6, and `run()` (`:235-242`) returns on the first non-zero step. The sequencing and the "stopped only on" phrasing are literally correct, and the paragraph makes no claim about later steps.
- "9 of 10 variants" — matches the reproduced `variants=10, blocking_failures=1`.
- "in review" is honest for a draft PR under review; the plan correctly leaves the final checklist item (`gates, review, merge, unblock certification_10`) unchecked while items 1–6 are `[x]`.
- Minor: the commit subject says "link certification 9 review" though the round-1/round-2 review links landed in `661498bfa`; this commit adds the *PR* link. Cosmetic only.

**Round-2 merge preconditions met.** Both review files are committed non-empty (`round-1` 12,059 B, `round-2` 11,284 B — round-1 finding 8 explicitly forbade committing empty placeholders) and both are linked from the plan's "Review and validation notes" (`:1066-1088`). Worktree and committed copies of `plans/reviews/active/` are identical (`git diff` empty).

**Test binding is real.** `fixtures/native_build_script/fixture.json` binds both directions to `crates/sifr_driver/src/tests/package_rust_interop_native_build_support.rs` with `profile: merge`, `step: crate_tests`, `suite_id: sifr_driver_generated_builds`. Both functions exist and are `#[ignore]`d (`:18-20`, `:94-96`), the module is wired at `package_rust_interop_build_tests.rs:7-8`, and `create-pr.json:90` runs `sifr_driver_generated_builds` as `cargo test -p sifr_driver --lib -- --ignored --test-threads=1`, `status: blocking`, `executed_in_merge: true`.

**Change is scoped, no regression surface.** The `_scenario_checks.py` diff is confined to the `native_build_script` dispatch (inline checks → `validate_native_build_scenario`), the shared token constant, adding `native_build_script` to `reject_unsafe_rust`, and the new `_scenario_files` helper. `_scenario_files` is the only cross-fixture behaviour change: it excludes any path component named `target` from `rglob`, for all fixtures. That is a strict narrowing of build outputs only, and every fixture's scenario check still passes in the reproduced area run — cleared, not a finding. No compiler-crate files are in the PR (`build/rust_interop.rs`, `trust_validation.rs`, and `rust_interop_contract_tests.rs` belong to `certification_8` and appear only against the stale local `main`). `scripts/check_file_size_guardrails.py` → `PASS (2978 files, limit 900 lines)`; `git show b5497901d --check` clean.

**Docs and matrix prose match observed behaviour.** The Apple/GNU arm64/x86_64 host scoping, the explicit non-advertisement of an MSVC envelope, and the C/C++-compiler + discoverable-`libclang` prerequisite appear consistently in `docs/rust-interop.mdx:195-207`, `internal_docs/rust_interop_architecture.md:957-980`, and both fixture READMEs — matching round-1's observed envelope (`stdc++` and the wrapper metadata names are described as portability entries, not as emitted on macOS).

**Not re-executed here:** the two `#[ignore]`d generated-build tests. Round 2 independently ran them green (`2 passed; 0 failed`, 75.56 s) and the validation context confirms it; the only change since round 2 is `b5497901d`, a documentation-only commit that touches no test, fixture, checker, or data input, so it cannot alter those outcomes.

## Findings

None. No actionable findings remain. All eleven round-1 findings were resolved or correctly deferred per round 2, and no new issue is introduced by the post-round-2 commit.

## Mechanical merge preconditions (outside the code under review)

- `plans/reviews/active/rust-interop-certification-9-review-round-3.md` is a 0-byte untracked placeholder. Per round-1 finding 8 it must be written with this review's content and linked from the plan, or left out of the commit entirely — not committed empty.
- Check the final `certification_9` checklist item, flip the status row to `merged`, and unblock only `certification_10`.
- Keep the shared-worktree exclusions and the `ecosystem_backend_certification` `category`/`future_owner` hunk out of any further commit; that hunk remains the sole unrelated reason the full area checker is not all-green.
- PR #3069 is still `DRAFT`; mark ready before merge.

## Carry-forward for `certification_10` (non-blocking, unchanged from round 2)

- Extract the `REQUIRED_SCENARIO_EXAMPLES` / per-fixture dispatch table out of `_scenario_checks.py` before adding to it (891/900 lines).
- Derive the artifact evidence literals in `_scenario_native_build.py` from the pin constants rather than restating them.

SATISFIED
