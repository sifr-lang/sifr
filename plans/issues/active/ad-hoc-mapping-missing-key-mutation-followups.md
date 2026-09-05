# Ad Hoc Issue: Remaining Mapping Missing-Key Mutation Semantics

## Status

Active non-blocking correctness follow-up recorded during the exact-SHA reviews
of [PR #3108](https://github.com/sifr-lang/sifr/pull/3108). These behaviors
predate that implementation and are outside the closed plain-name subscript
augassign item.

## Problem

Several related mapping mutation forms still use conditional generated-Rust
updates whose missing-key branch is empty, or retain stale exact-key presence
facts after a key can no longer be present. Those paths can silently no-op or
suppress a required checked `KeyError`.

## Scope

- Invalidate exact-key presence facts after `del d[k]` and any other operation
  that can remove the proven key.
- Make a missing plain-dictionary `del d[k]` return the approved checked
  `KeyError` instead of discarding `HashMap::remove`'s result.
- Make missing-key bucket mutation such as `d[k].append(value)` fail through
  the same checked-error surface instead of silently no-oping.
- Give `defaultdict(list)` and `defaultdict(set)` subscript augassign the same
  factory insertion guarantee already implemented for `defaultdict(int)`.
- Audit attribute-rooted forms such as `self.counts[k] += 1` so they use the
  same missing-key contract as plain-name mappings.
- Derive generated checked-error construction from the resolved lowering type,
  and render a single useful key-bearing error message without redundant
  string conversion.

## Acceptance Criteria

- [ ] Presence facts cannot survive deletion or another key-removing mutation.
- [ ] Missing plain-dictionary deletion returns checked `KeyError` and never
      silently succeeds or panics.
- [ ] Missing-key bucket method mutation returns checked `KeyError`.
- [ ] `defaultdict(int/list/set)` augassign inserts and mutates the factory
      default for every supported factory.
- [ ] Plain-name and attribute-rooted mapping augassign share the same
      checked-error contract.
- [ ] Focused lowering, codegen, native runtime, and stale-flow regressions
      cover every corrected form.
- [ ] Local validation and exact-SHA iterative review are satisfied.

## Source Evidence

- [agent review round 1](https://github.com/sifr-lang/sifr/pull/3108#issuecomment-5225915101)
  records the pre-existing delete, bucket-method mutation, defaultdict
  list/set, attribute-rooted, and error-message findings.
- [agent review round 2](https://github.com/sifr-lang/sifr/pull/3108#issuecomment-5225915170)
  confirms the stale delete guard, missing delete error, and defaultdict
  list/set gaps are pre-existing and non-blocking for PR #3108.
