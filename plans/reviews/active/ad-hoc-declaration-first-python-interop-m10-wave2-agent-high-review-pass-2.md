# M10 Wave 2 review — pass 2

- Date: 2026-07-15
- Pull request: [#2988](https://github.com/sifr-lang/sifr/pull/2988)
- Reviewer: agent
- Reasoning/service tier: `high` / `fast`
- Scope: complete `main...HEAD` diff
- Verdict: changes required

## Findings

1. **High — recursive affine capability handling is incomplete outside class
   derives and equality.** General unions still unconditionally derive `Clone`
   and format every non-class variant with `Display`; list, dict, and tuple
   projection paths still synthesize `Clone`/`.cloned()` for affine elements.
   The aggregate fixture declares shapes but does not exercise construction,
   movement, release, general unions, tuples, or recursive classes.
2. **Medium — `@python.buffer(Self, ...)` silently discards receiver ownership
   conventions.** An `own self` declaration is accepted but emitted as `&self`.
   Unsupported owning or mutable receiver conventions must be rejected, or
   preserved end to end.
3. **Low — review/status tracking claims a satisfied rereview while the review
   is still pending and has now requested changes.**

## Resolution status

- [x] Make union derives/formatting and all aggregate projection paths respect
  recursive affine capabilities, with static rejection where safe movement is
  unsupported.
- [x] Add rustc-backed operational aggregate coverage for records, options,
  collections, tuples, general unions, recursive classes, and exact release.
- [x] Reject unsupported owning and mutable `Self` buffer receivers and cover
  the diagnostic plus generated shared-receiver signature.
- [x] Record the pass-2 changes-required state atomically.
- [ ] Run focused and authoritative validation, then a fresh full-diff review.

## Remediation evidence

- General unions now derive `Clone` only when every member supports it and use
  `Debug` formatting for affine, class, non-display, and `None` members.
- Affine aggregate indexing, slicing, field projection, and iteration are
  rejected before code generation; list/tuple literal insertion moves and
  consumes affine names instead of synthesizing a clone.
- Structured and simple local binding coercion inject concrete fallible values
  into option and general-union targets using the unwrapped `Result` success
  type.
- The permanent compiled aggregate case constructs records, options, lists,
  tuples, general unions, and recursive classes, then verifies zero live and
  leaked resources. The complete buffer example suite passes `4/4`.
- Focused lowering contracts pass `13/13`; focused buffer code generation
  passes `5/5`; targeted union and local-coercion regressions pass.
