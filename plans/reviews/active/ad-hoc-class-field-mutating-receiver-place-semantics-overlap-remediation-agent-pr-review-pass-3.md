# Class-field receiver overlap remediation — agent PR review pass 3

## Review target

- Base: `a7a5df414b985cc95a9ad23c5b006caa84101f0d`
- Head: `70336021d97cca76b62e23c9e1d8c2eb391e4b3c`
- PR: [#3090](https://github.com/sifr-lang/sifr/pull/3090)
- Reviewer: agent, effort `medium`

## Prior findings

Both documentation-of-record blockers from pass 2 are closed:

1. The plan now distinguishes precise statically resolvable callable/recursive
   field projections from genuinely dynamic or unresolvable roots.
2. The status and review ledger now record #3087, #3090, and the remediation
   review rounds.

## Independent verification

The reviewer independently:

- ran the 21 targeted `unsupported_` lowering tests;
- ran the complete lowering suite (`941 passed`, `1 ignored`) and codegen suite
  (`954 passed`);
- reproduced callable and recursive overlap failures as structured
  `SIFR-OWN-0002`;
- checked that callable and recursive disjoint sibling fields remain accepted;
- reproduced the async-generator `SIFR-OWN-0002` with its structured binding;
- ran the generated-diagnostic-doc check, documentation link check, HIR
  maintainability guardrail, file-size guardrail, and diagnostic-doc sync;
- inspected the exact-head create-PR log, including `137/137` E2E fixtures,
  signature `eeeeb711211648b0`, and all enforced performance budgets;
- confirmed the six phase-added runtime fixtures are present in the create-PR
  manifest; and
- confirmed the reviewed worktree was clean.

## Findings

Blocking findings: none.

Non-blocking observations:

1. The validation ledger retained older lowering/codegen counts.
2. `class_method_mut_borrowed_field_argument` was only covered by the merge
   profile even though the plan calls out its runtime assertion.
3. The genuinely unresolvable fallback conservatively records both a dynamic
   root and the object footprint; the duplication is harmless.
4. Separate pre-existing value-codegen failures for independently moving
   callable/recursive fields and mutable free-function field arguments should
   be retained as follow-up debt.
5. `method_receiver_places.rs` is at 898 lines, so further implementation work
   in that area should split the module first.

## Verdict

**SATISFIED.** The prior blocking findings are closed, the implementation
preserves precise static field identity without under- or over-rejecting the
reviewed shapes, and no blocking correctness, diagnostic, test, or
documentation issue remains.
