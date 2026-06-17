## Review findings

### Blocking

**1. `verification_taxonomy.py:79-81` — case-sensitivity bug in the three `\s+\d+` patterns**

```python
re.compile(r"\bPhase\s+\d+\b"),
re.compile(r"\bMilestone\s+\d+\b"),
re.compile(r"\bWave\s+\d+\b"),
```

These three patterns are NOT `re.IGNORECASE`. Every other taxonomy pattern in the file is. As a result, lowercase `phase 29` / `milestone 5` / `wave 1` (with a space before the number) is invisible to the guard. The "phase 99 closeout" / "Milestone 99" self-test bad samples only use capitalized forms, so this gap is not exercised.

This causes the next set of blocking leaks to slip through despite being inside the expanded `ACTIVE_ROOTS`:

- `internal_docs/architecture.md:941-943` — `ad-hoc phase ... (wave 1+)`, `wave 1+, builtin lowering in wave 2`, `wave 0 lock, wave 1+ implementation`.
- `verification/areas/stdlib_parity/reports/stdlib_bytes_architecture_lock.md:16-17` — `wave 2 lands`, `during wave 3`.
- `verification/areas/stdlib_parity/reports/stdlib_iterator_architecture_lock.md:36` — `baseline mismatch captured at wave 0`.
- `verification/areas/stdlib_parity/reports/stdlib_rng_architecture_lock.md:20` — `bytes-first constructor entry for wave 2`.
- `verification/areas/stdlib_parity/reports/stdlib_bytes_cpython_{1..5}_traceability.md` — `Classified waivers carried from wave N`, `Local fixture anchors (wave N)`, `Governance closeout anchors (wave 5)` (~15 lines across the five files).
- `verification/areas/stdlib_parity/reports/stdlib_iterator_cpython_{1..5}_traceability.md` — repeated `adapted (closed in wave N ...)` rows (~14 lines).
- `verification/areas/stdlib_parity/reports/stdlib_runtime_cpython_0_traceability.md:34` and `stdlib_runtime_cpython_3_traceability.md:20` — `wave 3 owns ...`, `locked from wave 0`.

I verified directly via `validate_text(Path('.../stdlib_bytes_cpython_1_traceability.md'))` → 0 failures, and `validate_text(Path('internal_docs/architecture.md'))` → 0 failures. The script's PASS is masking these.

**Fix:** add `re.IGNORECASE` to the three patterns (or merge them into the existing IGNORECASE pattern that already handles the no-space variant), and add lowercase samples to `run_self_test` so the regression sticks.

**2. `verification/areas/fuzz_property/sustained_lane.md:9` — out of scope AND case-bug victim**

```
- non-blocking for merge decisions in phase 29
```

`verification/areas/fuzz_property` is parent-owned (no submodule, no corpora subtree there) but is missing from `ACTIVE_ROOTS`. Even if it were added, the lowercase `phase 29` would still slip through because of finding #1. Two issues compounded — both need to land for this to be caught.

### Non-blocking suggestions

**3. Bare-`m\d+` leaks in demos (intentional gap vs. real leak — please confirm)**

Patterns require a `[_-]` separator after `m\d+`, which intentionally lets local variables like `let m1 = ...; let m2 = ...` (e.g. `demos/stdlib_tools/idiomatic.rs:114-115`) pass. But the same exemption hides what looks like delivery-plan taxonomy:

- `demos/codegen_preamble/main.sifr:14,19,24` (+ `idiomatic.rs:38,43,48`, `emitted.rs:969,1019,1027`) — string literals `"m14 preamble"` and `getLogger("m14")`.
- `demos/safety_basics/main.sifr:1` — `# Reference: m0`.

The `m14`/`m0` tokens look like milestone references rather than language semantics. If they are, they are taxonomy leaks; if `m14` is the demo's intended identity string for some other reason, leave them. The guard cannot distinguish without a separator rule.

**4. ACTIVE_ROOTS coverage gaps (parent-owned but unscoped)**

Verified currently clean of taxonomy, but adding them prevents future drift:
- `verification/areas/algorithmic_compatibility`, `verification/areas/cpython_differential`, `verification/areas/ecosystem_compatibility` (only their non-corpora content; corpora are submodules).
- `verification/areas/diagnostics/{checks,fixtures,manifest.json,runner.py}` — only `data/` is in scope.
- `verification/areas/core_language/fixtures` — only `checks/` and `data/` are in scope.
- `verification/areas/stdlib_parity/{checks,fixtures,manifest.json,runner.py}` — only `data/docs/reports/tools` are in scope.
- `verification/{policy,schemas,README.md,owners.json,pyproject.toml,uv.lock}`.

If the intent is "any parent-owned non-plan surface," consider rooting at `verification/` itself with the existing skip logic doing the corpora exclusion, rather than enumerating subfolders.

### Confirmed clean (previous round fixes landed)

- `check_phase36_closeout.py` references only remain in `plans/issues/archive/*` and `plans/reviews/archive/*` (both correctly excluded).
- `verification/areas/project_workspace/data/validation_contracts/manifest.json` — `m22_4`/`m23_5`/`Phase 23 graph...` rewritten to `contract22_4`/`contract23_5`/`project graph isolation regression matrix`.
- `verification/areas/project_workspace/data/workspace_contracts.json` — `phase23-graph-isolation` → `project-graph-isolation`.
- `verification/areas/performance/data/{baselines,benchmark_manifest,budgets}.json` — all `phase27-non-regression*` and `phase34-*` evidence categories renamed; new files grep clean for any `phase|milestone|wave|M\d+`.
- `.github/workflows/preview-release.yml` — `Phase 33` and `Phase 39` strings removed.
- `.cursor/commands/create-new-version.md` — `Phase 33` references replaced.
- `.cursor/skills/codebase-closure-loop/SKILL.md` — properly renamed via `git mv` (R100), content reworked from phase/wave language to contract/slice.

### Rename coherence

- 17 `R100` renames for `internal_docs/typescript_go_architecture_transfer_m{2..17}_*.md` → de-m'd names. The `m1_guardrails.md` was renamed to `guardrails.md` (visible in `ls`). No dangling references to old names outside `plans/{issues,reviews}/archive/*` (excluded).
- 1 `R100` rename for `.cursor/skills/phase-closure-loop` → `codebase-closure-loop`. Body updated.
- `editor_integrations` is a submodule (`.gitmodules`, `git submodule status` confirm) and is correctly absent from the parent diff and from `ACTIVE_ROOTS`.

### Bottom line

Blocking: finding #1 (case-insensitivity gap) is the root cause; finding #2 piggybacks on it. The guard is currently green only because lowercase `wave N` / `phase N` is invisible to it. Once you add `re.IGNORECASE` to the three space-separator patterns and re-run, ~30+ real lines across `internal_docs/architecture.md` and `stdlib_parity/reports/*` will surface and need scrubbing (or an explicit allowlist of stdlib parity "wave" terminology if you decide to grandfather it).
