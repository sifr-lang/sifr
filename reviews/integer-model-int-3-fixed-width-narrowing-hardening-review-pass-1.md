# INT-3 Fixed-Width Narrowing Hardening Review Pass 1

## Findings

1. The four new hardening tests cover return, list literal, dict literal, and generic class specialization positions. They correctly assert `TYPE_MISMATCH` with precise diagnostic ranges.
2. `class_specialization_payload_conflicts` is correctly implemented: it gates on same class name and same field count, excludes self-recursive classes via `type_mentions_class` recursion, requires a `FixedInt` in at least one payload, and checks field-by-field assignability.
3. The self-recursive exclusion preserves existing recursive narrowing behavior for recursive classes such as `Tree[T]` with recursive fields.
4. The `lower_ann_assign` modification is minimal and additive, extending the existing error path rather than replacing it.
5. The conservative behavior is safe and aligned with the integer model: the check rejects `Box[X]` to `Box[Y]` where the payloads differ across fixed-width-sensitive types.

## Required Changes

None.

## Non-blocking Notes

None.

## Verdict

Approved for this milestone. The implementation correctly hardens the stated boundaries while preserving recursive class behavior, as confirmed by the passing recursive class regression test.
