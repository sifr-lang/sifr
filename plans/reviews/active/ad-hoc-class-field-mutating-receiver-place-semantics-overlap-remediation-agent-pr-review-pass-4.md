# Class-field receiver overlap remediation — agent PR review pass 4

## Review target

- Base: `a7a5df414b985cc95a9ad23c5b006caa84101f0d`
- Head: `eb509285ad6e7ecfd0e974e8e9df07a8ba37248a`
- PR: [#3090](https://github.com/sifr-lang/sifr/pull/3090)
- Reviewer: agent, effort `medium`
- Mode: read-only; scratch reproductions remained outside the repository

## Prior finding disposition

The pass-1 and pass-2 blockers remained closed. Of the five non-blocking
pass-3 observations:

1. Lowering and codegen counts were corrected to `941 passed`, `1 ignored` and
   `954 passed`, both independently reproduced. An older `680/680` E2E figure
   remained in the same sentence; see the non-blocking finding below.
2. `class_method_mut_borrowed_field_argument` was added to the create-PR
   manifest, which was revalidated at `138/138`, signature
   `4ede7c71d86f381c`.
3. The redundant dynamic-root/object-footprint fallback remained harmless.
4. The pre-existing value-codegen debt was explicitly retained in the plan.
5. `method_receiver_places.rs` remained at 898 lines and passed the file-size
   guardrail.

## Findings

Blocking findings: none.

Non-blocking:

1. The validation bullet mixed freshly reproduced lowering/codegen counts with
   a stale, non-reproducible `680/680` Item 2 pass-corpus figure. The reviewer
   confirmed the current pass corpus contains 685 fixtures and that the
   authoritative full merge gate is explicitly assigned to the integrated
   closure head. Removing the stale number is a record-precision cleanup, not a
   correctness or test blocker.
2. The separately recorded value-codegen debt was independently reproduced and
   correctly scoped as pre-existing: mutable free-function field arguments and
   independently moved recursive fields can still reach raw Rust errors, but
   this PR neither introduces nor conceals them.

No blocking correctness, diagnostic, test, record, or merge-readiness issue
remained.

## Independent validation

The reviewer independently:

- ran the full lowering suite (`941 passed`, `1 ignored`), codegen suite
  (`954 passed`), diagnostics suite (`32 passed`), and complete annotated fail
  suite;
- ran the 21 targeted `unsupported_` lowering tests;
- reproduced both new fail fixtures at their annotated columns as structured
  `SIFR-OWN-0002`;
- inspected `extract_footprint_place`, its field-identity resolution, the
  precise static fallback, and the genuinely dynamic fallback;
- checked the 138-entry create-PR manifest for duplicates and missing fixtures,
  and inspected the mutable-borrowed-field runtime assertions;
- inspected the exact create-PR log at implementation head `92b38be705`, which
  records `137/137`, signature `eeeeb711211648b0`, and every enforced step
  passing;
- inspected the post-review manifest E2E log, which records `138/138`,
  signature `4ede7c71d86f381c`;
- ran `cargo fmt --check`, canonical workspace clippy with warnings denied,
  generated diagnostic-doc drift checking, documentation error-link checking,
  HIR maintainability, and the file-size guardrail; and
- confirmed the reviewed worktree was clean at
  `eb509285ad6e7ecfd0e974e8e9df07a8ba37248a`.

## Verdict

**SATISFIED.** No blocking correctness, diagnostic, test, record, or
merge-readiness issue remains.
