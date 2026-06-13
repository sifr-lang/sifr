The review has already been written to `reviews/surface-parity-api-cleanup-review-subgroup1.md`. Here's the summary:

## Corrections (none needed)

- **0072_edit_distance** — `both` correct. Float sentinel in int DP array is genuine shape mismatch post-compiler-fix.
- **0221_maximal_square** — `both` correct. `max(cache.values())` returning `Option` vs `int` is a cross-bucket dependency already flagged.
- **2002_maximum_product...** — `compiler` correct. Minor note: taxonomy trigger label `other_type_surface_and_api_mismatch` miscategorizes it as builtin parity — the review correctly overrides this.
- **0130_surrounded_regions / 0200_number_of_islands / 0994_rotting_oranges** — `both` correct. Range membership is first diagnostic; `mut`/duplicate defs are genuine adaptation issues.
- **0241_different_ways_to_add_parentheses** — `adaptation` correct. Pythonic truthiness pattern (`res or [int(s)]`) is the root cause diagnostic.

## Variadic min/max: Yes, intended builtin parity

Definitive. Sifr's 2-argument ceiling for `min`/`max` is an underspecification, not an intentional restriction. The three-argument form is standard Python. Supporting variadic without weakening `Optional` safety is the right call — it preserves "if it compiles, it works."

## Subgroup verdict

**All classifications stand.** One doc note: `2002` has a trigger-label/category mismatch in the taxonomy that the review correctly overrides. No changes to the `compiler`/`adaptation`/`both` split needed.
