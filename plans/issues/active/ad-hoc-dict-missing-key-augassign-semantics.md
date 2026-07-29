# Ad Hoc Issue: Dict Missing-Key Augassign Semantics

## Status

Active non-blocking correctness follow-up discovered while diagnosing
[`ad-hoc-algorithmic-full-corpus-preexisting-failures.md`](./ad-hoc-algorithmic-full-corpus-preexisting-failures.md)
on 2026-07-29. It does not block that issue or the current Phase 40 release
qualification. This deferral expires on 2026-10-31; after that date release
readiness fails closed until this correctness issue is resolved or a separately
reviewed decision renews the deadline with current evidence.

## Problem

An explicitly typed plain dictionary can compile a missing-key augmented
subscript assignment into a conditional update that silently does nothing:

```python
values: dict[int, int] = {}
values[1] += 1
```

The approved Python-compatible behavior is to report the missing-key error.
Silently returning a wrong value violates Sifr's correctness guarantee.

The algorithmic full-corpus issue must not work around `defaultdict` inference
by annotating it as `dict`: doing so erases the defaultdict alias and selects
this incorrect plain-dict codegen path.

## Scope

- Establish the approved checked-error surface for missing plain-dict keys.
- Correct lowering/codegen so subscript augassign cannot silently no-op.
- Cover present and missing keys with focused lowering, codegen, and e2e tests.
- Audit related plain-dict subscript assignment forms for the same conditional
  update pattern.
- Cover the alias-erasure path where an explicitly typed `dict` initializer is
  a `defaultdict(...)`; it must not silently select wrong-result plain-dict
  augassign codegen.
- Do not change `defaultdict` missing-key insertion semantics.

## Acceptance Criteria

- [ ] Present-key plain-dict augassign preserves its result.
- [ ] Missing-key plain-dict augassign returns the approved checked error and
      never silently no-ops or panics.
- [ ] Annotated-`defaultdict` initialization cannot erase missing-key semantics
      and silently select the plain-dict conditional-update path.
- [ ] `defaultdict` augassign continues inserting its factory default.
- [ ] Focused compiler and build/run e2e tests cover all four paths.
- [ ] Local validation and iterative review are satisfied.
