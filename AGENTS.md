# AGENTS.md

## Project

Sifr compiles Python syntax to Rust and produces native binaries.

The compiler must prevent user-triggered runtime panics.

Pipeline: `sifr` CLI -> `sifr_driver` orchestrates `sifr_frontend` (parse/lower/type-check) -> `sifr_codegen` -> Cargo/`rustc` -> native binary.

Read `internal_docs/architecture.md` for architecture details.

## Work Boundaries

- Follow `.cursor/skills/project-workflow/SKILL.md`.
- Work on one item at a time.
- Use `.cursor/skills/phase-closure-loop/SKILL.md` for phase items.
- Solve root causes inside the approved scope (not superficial symptoms).
- Do not add backward compatibility unless the user requests them.
- Do not add fallback paths unless the user requests them.
- Do not absorb unrelated failures or externally owned dependencies.
- Record an out-of-scope failure in its owning issue.
- If an external failure blocks the item, record it and stop.

One session owns its worktree, branch, Git index, and temporary paths.

Do not let another session mutate them during validation or review.

## Code Rules

- Hand-maintained first-party source files must stay under 900 lines.
- Generated files, lockfiles, snapshots, baselines, `target/**`, and `third_party/**` are excluded.
- Split large modules by responsibility and ownership boundary.
- Do not use data-dependent `.unwrap()` or `.expect()` in generated runtime code.
- Use `assert!` only for programmer invariants.
- Keep `Cargo.lock` changes intentional.
- Use `insta` for snapshot tests.
- E2E fixtures run in lexical order.
- Snapshot expectations follow declaration order.

## Commands

```bash
# Run a Sifr file
cargo run -q -p sifr -- run <file>.sifr

# Unit tests without the slow E2E pass suite
cargo test -p sifr -- --skip test_e2e_pass

# E2E pass suite
verification/runner/e2e/run_e2e_pass.sh

# PR gate
scripts/run_all_tests.sh --profile create-pr

# Merge gate
scripts/run_all_tests.sh
```

Run targeted tests during implementation.

Run the PR gate before you open a PR.

Run the merge gate once on the final implementation candidate.

CI uses the same scripts.

Do not wait for CI instead of local validation.

## Cargo build storage

- Before a long Cargo gate, inspect free disk space and the private target size.
- If the private target exceeds 20 GiB, verify that no active process uses it.
- Run `cargo clean` only for a target owned by the current worktree.
- Do not clean shared targets or targets from other worktrees.
- Do not use a cold-cache run as performance evidence.
- If safe cleanup is insufficient, record the resource blocker and stop.

## Records

- Update the active issue after each merged item.
- Update `internal_docs/architecture.md` only when architecture changes.
- Update `plans/roadmap.md` only when roadmap status changes.

## Safety

- Do not use destructive git operations unless explicitly requested.
- Do not revert unrelated user changes.
- If unexpected repo modifications appear, stop and ask before proceeding.
