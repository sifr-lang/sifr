# M10 Wave 2 review — pass 3

- Date: 2026-07-15
- Pull request: [#2988](https://github.com/sifr-lang/sifr/pull/2988)
- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning/service tier: `high` / `fast`
- Scope: complete committed `main...HEAD` diff
- Verdict: changes required

## Findings

1. **High — affine capability checks still miss clone, equality, and
   hash-dependent collection operations.** `list.copy`, list concatenation,
   append, set elements, and dictionary keys can reach Rust operations that
   require capabilities intentionally absent from `PythonBuffer`.
2. **High — ownership-producing expressions still miss affine move tracking.**
   Default structural constructors, walrus aliases, and comprehensions can emit
   Rust moves without consuming or rejecting the Sifr source binding.
3. **High — writable buffer acquisition does not enforce exporter-level
   exclusivity.** Repeated writable acquisition through a shared receiver can
   create coexisting writable views over the same exporter.
4. **Medium — permanent aggregate coverage does not exercise those residual
   trait, move, comprehension, alias, and writable-alias surfaces.**

The reviewer found no separate blocker in call-then-acquire owner retention,
exact-once release mechanics, or activation ledger/document consistency.

## Resolution status

- [x] Gate or consume every clone/equality/hash-dependent aggregate operation.
- [x] Track or reject affine moves through constructors, walrus expressions,
  and repeated comprehension bodies.
- [x] Enforce exporter-level writable admission without hidden aliasing.
- [x] Add permanent compiler/runtime coverage for every remediation surface.
- [ ] Run focused and authoritative validation, then a fresh full-diff review.

## Remediation evidence

- Collection typing now rejects affine clone/comparison operations, list
  concatenation, concrete unhashable set elements and dictionary keys, while
  preserving unresolved generic key capability until specialization. Affine
  list insertion transfers ownership without generated cloning.
- Default structural constructors and walrus expressions consume affine source
  bindings. Synchronous and asynchronous comprehensions reject affine
  iteration and repeatable affine result production.
- The runtime buffer store now admits multiple readers or one writer per
  exporter identity. Admission stays live until in-flight access drains and
  `PyBuffer_Release` completes, then reopens deterministically.
- Permanent coverage includes collection capabilities and insertion,
  constructor/walrus moves, list/set/dict/generator repetition, non-cloning
  code generation, shared-reader/exclusive-writer admission, reopening, and
  zero-live-resource cleanup.
- Focused suites pass: lowering `17/17`, buffer code generation `6/6`, and
  runtime buffer operations `13/13`. Full affected suites pass: code generation
  `810/810`, lowering `727/727` with one ignored, type system `97/97`, and
  Python-enabled runtime `201/201`.
- The authoritative create-PR facade is functionally green through Python
  interop `11/11`, including buffer examples, but its latest run stopped on the
  interop timing budget (`356.8s > 300s`) after callback example recompilation;
  a clean budgeted run and fresh reviewer verdict remain pending.
