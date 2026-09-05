# agent Review: milestone_diag_10 slice 3 structured reveal_type notes

Date: 2026-05-03
Reviewer skill: `agent review`
Invocation: `agent review

## Validation Provided To Reviewer

- `cargo fmt --check`
- `git diff --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `cargo test -p sifr_hir guarded_index -- --nocapture`
- `cargo test -p sifr_driver --lib --tests`
- `cargo test -p sifr test_check_entrypoint_reveal_type -- --nocapture`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=52.30s`)

## Findings

Reviewer raised the following items:

1. `type_check_source` passed `None` source context to `reveal_type_diagnostics`, preserving a source-backed but spanless `check` path.
2. `into_single_file_frontend` drops pre-rendered `rendered_reveal_types` when returning the raw frontend result.
3. `check_project` now returns structured diagnostics instead of printing reveal diagnostics and returning an empty vector.
4. `FrontendModuleDiagnostics::rendered_reveal_types` stores pre-rendered diagnostics in addition to raw reveal diagnostics.
5. Source display names differ between source-only build/emit APIs and file-backed check APIs.
6. The reveal cap test expected `11` omitted diagnostics; reviewer considered this an off-by-one.

## Resolution

- Finding 1 accepted and fixed: source-only `type_check_source` now renders reveal diagnostics with a `"main"` source context, and the corresponding test now asserts a primary span.
- Finding 6 is not a bug: the cap reserves the final rendered diagnostic slot for the omission summary, so 60 raw reveal notes produce 49 explicit notes plus one summary for 11 omitted notes. Added an explanatory test comment.
- Findings 2, 3, 4, and 5 are not correctness issues for this slice:
  - `into_single_file_frontend` intentionally returns the raw `LoweringResult`; rendered diagnostics are produced by the diagnostic-facing entrypoint APIs.
  - `check_project` returning structured non-error diagnostics is the intended phase behavior and is covered by the non-error exit-code contract.
  - storing rendered reveal diagnostics preserves source-map context without keeping borrowed source text inside `ProjectLowering`.
  - source-only public build/compile APIs have no file path parameter; file-backed CLI run/check paths use the real entrypoint path.
