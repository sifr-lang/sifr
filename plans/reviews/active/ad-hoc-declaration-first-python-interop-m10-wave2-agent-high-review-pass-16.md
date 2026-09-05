# M10 Wave 2 whole-diff review — pass 16

Reviewer: agent, high reasoning, fast service tier, ephemeral read-only whole-diff review.

Scope: complete committed `main...3bfdff89d` diff after pass-15 remediation and a requested `cargo clean`.

## Verdict

**CHANGES REQUIRED. Not satisfied; M10 Wave 2 is not ready to merge.**

## Blockers

1. **High — emitted generic method bounds are not enforced at specialization sites.** Lowering only rejects a narrow non-Clone return-type case, so transitive `Clone`/`PartialEq` method requirements can be emitted in Rust while a concrete unsupported specialization remains accepted by `sifr check`.
2. **High — per-parameter operator inference is incomplete.** Recursive collection equality misses `PartialEq`, and generic `*`, `/`, and `%` bodies do not receive the operator bounds required by the emitted Rust definition.
3. **High — generic operator-protocol implementations are malformed.** Generic `__eq__`, `__lt__`, and unary implementations still emit bare class targets without generic parameters or the exact body-required bounds.
4. **High — keyed `sorted()` admission and source materialization are incomplete.** Lowering does not validate that the key parameter accepts the iterable element, and conditional sources can bypass intrinsic materialization while leaving branch moves untracked.
5. **Medium — inferred top-level returns remain source-order dependent.** An earlier caller sees `Any` for a later unannotated callee and rejects an otherwise valid program.
6. **Medium — milestone evidence overstates closure.** The phase ledger, capability row, and architecture claims require correction and permanent native pass/fail coverage for every counterexample above.

## Review evidence

- Reviewed all 225 committed `main...HEAD` files, all prior review artifacts, the phase ledger, architecture documents, activation matrix, buffer runtime/FFI, code-generation paths, and fixture discovery.
- Reproduced every blocker with `sifr check` and generated-Rust inspection.
- Existing focused pass-15 codegen tests and all 58 sorted-related lowering tests pass but do not cover these counterexamples.
- `git diff --check`, HIR maintainability guardrails, changed-JSON parsing, and the 900-line file-size guardrail pass.
- No additional production panic, buffer-release, overlap-admission, or deterministic-generation blocker was found.
- The unrelated dirty `third_party/ruff` submodule was excluded and left untouched.

## Required closure

Model method requirements in lowering and enforce them at concrete specialization; infer exact recursive/operator bounds for generic bodies and every operator protocol; align keyed `sorted()` typing and conditional ownership with code generation; make top-level inferred signatures order-independent; add permanent regression coverage and correct the evidence; rerun authoritative validation; then submit the complete diff to another independent whole-diff review.

## Remediation record

All six blocker families are implemented for the next independent review:

- lowering records, closes, exports/imports, and validates exact generic-method requirements at concrete specialization sites;
- code generation derives recursive per-parameter arithmetic/equality/ordering/negation bounds and emits generic operator-protocol targets;
- keyed `sorted()` validates its input parameter and materializes conditional sources branch-by-branch with exact ownership;
- a diagnostic-neutral module-signature prepass makes successful inferred top-level returns source-order independent while preserving reachability-aware body diagnostics;
- five permanent native pass/fail fixtures cover the reproduced counterexamples;
- the evidence ledger, capability matrix, and architecture statement are corrected.

Validation after remediation: full codegen `824/824`; lowering `745` passed with one ignored; compile-fail `520/520`; native execution for all three positive fixtures; authoritative create-PR gate passed every blocking lane in `489.54s`, including Python interop `11/11`, runtime platform `28/28` with one gated skip, and E2E `131/131`; HIR maintainability, formatting, and the `900`-line cap over `2659` files passed. The original verdict above remains historical; closure requires the next whole-diff reviewer to approve.
