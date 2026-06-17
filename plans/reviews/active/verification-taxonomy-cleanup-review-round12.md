## NOT SATISFIED — two blocking findings

### Blocking

**B1. Active delivery-taxonomy token slipped past the gate.**

`verification/areas/diagnostics/checks/code_baseline_coverage.py:227` —

```python
for field in ("owner", "reason", "issue", "expires_in_wave"):
```

`"expires_in_wave"` is a baseline-deferral schema field whose name embeds the delivery-taxonomy word `wave`. The taxonomy gate at `verification/areas/coverage_matrix/checks/verification_taxonomy.py:67` is `\b(?:wave|...)\b` (case-insensitive) — verified locally that this regex does **not** match `expires_in_wave` (the `\b` before `wave` requires a non-word char, but `_` is `\w`). The gate's `LEGACY_FIELD_PATTERNS` lists `closes_in_wave` and `closes_in_subwave` (verification_taxonomy.py:62-63) but not the `expires_in_wave` variant. So:
- The field name itself is the leak you said to eliminate.
- The gate is too weak here — it caught one underscore-prefixed `wave` variant but missed the sibling. The pattern set should include `expires_in_wave` (or relax the boundary, e.g., a `(?:^|[^A-Za-z0-9])wave(?:[^A-Za-z0-9]|$)` form, which would also catch any future `_wave` neologism).
- Whichever way: the field needs renaming (e.g., `expires_at`/`removal_target`) and the gate needs to learn the boundary rule so this never recurs.

**B2. Gate scope carves out `.cursor` too narrowly; a sibling skill still prescribes the taxonomy.**

`verification_taxonomy.py:19-20` only scans `.cursor/commands/create-new-version.md` and `.cursor/skills/codebase-closure-loop`. The siblings `.cursor/skills/project-workflow` and `.cursor/skills/sifr-demo-authoring` are excluded.

In that gap:
- `.cursor/skills/project-workflow/SKILL.md:60` — `Before marking a milestone (Epic) as Done, create a demo in ./demos named <milestone>_demo (e.g., m3_demo).` Confirmed both `<milestone>` and `m3_demo` would be flagged by the existing patterns (matched locally with `\b(?:wave|milestone|...)\b` and `\bm\d+[_-][a-z0-9][a-z0-9_-]*\b`) — they only survive because the file isn't scanned.
- `.cursor/skills/project-workflow/SKILL.md:62` — "all major features delivered in that milestone".

This is an active instruction telling future contributors to materialize `m3_demo`-style directories — the exact taxonomy you're removing. The skill is also referenced from `AGENTS.md` ("Follow `.cursor/skills/project-workflow/SKILL.md`"). Either widen the gate roots to `.cursor/skills/**` + `.cursor/commands/**` and update the skill to drop milestone framing, or commit to a deliberate carve-out (and put that decision in writing). Right now it's a silent hole.

### Non-blocking

1. `verification/areas/diagnostics/fixtures/diagnostics/{presentation,package_library_diagnostic,frontend_internal_diagnostic}_contract_cases/` — these group cases that pin diagnostic-API behavior, consistent with `shipped_guarantees.json` IDs like `diagnostic-contract`/`parser-syntax-contract`. They read as real API contracts, not renamed waves, so they're acceptable, but `*_contract_cases` is the closest thing left to the "contract = bucket" shape — if the team wants belt-and-braces clarity, rename to `*_contract_fixtures` (or `*_behavior_cases`).
2. `.cursor/skills/sifr-demo-authoring/SKILL.md:60` uses the word "milestone" inside an instruction to *remove* milestone framing — self-aware, fine, but file is also outside gate scope; folding it into the scope expansion in B2 closes the loop.
3. `internal_docs/architecture.md` carries durable references to historical "phase docs" under `plans/issues/archive/`. Those are anchored to archived material and are fine; just noting it's the only "phase" survivor in `internal_docs`.

### What's already clean (verified)

- All seven renames are coherent end-to-end: Cargo test discovery (`crates/sifr/tests/validation_suites.rs` + `validation_suite_support/`), env vars (`SIFR_VALIDATION_SUITE_MANIFEST`/`_FILTER` in both `manifest.rs:6-7` and `area_adapter.py:267-268,681`), profile-runner step `validation_suite_matrix` (`profile_runner.py:153`), area-adapter key `"validation_suites"` (`area_adapter.py:255`), classification entry (`cargo_metadata_classification.json:11`), `workspace_validation_suites.json` at the renamed path, no `validation_contracts` dir, no `SIFR_VALIDATION_CONTRACT_*` leftovers, no `phase-closure-loop` references, no `m1_`/`m2_` prefixes in active docs.
- Compiler-internal `phase` uses (`WorkspaceTracePhase`, `LintPhase`, `ProgressPhase`, build/parse/codegen phase prose in `docs/cli/build-run.mdx`) are legitimate algorithmic terminology and explicitly allowlisted at `verification_taxonomy.py:46-48`.

Fix B1 and B2 and this is good to ship.
