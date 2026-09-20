# DX.15 verification review followups

status: open; optional separate maintenance; not DX.15 or DX.16 acceptance blockers

Origin: [DX.15 implementation #3868](https://github.com/sifr-lang/sifr/pull/3868)
and [final SATISFIED scoped review](https://github.com/sifr-lang/sifr/pull/3868#issuecomment-5748158816).
The [phase record](../archive/ad-hoc-compiler-dx-and-toolchain-reuse.md) contains the exact
qualification and review receipts. No implementation is included in this issue.

| ID | Owner | Followup and bounded acceptance |
| --- | --- | --- |
| F1 | verification reporting | Document the expected visibility of RSS advisory notes after the Linux KiB-to-byte correction. Compare like-for-like measured RSS; do not treat the previously suppressed note as proof of a new regression, restore incorrect units, or weaken a blocking resource bound. |
| F2 | verification fixtures / project cache | Consider isolating project-cache hints from audit fixture directories. Preserve the ordinary project hint contract, current semantic authority, rejection of symlinked hints and unknown files, and optional-cache failure behavior. The existing regular-file exception is qualified; this is a design improvement, not a current failure. |
| F3 | driver native tests | Consider an explicit cache-hit/freshness assertion across leased fixture preparation and assertion execution. Preserve deterministic generated-project identity, the reset/lease ownership boundary, native assertions and deadlines. Current qualification passes; this would make a future loss of preparation reuse easier to detect. |

The review's case-ID mapping suggestion is addressed in the DX.15 phase record:
P01/P02/P05/P06/P08/P10 map to the current descriptive protocol labels.
These followups do not authorize additional Phase DX implementation or require
code in the documentation-only DX.16 closer.
