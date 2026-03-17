# Ad Hoc Phase: Python Source Parity Extension and Waiver Reduction

Status: open (documented 2026-03-17)
Phase owner: Codex (GPT-5)
Predecessor: `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md` (closed)

## Objective

Extend the closed Python parity phase with a focused follow-up that:

1. closes validated, high-signal parity gaps
2. reduces selected `unsupported` and `host-limited` waivers where feasible
3. preserves Sifr safety principles (typed safety, deterministic behavior, no panic paths)
4. uses the same external review loop discipline for every PR and every closure step

## Operating Constraints

- No breaking signature changes for existing public stdlib APIs.
- Behavior and API additions must remain Sifr-safe and typed; no workaround-first fallbacks.
- Any remaining gap must be explicitly classified in traceability and waiver ledgers.
- CPython parity is improved through adopt/adapt/waive governance, not exception-first behavior.

## Milestone Plan

### Milestone `m1_ext`: Core semantics and high-value APIs (tight 3-module batch)

Wave: `wave_psp_f1_core_semantics_and_high_value_apis`

- `json`
  - Add `JSONEncoder` and `JSONDecoder` class surfaces as typed wrappers.
  - Keep existing `load`/`dump`/`loads`/`dumps` signatures unchanged.
- `re`
  - Add `finditer(...)` and `Pattern.finditer(...)` with explicit adapted list-backed semantics.
  - Keep existing `Match`/`Pattern`/`compile`/`fullmatch` behavior intact.
- `random`
  - Add `seed`, `getstate`, and `setstate` global RNG state helpers with deterministic typed behavior.
  - Keep existing random helper signatures unchanged.
- Update wave traceability and adopt/adapt/waive mapping for all newly closed or deferred surfaces.

### Milestone `m2_ext`: Callable surface and reflection boundary

Wave: `wave_psp_f2_callable_surface_expansion`

- `functools`
  - Target `partial` and `cmp_to_key` only if they can be shipped with typed-safe lowering.
  - Keep decorator-heavy families explicitly waived unless fully closed in-wave.
- `operator`
  - Preserve current helper surface.
  - Only reduce reflective waivers (`attrgetter`, `methodcaller`) if implementation is explicitly typed and deterministic.
- Expand CPython-derived and fail-guard fixtures for newly shipped and still-waived callable/factory paths.

### Milestone `m3_ext`: Utility module parity expansion

Wave: `wave_psp_f3_utility_parity_expansion`

- `hashlib`
  - Add selected high-value algorithms only where backend/runtime support is available without unsafe fallback behavior.
  - Keep current digest/hexdigest bytes-gap classification explicit unless bytes model changes.
- `base64`
  - Prioritize `b16encode` and `b16decode`.
  - Defer base85/ascii85 if parity closure would violate current safety/typing boundaries.
- `statistics`
  - Add pure-safe helpers (`geometric_mean`, `harmonic_mean`, `multimode`, `quantiles`) with typed domain contracts.
- `uuid`
  - Prioritize `uuid3` and `uuid5`; keep unsupported families explicitly waived until architecture permits closure.

## Validation and Acceptance Criteria

For every implemented API or behavior change:

- Add CPython-derived pass fixtures for shipped behavior.
- Add fail/negative fixtures for unsupported or invalid-domain behavior.
- Update traceability evidence and waiver entries to match actual behavior.
- Run:
  - `cargo run -q -p sifr -- run demos/<wave-demo>.sifr`
  - `scripts/run_all_tests.sh --profile quick`
- For wave and milestone closure PRs, also run:
  - `scripts/run_all_tests.sh`

## Mandatory Review Cycles (Same Loop Policy)

For each PR:

1. completion review pass via external reviewer
2. production-grade review pass via external reviewer
3. apply actionable findings only
4. create PR and merge
5. run:
   - `say "First review is done"` after completion pass closure
   - `say "Second review is done"` after production-grade pass closure

Reviewer invocation must write into the active worktree path (`${PWD_NOW}/reviews/...`) and use the wait script (`wait_for_review.py`) with polling.

Additional closure loops required:

- At wave closure: completion check + production-grade check
- At milestone closure: completion check + production-grade check
- At phase closure: completion check + production-grade check

After each closure stage, send status update to Telegram using the existing project script workflow.

## Done Criteria

This ad-hoc phase is complete when:

- target milestones and waves are merged
- reviewer passes are satisfied for PRs, waves, milestones, and phase closure
- all newly touched surfaces have explicit traceability and waiver-state classification
- no unresolved actionable review findings remain
- closure status and ledgers are updated under `issues/` and `verification/stdlib/`
