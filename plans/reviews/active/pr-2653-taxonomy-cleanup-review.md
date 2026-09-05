I have enough to issue the verdict. The taxonomy check that the PR claims to extend now fails on the PR branch itself.

## Code Review — PR #2653 "Clean delivery taxonomy from codebase surfaces"

### Findings (ordered by severity)

**1. BLOCKER — the strengthened taxonomy check fails on this PR branch.**
The PR's own validation claim (`python3 verification/areas/coverage_matrix/checks/verification_taxonomy.py: pass`) does not reproduce. Running the script on the head commit emits 34 errors and exits with code 1. The leaks the script catches are all in active surfaces the script itself opted to scan:

- `.cursor/skills/phase-closure-loop/SKILL.md` — 26 violations, file untouched by this PR. The file is structured entirely around "wave/milestone/phase closure" cycles — exactly the taxonomy this PR bans. It should be deleted or renamed/rewritten to neutral execution vocabulary, or moved under `plans/`.
- `.cursor/skills/project-workflow/SKILL.md` — 6 violations (lines 12, 18, 58, 60, 62), file untouched by this PR. Contains "Milestone Demo", "work items", "<milestone>_demo".
- `AGENTS.md:93` — "Keep changes focused on the requested milestone/issue." This PR edited AGENTS.md but missed this line.
- `AGENTS.md:113` — "Roadmap: `plans/roadmap.md`" — same: this PR edited adjacent lines but left this one. The PR also removed line 84's "Roadmap, phase plans" wording, so leaving "Roadmap:" here is inconsistent.

**2. BLOCKER (self-inflicted) — the PR introduces a false positive in its own guidance text.**
`.cursor/skills/sifr-demo-authoring/SKILL.md:60` was edited by this PR (commit f44bfe08b) from
`"Remove process, planning, milestone, or problem framing"` to
`"Remove process, phase, milestone, wave, or problem framing"`,
and the new wording trips the taxonomy regex it was meant to support. Either the line needs to be rephrased to avoid literal tokens (e.g., bullet list with backticked code-style examples behind a comment, or a placeholder like "delivery-process taxonomy") or the script needs an explicit allowlist for self-referential guidance.

**3. Inconsistency agent's "passed" report was misleading.**
The validation log in the PR description should not be trusted as a pre-merge gate without re-running. The merge gate `scripts/run_all_tests.sh` is the right oracle — agent notes only "representative p95 budget" failures there, but a hand re-run of `verification_taxonomy.py` shows that script also fails. Worth confirming whether `run_all_tests.sh` actually drives this check; if not, the new enforcement is not wired into the gate it needs to live behind.

### Areas that look clean (no blockers)
- Verification schemas (`area.schema.json`, `profile.schema.json`, profile JSONs, runner Python) — keys consistent, no stale `contract`/`phase`/`wave`/`milestone` references in active runner code.
- Renamed `verification/areas/*/reports/*.md` — no dangling references in active tree (only archived `plans/` mentions, which are exempt).
- Rust crate changes — pure mechanical terminology renames in user-facing strings and one test name; `Cargo.toml` change is comment-only; `build_output_contracts.rs` → `build_output_behavior.rs` is a clean integration-test rename (Cargo auto-discovers; no `mod` wiring needed).
- ~100 `.sifr` e2e fixtures — comment-only `# Reference:` updates; no snapshot drift risk (these are metadata lines, not source).
- Submodule SHA bumps for `editor_integrations` (7e972f2 → d03a2f4) and the LeetCode corpus (101df22 → a20d9d5) — terminology-only commits; the LeetCode field rename `related_slowness_phase → has_slowness_seed_entry` is not consumed by the parent repo.

### What needs to change before merge
1. Clean or remove `.cursor/skills/phase-closure-loop/SKILL.md` and `.cursor/skills/project-workflow/SKILL.md`, or formally scope them out of the scanner with a documented allowlist (not preferred — they describe live workflows).
2. Fix the remaining two lines in `AGENTS.md` (93, 113).
3. Resolve the self-collision in `.cursor/skills/sifr-demo-authoring/SKILL.md:60`.
4. Re-run `python3 verification/areas/coverage_matrix/checks/verification_taxonomy.py` and confirm exit 0 before re-requesting review.

Verdict: BLOCKED
