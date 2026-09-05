# Wave 6.1 CPython Generated Differential Review

## Verdict
**Approve with required follow-ups.** The slice implements the generator, serializer, runner, shrinker artifact, lint extensions, and profile wiring against the acceptance criteria. No structural blockers, but one timeout-budget bug and a couple of contract gaps should be fixed before PR.

## Blocking findings
None.

## Required follow-up before PR

1. **Suite deadline starts *before* the release build, causing the build to consume the suite budget.**
   `verification/areas/cpython_differential/checks/generated_suite.py:63-64` sets `deadline = time.monotonic() + overall_timeout_seconds` and then calls `build_release_binary` (which has its own `build_timeout_seconds: 300`). On a cold cache, a 60–120 s cargo build subtracts from the 180 s (`generated_minimized_seeds`) or 240 s (`generated_broader`) overall budget, so the per-case loop at `:80` can trip `f"{suite_name} exceeded overall timeout"` before running any case. Move the `deadline = …` assignment to *after* `build_release_binary` returns successfully, or document `overall_timeout_seconds` as case-wall-time only.

2. **`tomllib` is imported at module top (line 10) but `validate_python_version` is the thing that's supposed to detect a too-old Python.** On Python 3.10 the runner ImportErrors before producing the documented `requires-python` failure. Move the `import tomllib` inside `validate_python_version`, or check `sys.version_info` first and short-circuit.

## Optional follow-up

- **Shrinker is real only for `arith_branch`.** `minimized_candidate` (`generated_programs.py:237-282`) returns `generate_program({"seed": 1, "shape": …})` for the other three shapes — that's the seed-1 generator output, not a smaller witness. Acceptance only requires a minimized artifact path, so it passes, but the wave title promises a shrinker. Worth a real shape-aware reducer in a follow-up.
- **Release binary builds twice when both suites run in one profile invocation.** Each adapter entry (`generated_minimized_seeds.py`, `generated_broader.py`) spawns its own process and re-invokes `cargo build --release -p sifr`. Cargo is idempotent so the second run is fast, but acceptance text says "built once." Either share state (timestamp file, env hint) or update the language to "built once per suite invocation."
- **Source digest inputs miss per-crate `Cargo.toml`s.** `generated_seed_manifest.json:15` lists `["Cargo.toml", "Cargo.lock", "crates/**/*.rs"]`. A feature-flag toggle in a per-crate manifest that doesn't perturb `Cargo.lock` would not change the digest. Consider adding `crates/**/Cargo.toml`.
- **Shape vocabulary duplicated.** `SUPPORTED_SHAPES` in `generated_programs.py:9-14` and the literal `{"arith_branch", "string_choice", "list_tuple_loop", "dict_sorted"}` in `catalogue_lint.py:230` will drift. Centralize the list (e.g. import the constant in the lint, or load it from the manifest).
- **`forbidden_exclusions` duplicated verbatim 10×.** All 12 IDs appear identically on every case; the lint enforces exact-match. A manifest-level default + per-case override list would shrink the JSON without weakening the contract.
- **`minimized_failures.json` entry schema is undefined.** The lint only checks `entries` is a list. Define an entry shape (suite, case_id, seed, minimized paths, regression PR link) now so the first real failure doesn't bikeshed format.
- **Hardcoded `">=3.11"` in `validate_python_version`** is brittle if `requires-python` ever moves. Either parse the version constraint or add a code comment pointing at the policy source of truth.
- **"compile-error" bucket is Sifr-only by construction** (`error_presence` only emits it when `runtime == "Sifr"` and stderr contains `error[`). Worth a one-line comment in the function so future readers know CPython will never report it.

## Another agent round required
**No.** Fix the deadline/build-ordering bug (item 1) and the `tomllib` import order (item 2), then proceed to PR. The rest is incremental cleanup that fits a Wave 6.2 follow-up.
