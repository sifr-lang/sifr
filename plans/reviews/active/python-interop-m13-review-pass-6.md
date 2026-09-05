All remaining diffs are benign (strict `false` flag on the run path, a digest re-export, visibility widening, and a `selects_environment` refactor). Verification is complete — here is my pass-6 closure review.

---

# M13 Closure Review — Pass 6 (PR #2993, `codex/m13-python-readonly-check-doctor`, HEAD `81763d521`)

**Bottom line: the pass-5 flaky-test blocker is fixed at the root and holds up under independent stress, the pass-4 catalog fix remains intact on all four surfaces, and the full behavioral, safety, docs, demo, validation, and evidence contract reproduces cleanly at HEAD.**

## Pass-5 blocker — confirmed closed at the root

Commit `81763d521` changes exactly what pass 5 prescribed, in `crates/sifr/tests/python_read_only_cli.rs`: a process-wide `static NEXT_FIXTURE_ID: AtomicU64` (line 9) whose `fetch_add` value is appended to the fixture path alongside pid and the nanosecond nonce (lines 77–81), and root creation switched from the collision-masking `create_dir_all` to `create_dir(&root)` so any residual collision fails loudly (line 82). This is structurally sound: pid + monotonic counter guarantees intra-process uniqueness (the exact pass-5 failure mode was same-pid, same-label, microsecond-granularity nonce), the nonce covers pid reuse across runs, and `create_dir` converts any impossible leftover case into a loud failure instead of silent fixture sharing.

**Independent stress.** I ran the compiled suite under default parallel execution 13 verified consecutive times: 12 loop runs plus one direct run, every one `test result: ok. 6 passed; 0 failed` (~42s each) — 78/78, against pass 5's baseline of 3 failures in 6 runs (probability of 13 straight passes at that failure rate ≈ 10⁻⁴). An earlier 15-run batch hit my own 10-minute timeout on run 15; forensics on the 5 leftover temp directories showed they all belong to pid 4782 at the kill instant (`Drop` never ran on the SIGTERM'd process — not a suite defect), and their names (`…-1784672408008878000-3`, `…-008894000-4`, `…-009107000-5`) show three fixtures created within ~230µs cleanly disambiguated by the atomic discriminator, i.e. the fix working exactly as designed. All earlier completed runs left zero leftovers, confirming cleanup. The plan doc's recorded eight consecutive 48/48 passes are consistent with what I measured.

**Merge-gate participation.** I ran the full blocking `sifr_cli_full` command (`cargo test -p sifr -- --skip test_e2e_pass`, the `cargo-test-sifr-full` toolchain step selected by `crate_tests: "full"` at `verification/profiles/merge.json:350`, `nightly.json:345`, `release.json:344`). The step completed with `python_read_only_cli` passing 6/6 in situ under default parallelism, the final test targets all `ok`, and no failure lines — under cargo's fail-fast semantics that means every earlier target passed too.

## Pass-4 catalog blocker — remains fixed

All four surfaces agree on `crates/sifr_package/src/python/probe_validation_tests.rs::probe_rejects_*` for SIFR-PYENV-0004–0011: `verification/areas/diagnostics/data/code_catalog.json` (verified programmatically for all eight codes), `crates/sifr_diagnostics/src/codes/registry/registry_entries/python_interop.rs:50-136`, all eight `docs/errors/SIFR-PYENV-00XX.md` pages, and `internal_docs/diagnostic_codes.md` (8 references). No stale `python/tests.rs::probe_rejects` reference survives outside the historical ledger quote. The module exists (151 lines, 10 tests, registered at `crates/sifr_package/src/python/mod.rs:52`).

## Full contract — independently re-verified at HEAD

- **Shared typed authority.** `resolve_python_environment_for_check` (`environment.rs:116`) is the single `NotRequired`/`Resolved`/`DeferredToFinalApplication` decision point; deferral requires the opt-in flag *and* only `PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED`/`PYENV_MISSING_SELECTION` errors (lines 187, 198). The strict wrapper's `unreachable!` (line 111) is provably dead — with the flag false both error paths return `Err` before a deferred outcome can be constructed. Build and run stay strict: `into_generated_binary_project` passes `false` (`entrypoint.rs:589`), as does `cmd_run_package_file`.
- **Blocking evidence suite.** The official runner passes at HEAD: `uv run --project verification --locked python -m sifr_verify areas run --area python_interop --suite readonly-check-doctor` → the exact recorded line `python interop read-only check/doctor ok: deferred=1 resolved=3 parity=5 mutations=0`, `variants=1, failures=0, blocking_failures=0`. Adapter-case failures are blocking by construction (`verification/runner/sifr_verify/area_adapter.py:76`), and the suite is selected in all four profiles (`create-pr.json:117`, `merge.json:109`, `nightly.json:122`, `release.json:121`). Its cases cover bare-library deferral parity with ordinary `check --frozen`, explicit and uv-discovered standalone resolution, multi-target application verification (`math.ceil` from `src/bin/secondary.sifr` plus `math.sqrt`), `SIFR-PYIMP-0001` failure parity on both surfaces, deterministic doctor output with one-sided patches, source-digest identity, and byte-level snapshot non-mutation.
- **Demo.** `demos/m13_python_read_only`: `python check --json` → `"status": "ok"`, environment resolved, trust verified; `python doctor` → `lock: verified-frozen-read-only`, deterministic snapshot; `sifr run` → `Python read-only check demo: target verified`; full-tree SHA-256 digest byte-identical before/after and `git status` clean.
- **Safety.** No `.unwrap()`/`.expect()`/`panic!` in any new production path (`python_cli.rs`, `python_runtime_context.rs`, `python_check.rs`, `report.rs`, `digest_source_map.rs`); the only `expect`s in `environment.rs` sit inside `#[cfg(test)]`. The two `Command::new` calls in `python_cli.rs` belong to the pre-existing M11 certify path and are read-only interpreter queries.
- **Guardrails and hygiene.** File-size guardrail PASS (2768 files, 900-line cap), HIR maintainability PASS, driver maintainability PASS, `cargo fmt --check` clean, and the documented gate `cargo clippy --workspace -- -D warnings` exits 0.
- **Docs and closure evidence.** `docs/python-interop.mdx:62-93` documents both commands and the never-writes guarantee; `internal_docs/python_interop_architecture.md` gains a Read-Only Inspection section matching the implemented semantics; all five review-pass ledgers are committed; the plan doc records the pass-5 remediation and eight consecutive 48/48 runs. Wave 4's checkbox stays open pending merge, matching prior milestones (roadmap records M13's PR link at merge, as it did for M10–M12).
- **No committed Ruff change.** `git diff main...HEAD -- third_party/ruff` is empty; both trees pin `8111415495`. The working-tree dirt is a semantically neutral one-line join in `expression.rs`, absent from the PR.

## Non-blocking notes

- `cargo clippy --workspace --all-targets -- -D warnings` (a stricter variant than any documented or gate-mirrored command) fails with 27 pedantic lints in `sifr_lowering` lib tests — a crate untouched by this branch, so pre-existing and outside every gate; worth a separate cleanup ticket.
- The untracked zero-byte `plans/reviews/active/python-interop-m13-review-pass-6.md` and `.agent-m13-pass6.log` are local placeholders, not in the diff.
- Carried from passes 3–5 as M14 candidates: the defensive `(false, true)` doctor arm and the dead `PythonInteropCheckReport.environment` CLI fields.

The pass-5 structural flakiness is eliminated at its root cause, the milestone's evidence suite now participates reliably in the authoritative merge gate, and every other contract item reproduces independently at HEAD.

VERDICT: SATISFIED
