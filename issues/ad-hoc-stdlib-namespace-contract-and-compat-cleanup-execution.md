# Ad Hoc Phase Execution Checklist: Stdlib Namespace Contract And Compatibility Cleanup

Phase contract: [ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md)

Status: complete; all milestones merged, local create-pr and merge gates passed, and final reviewer closeout is READY.

## Checklist

- [x] `milestone_stdlib_namespace_1`: Policy And Diagnostics
- [x] `milestone_stdlib_namespace_2`: Atomic Compatibility Removal
- [x] `milestone_stdlib_namespace_3`: Corpus Adoption And Closeout

## Review Artifacts

Record planning and implementation reviews here.

- Planning review pass 1: `reviews/ad-hoc-stdlib-namespace-contract-planning-review-pass-1.md` -> `CHANGES_REQUESTED`; requested explicit `Stmt::Import` coverage, corrected current bare-import no-op framing, pinned `SIFR-IMPORT-0008` / `IMPORT_BARE_STDLIB`, clarified stdlib-tail resolution order, split M2 synthetic-import cleanup from M3 defaultdict cleanup, added synthetic import consumer removal, tightened guardrails, and renamed typed defaultdict internals away from `__compat_defaultdict_*`.
- Planning review pass 2: `reviews/ad-hoc-stdlib-namespace-contract-planning-review-pass-2.md` -> `CHANGES_REQUESTED`; narrowed guardrails to the removed `math|heapq|collections` synthetic aliases so retained async/task aliases stay out of scope, added explicit cleanup for Rust tests that hard-code removed aliases, kept the generic async/task codegen guard intentionally retained, required grep-driven fixture classification, and originally requested a transitional defaultdict helper. The transitional-helper decision is superseded by the no-legacy-support clarification below.
- Planning review pass 3: `reviews/ad-hoc-stdlib-namespace-contract-planning-review-pass-3.md` -> `READY`; reviewer verified the plan was implementation-ready before the later no-legacy-support clarification removed the transitional defaultdict bridge.
- Gap audit pass 1: `reviews/ad-hoc-stdlib-namespace-contract-gap-audit-pass-1.md` -> `CHANGES_REQUESTED`; found missing cross-layer diagnostic ownership and transport decisions. Addressed by requiring structured args in `HirDiagnostic`/lowering transport, assigning project/package `ImportFrom` diagnostics to discovery after real resolution fails, assigning all `Stmt::Import` bare-stdlib diagnostics to lowering, adding shared `sifr_stdlib` tail helpers, defining duplicate-prevention rules, expanding M1 project/package/single-file tests, and naming explicit defaultdict binding state.
- Gap audit pass 2: `reviews/ad-hoc-stdlib-namespace-contract-gap-audit-pass-2.md` -> `READY`; reviewer confirmed all cross-layer decisions are locked, including structured lowering diagnostic transport, discovery/lowering ownership split, duplicate prevention, exact-tail/root-fallback matching, M1 test scope, explicit defaultdict binding state, and guardrail coverage.
- Final readiness pass 1: `reviews/ad-hoc-stdlib-namespace-contract-final-readiness-pass-1.md` -> `READY`; reviewer confirmed the phase is implementation-ready after the final scan added exact diagnostic arg shape, compile-order/dependency-collector carveouts, and explicit cleanup for `class_field_inference.rs` bare `deque`/`Counter`/`defaultdict` compatibility paths.
- No-legacy clarification: superseded the previous M2/M3 transitional split. Current phase requires atomic compatibility removal: `math.*`, `heapq.*`, `collections.*`, bare `deque(...)`, bare `Counter(...)`, bare `defaultdict(...)`, `collections.defaultdict(...)`, class-field inference compatibility, synthetic imports, and `__compat_defaultdict_*` naming are removed or converted directly to explicit `sifr.*` binding in `milestone_stdlib_namespace_2`.
- No-legacy review pass 1: `reviews/ad-hoc-stdlib-namespace-contract-no-legacy-review-pass-1.md` -> `READY`; reviewer confirmed the revised phase has no backward-compatibility or legacy-support loopholes for CPython-style bare stdlib calls.
- Corpus discovery update: added final LeetCode/demo adoption requirements after scanning `audits/leetcode/src` and `demos`. Current discovery found 416 checked-in LeetCode `.sifr` fixtures and 389 demo `.sifr` files; M3 must update/validate all affected LeetCode fixtures and demos, and add or update corpus validation commands so all checked-in LeetCode fixtures and all runnable demos work.
- Corpus review pass 1: `reviews/ad-hoc-stdlib-namespace-contract-corpus-review-pass-1.md` -> `READY`; reviewer confirmed the final corpus milestone is implementation-ready and covers all checked-in LeetCode fixtures, all runnable demos, and repeated discovery after implementation.
- Final implementation-readiness pass 1: `reviews/ad-hoc-stdlib-namespace-contract-final-implementation-readiness-pass-1.md` -> `READY` with one non-blocking observation that `demos/collections_and_argparse/main.sifr` left a small `defaultdict(0)` judgment call.
- Final implementation-readiness pass 2: `reviews/ad-hoc-stdlib-namespace-contract-final-implementation-readiness-pass-2.md` -> `READY`; reviewer confirmed the phase is implementation-ready with no hidden decisions after the phase explicitly chose the typed `defaultdict(int/list/set)` public contract and rejected preserving the older integer-default `defaultdict(0)` class-style API.
- M1 implementation review pass 1: `reviews/ad-hoc-stdlib-namespace-m1-implementation-review-pass-2.md` -> `READY` with non-blocking cleanup items.
- M1 implementation review pass 2: `reviews/ad-hoc-stdlib-namespace-m1-implementation-review-pass-3.md` -> `READY`; reviewer found no blocking M1 issues and confirmed `SIFR-IMPORT-0008` registry/docs, shared bare stdlib helper behavior, lowering args/help transport, project/package probe-then-reclassify behavior, `Stmt::Import` ownership, user-module priority, and required test/fixture coverage.
- M2 implementation review pass 1: `reviews/ad-hoc-stdlib-namespace-m2-implementation-review-pass-1.md` -> `READY`; reviewer found no blocking M2 contract issues and called out validation hygiene items for the PR gate and M3 closeout.
- M3 implementation review pass 1: `reviews/ad-hoc-stdlib-namespace-m3-implementation-review-pass-3.md` -> `READY`; reviewer found no correctness blockers in the corpus adoption, defaultdict string-key lowering, typed nested closure params, speculative string-cache rollback/init, or artifact-cache concurrent populate handling. Non-blocking notes to exclude generated pending-snapshot edits and commit/update the LeetCode submodule were handled before the parent PR.
- Final implementation review pass 1: `reviews/ad-hoc-stdlib-namespace-final-implementation-review-pass-1.md` -> `READY`; reviewer confirmed milestones M1-M3 are complete, checklist state is closed, PRs are merged and recorded, local create-pr and merge gates passed, removed compatibility symbols remain absent from production, `SIFR-IMPORT-0008` / `IMPORT_BARE_STDLIB` is wired across the required layers, and exit gates 1-10 are satisfied.

