## Review — `hardening_1` (Rust-interop area in authoritative profiles)

**What's right.** The step is scheduled in the *same* registry execution uses: `run()` now consumes `legacy_facade_steps()` (`profile_runner.py:185,196`), and the self-test introspects that exact method, so deleting the step genuinely fails the self-test. Ordering is preserved. Staleness is handled at the source — `result_path.unlink(missing_ok=True)` before the run (`:477`), a per-profile result filename, and `--result-json` correctly plumbed through `areas.run_area` → `area_adapter.run_area`. Failure propagation is real: `blocking_failures > 0` → exit 1 → `CommandFailed`. Suite/area name typos in profile JSON are already rejected against the manifests. Budget measured at 0.43–0.60 s over three warm runs against 5000 ms blocking — appropriate headroom, no concern. `--emit-plan` confirmed to contain the selection for all four profiles; self-tests pass.

### Findings

**1. Medium-high — the new self-test can shell out to `cargo build` and leaks global env** (`selftest.py:293`)
`ProfileRunner.__init__` calls `resolve_sifr_binary(...)`, which rebuilds the debug binary when it is missing *or stale* (`sifr_binary.py:36-38`; `_binary_is_stale` rglobs `crates/`, `stdlib/`, `third_party/ruff/crates`). Constructing a runner just to read a static step list drags that in. Verified by stubbing `_build_sifr_binary` and pointing `CARGO_TARGET_DIR` at an empty dir:

```
cargo build invocations triggered by self-test: ['<tmp>/debug/sifr']
CARGO_NET_OFFLINE leaked: true
PROBE_CACHE leaked: .../target/sifr_rust_bridge_probe_cache/release
SIFR_GCQ_BIN leaked: <tmp>/debug/sifr
```

Failure scenario: on a clean checkout, or after editing any file under `crates/`, the mandated `uv run --project verification --locked python -m sifr_verify --self-test` turns from milliseconds into a full debug build; if that build fails it exits via `SystemExit(rc)` from `sifr_binary`, attributing a cargo failure to the runner self-test. It also permanently sets `CARGO_NET_OFFLINE=true` and pins the probe cache to the `release` profile for the rest of the process. In-lane this is masked because the parent runner already resolved the binary into `os.environ`, so it is standalone-only — but standalone is exactly how the plan's Required Validation invokes it.
Fix: make step scheduling a pure function over profile data (it only needs `legacy_facade(profile)["generated_code_quality"]`), or make binary resolution lazy in `ProfileRunner`.

**2. Medium — required suite set is hardcoded in three places, not derived from the area manifest** (`profiles.py:169-173`, `selftest.py:276-281`, four profile JSONs)
Adding a fifth suite to `verification/areas/rust_interop/manifest.json` leaves it unexecuted in all four authoritative lanes while both the profile validator and the self-test still pass — the area would be under-executed silently, which is the failure mode this issue exists to prevent. The sibling `python_interop` path derives its required set from the capability matrix (`_compiled_evidence_suites()`). Derive the rust_interop set from the manifest's suite names the same way.

**3. Medium — result-JSON mutation coverage is one-sided** (`selftest.py:311-329`)
Only the missing-file branch is exercised. The branches that actually defend against a stale, truncated, or foreign artifact — suite-set mismatch, `area != "rust_interop"`, non-list `suites`, malformed JSON — have no test at all, so a regression that neuters them (e.g. `!=` → subset check) passes the self-test. Given the stated constraint "cannot silently accept missing or stale result artifacts", add at least the mismatch case: write a three-suite payload and assert it is rejected.

**4. Low — the guard validates identity, not evidence** (`profile_runner.py:64-85`)
`validate_rust_interop_result` never asserts `summary.blocking_failures == 0`, `suite["blocking"] is True`, or `total_variants > 0`. Today the area's nonzero exit covers real failures, so this is defense-in-depth only — but the function's name implies more than it checks, and it would not notice a zero-exit-with-failures path (e.g. anything bless-like) added later.

**5. Low — one silent-skip path remains** (`profile_runner.py:466-468`)
`validate_selected_area_suites` only enforces the four suites *if* a `rust_interop` selection exists; nothing requires a legacy-facade profile to select the area at all, and the step then prints `Skipping ...`. Only the hardcoded four-name loop in the self-test catches this, so a future legacy-facade profile would skip the area silently. Consistent with sibling steps and with the plan's literal wording, so acceptable — flagging because "no skips" was a stated constraint.

**6. Low — README omits the executing profile command** (`verification/areas/rust_interop/README.md`)
It gives the direct command and `--emit-plan`, but not `scripts/run_all_tests.sh --profile create-pr`, which is the command the exit gate names. Separately: the issue's own Required Validation line `python -m sifr_verify --area rust_interop` is not a valid CLI form — `--area` exists only under `areas run`. The README is correct; the plan text is stale and should be corrected (fine to defer to `hardening_5`).

### Verdict

**Not ready as-is.** Every hardening_1 bullet and exit-gate item is satisfied in substance, and the central design question — same registry for scheduling and execution, no stale-artifact acceptance — is answered correctly. But finding #1 is a genuine regression in a command the plan mandates, and #2/#3 leave the guarantee weaker than it reads. Fix #1, #2, and #3 in this PR (all small and local), then it is ready for create-PR validation. #4–#6 are optional.
