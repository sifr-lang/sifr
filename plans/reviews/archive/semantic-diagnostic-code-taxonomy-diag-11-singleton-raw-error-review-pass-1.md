# Review: semantic-diagnostic-code-taxonomy-diag-11-singleton-raw-error

Branch: `codex/diag-11-raw-hir-singletons`

Date: 2026-05-03

Reviewer: agent (`agent review`)

## Scope

- Migrate generator return inference diagnostics emitted through `infer_function_return_type` callbacks from raw `ctx.error(String)` to structured `TYPE_MISMATCH` diagnostics with a primary range on the return annotation.
- Migrate nested function ambiguous return inference failure from raw `ctx.error(String)` to `TYPE_MISSING_ANNOTATION` with a primary range on the nested function name.
- Remove the legacy raw diagnostic transport unit test and add guardrail coverage for the now-migrated files.
- Add focused HIR tests and the e2e fail fixture `generator_return_annotation_mismatch.sifr`.

## Findings

Required fixes: none.

The reviewer found the taxonomy fit correct:

- `TYPE_MISMATCH` is appropriate for generator functions whose body uses `yield` while the declared return annotation is not `Iterator[T]`.
- `TYPE_MISSING_ANNOTATION` is appropriate for nested helper return inference that cannot be determined deterministically, because the function needs an explicit annotation.

The reviewer found the primary ranges correct:

- Generator diagnostics point at the return annotation expression, such as `list[int]`.
- Nested ambiguous return inference diagnostics point at the nested function name.

The reviewer found the guardrail update correct:

- `diagnostic_transport_tests.rs`, `nested_function_inference.rs`, and `typing_and_functions.rs` are now protected by `RAW_HIR_ERROR_FREE_FILES`.
- The legacy raw transport test was removed instead of preserving a fallback contract.

The reviewer found the test coverage sufficient:

- `test_generator_function_rejects_non_iterator_annotation` verifies code and range.
- `test_nested_ambiguous_return_inference_has_code_and_primary_range` verifies code and range.
- `generator_return_annotation_mismatch.sifr` verifies e2e fail output with `SIFR-TYPE-0002`.

## Status

Approved / satisfied.

agent note: the review completed successfully, but the reviewer process could not write this file directly due to permission restrictions. The review output reported no required fixes.
