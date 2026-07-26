## Review pass 2 — `hardening_1` (Rust-interop area in authoritative profiles)

Reviewed the complete intended diff (10 files, +340/−30) — the paths listed match exactly; no stray files. Ad-hoc class-field review artifacts ignored.

### Pass-1 findings: all six resolved

| # | Pass-1 finding | Status | Evidence |
|---|---|---|---|
| 1 | Self-test shells out to `cargo build`, leaks env | **Fixed** | Step registry is now pure data (`profile_runner.py:64-92`, `legacy_facade_step_methods`); `selftest.py` imports only `legacy_facade_step_names` / `validate_rust_interop_result` — no `ProfileRunner`, no `resolve_sifr_binary`, no `os.environ` touch. Standalone `--self-test` runs in **0.157 s** total. `run()` consumes the same registry via `getattr` (`:257-262`), so scheduling and execution still share one source. |
| 2 | Required suite set hardcoded | **Fixed** | `required_rust_interop_suites()` (`profiles.py:186-199`) derives from `verification/areas/rust_interop/manifest.json`, with empty/invalid/duplicate rejection. Verified live: injecting a 5th manifest suite fails all four profiles (`profile create-pr omits required Rust interop verification suites: runtime-ecosystem`); a duplicate name yields `manifest has invalid or duplicate suites`. |
| 3 | One-sided mutation coverage | **Fixed** | `selftest.py:326-385` covers missing file, foreign `area`, incomplete suite set, non-list `suites`, `blocking_failures: 1`, and malformed JSON, plus a positive payload. |
| 4 | Guard validated identity, not evidence | **Fixed** | `validate_rust_interop_result` now requires per-suite `blocking is True`, `total_variants > 0`, `total_failures == 0`, plus `summary.blocking_failures == 0` and `summary.total_variants > 0`. Field names verified against the real emitter (`area_adapter.py:71-89,151-159`). |
| 5 | Silent-skip path if a profile omits the area | **Fixed** | `profiles.py:177-185` requires `rust_interop` in `selected_areas` for every non-`selected-areas-only` profile; `python-interop-live.json` (the only `selected-areas-only` profile) remains exempt. Self-test asserts both the missing-area and missing-suite rejections. |
| 6 | README/plan commands | **Fixed** | README adds direct, `--profile create-pr`, and `--emit-plan` commands; all three verified to exist (`run_all_tests.sh:23-24`, `areas run --result-json` at `areas.py:126`). Both issue plans corrected to `python -m sifr_verify areas run --area rust_interop`, which I ran successfully. |

Registry-deletion still fails loudly: removing the step entry in-memory made the self-test raise `create-pr omits the executable Rust interop step`.

### Validation re-run locally

- `py_compile` on all three runner modules: pass
- `python -m sifr_verify --self-test`: 8/8 pass, incl. the new *Rust interop profile execution self-test*
- Profile validation: `load_all_profiles()` clean; coverage-matrix profile-policy area passes (5 variants, 0 failures)
- Direct area: `variants=4, failures=0, blocking_failures=0` (all four suites)
- Focused step via the real code path: `[sifr-lane-step] name=rust_interop_checks elapsed_ms=504 status=pass` against a 5000 ms blocking budget (≈10× headroom)
- `--emit-plan` for create-pr/merge/nightly/release: each contains the four-suite `rust_interop` selection
- `git diff --check` clean; file-size guardrail PASS (2821 files)
- Not run here: the full `scripts/run_all_tests.sh --profile create-pr` lane and `cargo clippy`/`fmt` (untouched by this diff) — run the create-PR lane before opening the PR, as the exit gate requires.

### Findings

No actionable findings. Three optional observations, none affecting the shipped guarantee:

1. **Optional (test-coverage)** — `profile_runner.py:74-84`: the per-suite evidence branch added for pass-1 #4 has no mutation case; a regression deleting it still passes the self-test (the summary mutation only covers `summary.blocking_failures`). Two more entries in `invalid_payloads` (`blocking: False`, `total_failures: 1`) would close it. Beyond the mandated mutation set.
2. **Optional (dead branch)** — `profile_runner.py:503-506`: the `Skipping Rust interop checks` path is now unreachable for any validated legacy-facade profile, since `profiles.py` requires the area *and* all manifest suites. Raising instead of printing would remove the last skip-shaped fallback.
3. **Forward note** — requiring *every* manifest suite makes any future rust_interop suite (e.g. the external-crate runtime suites planned in `rust-interop-runtime-ecosystem-certification.md`) mandatory in create-PR, and the step executes selected suites without resource-class filtering. The failure is loud, not silent, so this is the correct default — just expect that issue to have to either scope such suites into create-PR or introduce an explicit profile-tier split.

### Verdict

**Approved.** All pass-1 findings are resolved at the root, not papered over; the scheduling/execution registry is single-sourced and pure, required suites are manifest-derived with invalid/duplicate rejection, result-JSON evidence is checked in substance with six mutation cases, and no silent-skip path remains for legacy-facade profiles. Ready for full create-PR validation and the PR.
