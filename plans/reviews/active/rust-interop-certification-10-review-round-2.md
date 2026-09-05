## Scope

`HEAD == origin/main == afd25c392`, so the working tree *is* the diff (32 tracked files + 4 untracked work files). Excluded per instruction: `editor_integrations`, `verification/areas/algorithmic_compatibility/corpora/leetcode`, `.cert5probe/`, `.agent/`, the two stray `*.webp`, `plans/phases/43_interoperability.md`, and the `ecosystem_backend_certification` hunk in `rust_interop_compatibility_matrix.json`.

**Exclusion confirmed separate:** `git diff` renders two independent hunks — `@@ -396,8` (backend row → `supported`, drops `future_owner`) and `@@ -434,15` (`proc_macro_trust` promotion). No shared context lines; the proc_macro_trust promotion stands alone and is internally complete (category + both evidence statuses + notes).

## Round-1 remediations — all independently confirmed

1. **Negative evidence type-correctness.** `negative/untrusted_proc_macro_rejected_pre_execution.sifr:8` now declares `-> str`, matching `src/bridges/generated.rs:4` (`pub fn decode(input: &[u8]) -> String`), and the local is `str`. The mandatory control *executes* it: `assert_armed_build_time_dependencies_write_sentinels` (`package_rust_interop_proc_macro_support.rs:140-147`) installs the negative source with trust intact and asserts `check_package_project` returns no errors — and it runs first, at line 81, before either removal. Verified by running the test, not by reading it.
2. **Mutation control.** `check_fixture_matrix.py:855-876` rejects the exact `-> bytes` regression ("must match the scenario bridge `str` return"), driven by a self-test case at `:373-394` that reads the checked-in fixture and mutates it. A sibling control rejects marker regression (`:330-353`).
3. **Prepass declaration selection.** `trust_validation.rs:94-108` prefers a declaration whose root segment equals `backend.dependency_name`, falling back to `declarations.first()`. No duplicate diagnostics: prepass and per-path both key `seen_trust_requirements` on `canonical_sifr_target_path` of the *same* declaration, and `resolve_declaration` bails at `rust_interop.rs:272-274` once the prepass has pushed anything. Asserted by `package_rust_interop_attributes_package_trust_to_matching_declaration` (`diagnostics.len() == 1`, names `app.hash`, not `app.bridge_call`) and by `errors.len() == 1` in both removal cases of the mandatory negative test.
4. **User-visible allow-list keys.** `require_trust` now renders ``add `X` to `[trust].<allowlist>` ``(`trust_validation.rs:56-58`); `trust_allowlist_name` (`target_resolution.rs:98-108`) maps all seven kinds, and every name matches the real manifest keys in `sifr_package/src/manifest/sifr_fields.rs:77-108`. Covered by child-message assertions in `rust_interop_trust_tests.rs:32,77,102` and by `required_allowlist` in the mandatory negative test (`:95,103,188-193`).
5. **Direct-root coverage restored.** `package_rust_interop_rejects_untrusted_proc_macro_for_direct_root` (`rust_interop_trust_tests.rs:60`) alongside `..._for_local_bridge` (`:81`). Seven trust tests total in that file, matching the issue's wording.
6. **Marker wording.** `serde_derive=1.0.228;upstream=compiled;sifr_wrapper_macro=executed` — consistent across the derive wrapper, bridge output, both package examples, positive fixture, runtime assertion, and both checkers (grep: 14 sites, no stale `macro=executed`). Docs/internal docs/README explicitly state the upstream derive is not invoked through the wrapper. I also confirmed the backing structural checks still bite: mutating `serde_derive_upstream`, `prost_build_upstream`, or `prost_types` out of the wrapper manifests (temp-copy probe) each produces the corresponding failure.
7. **Decomposition / file sizes.** `rust_interop.rs` 899→853 (trust concern moved to `trust_validation.rs`, 168); `_scenario_checks.py` 891→745 with `_scenario_registry.py` (140, scenario token registry) and `_scenario_proc_macro.py` (403, proc-macro scenario policy). `check_file_size_guardrails.py` PASS (2982 files, limit 900); `check_sifr_driver_maintainability_guardrails.py` PASS.
8. **Issue accuracy.** Round 1 is linked at the certification_10 section with its "one medium and five low" outcome, the round-1 fix set is summarized, the double-count is corrected to "seven focused trust tests", and every number I re-ran matches. Expected post-item inventory (36 rows; 20/12/1/3; 13/4/10/9; 66 passing + 6 planned) reproduces exactly when the excluded backend hunk is neutralized.

## Re-run validation (this worktree)

| Gate | Result |
|---|---|
| `cargo test -p sifr_driver` | 432 passed, 0 failed, 61 ignored |
| both mandatory ignored tests | 2 passed, 36.13s combined |
| `check_fixture_matrix.py` | fixtures=36 diagnostics=10 crates=44 package_examples=60 scenario_examples=18 |
| `--self-test` | 184 cases ok |
| tiers / stable claims / stale drafts | 5+36 ok / 33 ok / ok |
| area runner | `variants=10, failures=1` — sole failure is `ecosystem_backend_certification: supported rows require passing positive and negative fixture evidence` (the excluded hunk) |
| `cargo clippy -p sifr_driver --lib -D warnings` | clean |
| scenario `cargo clippy --workspace --all-targets --locked --offline --frozen -D warnings` | clean |
| `cargo fmt --check`, `git diff --check`, HIR guardrail | clean |
| workspace `--all-targets` clippy | 622 diagnostics, all test-only/unrelated (`sifr_package`, `sifr_stdlib_manifest`, `sifr`/e2e, plus `rust_interop_tests.rs:677` `needless_pass_by_value` on `package_context_with_root`, byte-identical to `origin/main:759-763` — pre-existing) |

## Findings

No actionable findings. Non-blocking observations, none of which invalidate a milestone claim:

- **Fallback attribution (nit).** The prepass fallback is `declarations.first()` rather than a bridge-preferring pick, so a package with several declarations and a never-referenced build-time dependency anchors the diagnostic on whichever declaration comes first. Fail-closed, and the message still names the dependency and the exact `[trust]` key, so guidance stays correct.
- **Headroom (nit).** `check_fixture_matrix.py` is 899 and `package_rust_interop_build_tests.rs` is exactly 900 — at the guardrail limit. The next addition to either forces a split.
- **Self-test arithmetic (informational).** 183 → 184 while three fixture-matrix controls were added, i.e. ~2 scenario mutation cases were consolidated during the marker rewording. I probed the affected properties directly (above) and they still reject mutation, so this is bookkeeping, not lost coverage.
- **Artifacts (pre-merge chore).** Both `…round-1.agent.log` and `…round-2.agent.log` are 0 bytes and all four review artifacts are untracked; they need staging with the PR. I left the empty round-2 file untouched, since this run was read-only.

VERDICT: SATISFIED
