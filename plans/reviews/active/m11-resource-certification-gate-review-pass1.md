I have gathered enough context to finalize the review.

VERDICT: PASS

The gate correctly enforces the intended invariant: M11 resource-sensitive surfaces cannot be flipped to movable while their required Rust interop matrix rows are still `future-owned-by-separate-phase`. It reads the correct two sources of truth (`internal_docs/stdlib_native_surface_ownership.toml` + `verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json`), and is wired into `run_core_guardrails` at `verification/runner/sifr_verify/profile_runner.py:289-290`, so `create-pr` and `merge` (both legacy-facade profiles) will execute it — not just list it. The docs explicitly defer migration ("cannot advance a resource-shaped surface ahead of the runtime evidence") and never claim resource support. The self-refreshing backstop at `scripts/check_sysroot_stdlib_resource_certification_gate.py:90-99` correctly forces this guard to be revisited when the last future-owned row lands.

Row mapping cross-check against the ownership registry: every entry the guard flags (`_sifr.crypto`, `_sifr.time`, `_sifr.logging`, `_sifr.fs`, `_sifr.process`, `_sifr.sys`, `_sifr.signal`, `_sifr.net`, `_sifr.tls`, `_sifr.http`, `_sifr.python`) has `can_move_before_runtime_certification = false` and a `certification_state` that matches one of the three accepted prefixes (`future-owned-by-runtime-resource-certification` / `mixed-` / `retained-compiler-language-glue`). A flip to `true` or a `certification_state = can-move-before-runtime-certification` or `migration_blocker = "none"` on any of these rows produces a failure — verified by walking the logic against each surface.

`verification/policy/guardrails.json` is a documentation registry that is not actually consumed by the profile runner today, but the guard runs via explicit invocation in `profile_runner.py`, so this is fine and matches the pattern of the other guards.

Non-blocking notes:

1. **No self-test.** Sibling guards (e.g., `scripts/check_source_crate_dependency_direction.py`, `scripts/check_submodule_ownership.py`) support `--self-test`. Adding a small self-test that synthesizes an ownership/matrix pair to prove the "movable while future-owned" and "missing backstop row" failure paths would harden this against silent breakage from future schema changes.

2. **Surface→row mapping is hard-coded Python.** If the compatibility matrix renames a row, `_matrix_category` fails loudly (good). But if the ownership TOML adds a new resource surface (e.g., a hypothetical `_sifr.database`), it will silently be uncovered. Consider driving `SURFACE_CERTIFICATION_ROWS` from a field in the ownership TOML (e.g., `required_matrix_rows = ["opaque_resource_matrix"]`) so the source of truth stays in one place.

3. **`_sifr.runtime` / `_sifr.task` / `generated-runtime-integer-glue` are not in the guard.** They're `retained-compiler-language-glue` with `can_move_before_runtime_certification = false`, so today it doesn't matter — but the guard wouldn't catch a bad flip. Consider a companion check that "retained-compiler-language-glue must not be movable" independent of matrix state.

4. **`mixed-` prefix is permissive.** `certification_state.startswith("mixed-")` accepts any string starting with `mixed-`. Currently fine because content is curated, but a small allowlist (`mixed-stateless-supported-resource-state-needs-review`, `mixed-stateless-supported-runtime-sensitive`, `mixed-stdlib-leaf-plus-runtime-sensitive`) would prevent typo-drift.

5. **Duplicate error reporting.** If `opaque_resource_matrix` were missing from the matrix, `_matrix_category` appends a "missing row" failure once per surface that requires it (8 times). Cosmetic only.

6. **`_sifr.python` does not require `async_runtime_reqwest`.** Python interop today is typically synchronous at the boundary and the ownership blocker doesn't mention async runtime, so this is defensible — but worth a sanity check with the certification owner if Python asyncio bridging is on the roadmap.

No required fixes. The wave is ready to open a PR as-is.
