Reviewed the hardening changes. The gate now:
- Enforces the mixed-state allowlist by exact match (no more `startswith("mixed-")` typo drift).
- Adds an explicit `RETAINED_COMPILER_GLUE_SURFACES` loop that requires `_sifr.runtime`, `_sifr.task`, and `generated-runtime-integer-glue` to keep `retained-compiler-language-glue` state and remain non-movable.
- Ships a `--self-test` covering three failure paths: movable future-owned surface, movable retained compiler glue, missing future-owned backstop row.
- Runs both the guard and `--self-test` inside `run_core_guardrails` in `profile_runner.py:289-291`, so `create-pr` and `merge` execute both.
- Documents the guard in `verification/policy/guardrails.json` and in the architecture / issue tracking docs.

Sanity checks I ran:
- `python3 scripts/check_sysroot_stdlib_resource_certification_gate.py` → PASS (surfaces=11, future_runtime_rows=11).
- `python3 scripts/check_sysroot_stdlib_resource_certification_gate.py --self-test` → PASS.
- Cross-checked each of the three `MIXED_CERTIFICATION_STATES` against `stdlib_native_surface_ownership.toml`: `_sifr.crypto` → `mixed-stateless-supported-resource-state-needs-review`, `_sifr.time` → `mixed-stateless-supported-runtime-sensitive`, `_sifr.sys` → `mixed-stdlib-leaf-plus-runtime-sensitive`. All three states are in the allowlist and their rows still reference future-owned matrix ids — the invariant holds.
- Confirmed the self-test's ordering assumptions: `surface[0]` = `_sifr.crypto` (first key of `SURFACE_CERTIFICATION_ROWS`) → triggers "must not be movable"; `surface[-1]` = `generated-runtime-integer-glue` (last entry of `RETAINED_COMPILER_GLUE_SURFACES`) → triggers "retained compiler glue must not be movable". Both assertions match their expected failure substrings.
- Verified the retained-glue loop actually fails on a flip: it checks both `certification_state` drift and `can_move_before_runtime_certification` flip, independently of matrix state.

---

**VERDICT: PASS**

Findings (none blocking):

1. **cosmetic** — `scripts/check_sysroot_stdlib_resource_certification_gate.py:183-184` is missing the PEP 8 blank line between `_matrix_category` and `_self_test`. Scripts aren't lint-gated in this repo, so it's cosmetic, but a blank line matches the rest of the file's style.

2. **self-test coverage gap** — the self-test covers movable-flip on both surface classes and the backstop, but doesn't exercise: (a) the `MIXED_CERTIFICATION_STATES` allowlist rejecting a typo like `"mixed-typo"`, (b) empty/`"none"` `migration_blocker` while future-owned, (c) `certification_state` drift on retained glue (only the movable flip is tested). Adding those would make the self-test a fuller regression fence. Non-blocking.

3. **argv parsing is exact-match** — `if sys.argv[1:] == ["--self-test"]` at line 240 means any typo (e.g., `--selftest` or `--self-test --verbose`) silently falls through to `main()`. Sibling scripts use `argparse`. Non-blocking cosmetic drift.

4. **`SURFACE_CERTIFICATION_ROWS` is still hardcoded in Python** — carried over from pass 1 note #2. A new resource surface added to the ownership TOML is silently uncovered. Consider driving this from a `required_matrix_rows = [...]` field on each ownership entry in a follow-up. Non-blocking.

5. **`guardrails.json` doesn't record the `--self-test` invocation** — the JSON registry only lists the primary entrypoint. Since the profile runner runs both explicitly and this file is a documentation registry (not consumed by the runner), this is documentation drift only. Non-blocking.

**Required fixes: none.** The wave is ready to open a PR.
