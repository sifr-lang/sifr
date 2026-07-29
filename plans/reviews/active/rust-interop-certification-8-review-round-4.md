## Review round 4 — `certification_8` (crate-backed advanced data runtime)

Re-reviewed the complete working tree against `origin/main` (merge-base `94a5fec67`; the branch has no commits — the milestone is entirely uncommitted + 13 intended untracked scenario files, the new driver test module, and two new checker modules). No files modified.

### Scope confirmation

Comparing mtimes against the round-3 artifact (`15:36`), exactly two files changed after round 3: `verification/areas/rust_interop/checks/_scenario_checks.py` and the new `_scenario_lock_checks.py` (both `16:37`). Every Rust, fixture, matrix, claims, and doc file is byte-identical to what round 3 certified. So the audit below is the cache in depth plus independent re-verification of the milestone-level invariants.

### Lock-cache audit — clean

**Stale reads.** `_load_root_lock` (`_scenario_lock_checks.py:11-18`) is keyed on `Path` and called from exactly one site with the module constant `REPO_ROOT / "Cargo.lock"` (`_scenario_checks.py:162, 489`). `REPO_ROOT` is derived from `__file__` and is never parametrized, so `maxsize=1` never thrashes and never aliases a different file. I grepped every write in `verification/areas/rust_interop/checks/*.py`: all mutations (`_scenario_checks.py:283,291,298,306,353,361`, `_scenario_advanced_data.py:302,310,320`, and the zero-copy/reqwest/callback equivalents) target `fixture_dir` under a `tempfile.TemporaryDirectory`. Nothing in the checker process writes the repository root lock, so the cached parse cannot go stale.

**Fail-open.** Missing file → `(None, None)` with no failure appended (`:13-14`). This is exact behavior parity with the replaced `_read_toml` (`_scenario_checks.py:747-748`, same silent `None`), so it is not a regression — and it is self-test-covered in the negative direction: if `read_root_lock` ever returned `None`, `require_root_lock_subset` is skipped, the `"root lock drift"` mutation (`_scenario_checks.py:270-276`) would stop reporting `"not present in root Cargo.lock"`, and `run_self_test` fails. The opposite break (returning an empty/partial dict) trips the baseline assertion at `_scenario_checks.py:157`. The cache is genuinely pinned from both sides by existing coverage.

**Error-report drift.** Old message: `f"{fixture_id}: {raw_path}/{path.name} is not valid TOML: {error}"` with `raw_path="repository root"`. New (`:30`): `f"{fixture_id}: repository root/{path.name} is not valid TOML: {error}"`. Character-identical output. Critically, `failures`/`fixture_id` are on the **uncached wrapper**, not on `_load_root_lock` — so a malformed root lock still appends one failure *per fixture*, not once for the whole run. That's the right split.

**Subset strength.** `require_root_lock_subset` is a verbatim move of `_require_root_lock_subset` (diff shows pure deletion + relocation, no logic edit). The scenario-side lock is still parsed fresh on every validation via `_read_toml` (`_scenario_checks.py:483`), so mutated scenario locks are always re-read. Nothing is weakened.

**Independent reruns.** `PYTHONPATH=verification/runner python3 verification/areas/rust_interop/runner.py` → `variants=10, failures=0, blocking_failures=0`, `cases=152`, `rows=36 fixture_rows=36`, `claims=31`. `check_file_size_guardrails.py` → PASS (2961 files, limit 900); `_scenario_checks.py` is 864, `rust_interop.rs` 898. `python3 -m compileall` on the checks package and `git diff --check` both clean. Recomputed matrix categories directly: `18 supported / 12 supported-through-bridge / 1 unsupported-by-design / 5 future-owned`.

### Round-3 findings — status

- **F1 (sysroot doc drift) — resolved, and correctly over-resolved.** `internal_docs/sifr_sysroot_and_stdlib_architecture.md:151-161` now reads 18/12/1/5 and names exactly `ecosystem_backend_certification, ecosystem_cli_certification, native_build_script, proc_macro_trust, cargo_locked_offline`. I recomputed the future-owned set from the matrix — it matches those five exactly. This also cleared the pre-existing drift left by certifications 5–7.
- **F2 (plan/artifact hygiene) — resolved.** The plan now links rounds 1, 2, and 3 (`…certification.md:928-940`), six of seven boxes are `[x]`, and the final gate/merge box is correctly still `[ ]`.

### Findings

All non-blocking; none is an acceptance-criterion violation.

**N1 — `_load_root_lock` hands the same mutable dict to every caller** (`_scenario_lock_checks.py:16`, severity: low, hardening). Memoizing a parsed structure without a defensive copy means any future consumer that mutates `root_lock` silently corrupts every subsequent fixture's subset check. Today the only consumer is read-only (`require_root_lock_subset:43-56`), so there is no live defect — but the previous per-call parse made this class of bug impossible by construction, and the cache removes that guarantee. A docstring warning or returning a frozen `(name, version)` set instead of the raw dict would restore it.

**N2 — only the parse was memoized; the `root_packages` set is still rebuilt per call** (`_scenario_lock_checks.py:43-47`, severity: informational). Roughly 700 lock entries re-scanned on each of ~150+ validations. The parse was the dominant cost and 6811ms/10000ms is comfortable, so this is a further win available, not a problem. Caching the derived identity set (rather than the dict) would also resolve N1.

**N3 — import ordering nit, carried from rounds 1–3** (`_scenario_checks.py:15-19`). `_scenario_advanced_data` is still placed after `_scenario_async_reqwest`, breaking the file's otherwise alphabetical block. The new `_scenario_lock_checks` import (`:25`) *is* correctly ordered. Cosmetic; no linter in the gate flags it.

**N4 — round-4 artifact is 0 bytes** (`plans/reviews/active/rust-interop-certification-8-review-round-4.md`), and rounds 1–4 remain untracked. Expected mid-flight; I did not write it, per instruction.

### Carried-forward observations (all re-confirmed, none actionable)

`dlpack::Capsule` and `sifr_arrow_bridge::schema::RecordBatch` remain never-constructed markers satisfying only the validator's crate-name prefix check (public-doc caveat present at `docs/rust-interop.mdx:187-188`); the `"…was already transferred"` branches stay unreachable because `own` consumes the handle; N4-from-round-1 (path precision recorded in `trust_validation.rs:16` but flattened in `materialize.rs`) is unchanged and not a new bypass; the x86_64 native-link leg is still derived from the pinned `blake3-1.8.5/build.rs` rather than executed here, which subset enforcement makes a fail-closed build error rather than a false certification.

### Out of scope — preserved unmodified

`editor_integrations` submodule bump, dirty `verification/areas/algorithmic_compatibility/corpora/leetcode` submodule, `plans/phases/43_interoperability.md`, `.cert5probe/` (5 files), `.claude/` (2 files), and the two stray `.webp` files. `git status -uall` otherwise shows exactly the 13 intended scenario files, the new driver test module, and the two checker modules.

The refactor is a faithful extraction: identical logic, identical error text, a cache whose only key is a file the checker provably never writes, and both failure modes already pinned by the existing `root lock drift` and baseline self-test cases. I relied on the provided crate-test/E2E/Python-interop evidence for the Rust surface, which is unchanged since round 3, and re-ran the rust_interop area and guardrails myself.

SATISFIED