## Validation Ledger

Record local validation for each milestone before opening the corresponding PR.

- M1: focused validation passed:
  - `cargo test -p sifr_stdlib bare_stdlib_tail`
  - `cargo test -p sifr_lowering bare_stdlib -- --nocapture`
  - `cargo test -p sifr_driver bare_stdlib`
  - `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture`
  - `cargo run -q -p sifr_driver --bin diagnostic_contract_harness`
  - `python3 scripts/run_verification_hardening.py --suite project`
  - `scripts/run_all_tests.sh --profile create-pr`
  - PR: https://github.com/sifr-lang/sifr/pull/2291
- M2: focused validation passed:
  - `cargo check -p sifr_lowering -p sifr_codegen -p sifr_type_system -p sifr`
  - `cargo test -p sifr_lowering defaultdict -- --nocapture`
  - `cargo test -p sifr_codegen defaultdict -- --nocapture`
  - `cargo test -p sifr_type_system defaultdict -- --nocapture`
  - `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture`
  - `cargo test -p sifr_driver stdlib_exports -- --nocapture`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_attribute_calls.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/defaultdict_len_and_deque.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/counter_defaultdict_and_argparse.sifr`
  - `python3 scripts/check_file_size_guardrails.py`
  - forbidden production-symbol scans for removed `__compat_sifr_(math|heapq|collections)_`, `__compat_defaultdict_*`, alias resolvers, and synthetic import state returned no hits.
  - `scripts/run_all_tests.sh --profile create-pr` passed on rerun. The first run hit transient cached e2e Rust temp-dir failures for `stdlib_json_consolidated` and `stdlib_tomllib`; both fixtures passed direct `cargo run -q -p sifr -- run ...`, and the rerun passed the full create-pr lane with 67/67 e2e pass fixtures.
- M3: focused validation passed:
  - `cargo build -q -p sifr`
  - `cargo test -p sifr_codegen test_generate_rust_defaultdict_list_len_borrows_string_literal_key -- --nocapture`
  - `cargo test -p sifr_codegen test_generate_rust_nested_string_function_closure_params_are_typed -- --nocapture`
  - `cargo test -p sifr_codegen string_local_cache_decl_survives_speculative_if_lowering_rollback -- --nocapture`
  - `cargo test -p sifr_driver pending_artifact_commit_treats_existing_final_dir_as_concurrent_populate -- --nocapture`
  - `target/debug/sifr run demos/defaultdict/main.sifr`
  - `target/debug/sifr run demos/nested_functions/main.sifr`
  - `target/debug/sifr run demos/text_and_patterns/main.sifr`
  - `python3 scripts/run_stdlib_namespace_corpus_validation.py --scope demos --command run` passed with 272/272 runnable non-negative demo `main.sifr` entrypoints in 259.4s.
  - `python3 scripts/run_stdlib_namespace_corpus_validation.py --scope leetcode --command check` passed with 411/411 checked-in LeetCode `audits/leetcode/src/*.sifr` fixtures in 346.1s.
  - `rg -n -P "(?<!sifr\\.)\\b(math|heapq|collections)\\.[A-Za-z_]" audits/leetcode/src demos crates/sifr/tests/e2e --glob '*.sifr'` returned only intentional negative e2e fixtures: `bare_stdlib_from_collections_abc.sifr` and `collections_defaultdict_constructor_rejected.sifr`.
  - `rg -n "\\b(defaultdict|Counter|deque)\\s*\\(" audits/leetcode/src demos crates/sifr/tests/e2e --glob '*.sifr'` returned explicit `sifr.collections` uses, local-class false positives, and intentional negative fixtures; no unclassified corpus compatibility users remain.
  - `cargo fmt --check`
  - `cargo clippy --workspace -- -D warnings`
  - `python3 scripts/check_hir_maintainability_guardrails.py`
  - `python3 scripts/check_file_size_guardrails.py`
  - `scripts/run_all_tests.sh --profile create-pr` passed; report `target/validation_lane_reports/create-pr.latest.json`, wall time 149.45s, 67/67 e2e pass fixtures, non-blocking warm wall-time advisory.
  - `scripts/run_all_tests.sh` passed; report `target/validation_lane_reports/merge.latest.json`, wall time 598.34s, 73/73 e2e pass fixtures, non-blocking group-skew advisory.
- Final closeout docs:
  - `python3 scripts/check_file_size_guardrails.py`
  - `scripts/run_all_tests.sh --profile create-pr` passed; report `target/validation_lane_reports/create-pr.latest.json`, wall time 123.03s, 67/67 e2e pass fixtures, non-blocking warm wall-time advisory.

## Merged PRs

Record merged PR links here as each milestone lands.

- M1: https://github.com/sifr-lang/sifr/pull/2291
- M2: https://github.com/sifr-lang/sifr/pull/2292
- M3 corpus submodule: https://github.com/sifr-lang/leetcode/pull/38
- M3 parent: https://github.com/sifr-lang/sifr/pull/2293
