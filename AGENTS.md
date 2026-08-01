# AGENTS.md

## Project

Sifr compiles Python syntax to Rust and produces native binaries.

It enforces static types, `Result`/`Option` error handling, and compile-time ownership.

The compiler must prevent user-triggered runtime panics.

Pipeline: `sifr` CLI -> `sifr_driver` orchestrates `sifr_frontend` (parse/lower/type-check) -> `sifr_codegen` -> Cargo/`rustc` -> native binary.

Read `internal_docs/architecture.md` for architecture details.

## Work Boundaries

- Follow `.cursor/skills/project-workflow/SKILL.md`.
- Work on one item at a time.
- Use `.cursor/skills/phase-closure-loop/SKILL.md` for phase items.
- Solve root causes inside the approved scope (not superficial symptoms).
- Do not add backward compatibility unless the user requests it.
- Do not add fallback paths unless the user requests them.
- Do not absorb unrelated failures or externally owned dependencies.
- Record an out-of-scope failure in its owning issue.
- If an external failure blocks the item, record it and stop.

One session owns its worktree, branch, Git index, and temporary paths.

Do not let another session mutate them during validation or review.

## File-size guardrail

Hand-maintained first-party source files must stay under 900 lines.

Markdown, MDX, generated files, lockfiles, snapshots, baselines, `target/**`, and `third_party/**` are excluded.

Run the file-size guardrail before you finish the item.

If a touched file exceeds the limit, split it by responsibility and ownership boundary.

Do not split a module alphabetically or by line-count chunks.

## Code Rules

- Do not use data-dependent `.unwrap()` or `.expect()` in generated runtime code.
- Use `assert!` only for programmer invariants.
- Workspace lints warn on `unsafe_code`, `print_stdout`, `print_stderr`, and `dbg_macro`.
- `Cargo.lock` is tracked. Treat its diffs as intentional dependency changes.
- Use `insta` for snapshot tests.
- E2E fixtures run in lexical order.
- Snapshot expectations follow declaration order.

## Commands

```bash
# Build the compiler
cargo build --release

# Run a Sifr file
cargo run -q -p sifr -- run <file>.sifr

# Build, check, or emit a Sifr file
cargo run -q -p sifr -- build <file>.sifr
cargo run -q -p sifr -- check <file>.sifr
cargo run -q -p sifr -- emit <file>.sifr

# Unit tests without the slow E2E pass suite
cargo test -p sifr -- --skip test_e2e_pass

# Single test
cargo test -p sifr -- <test_name>

# E2E pass suite
verification/runner/e2e/run_e2e_pass.sh

# Linting
cargo clippy --workspace -- -D warnings
cargo fmt --check
python3 scripts/check_hir_maintainability_guardrails.py

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

- Before a long Cargo gate, inspect free disk space and the current worktree's target size.
- If its private target exceeds **20 GiB** and no process uses it, run `cargo clean` in that worktree.
- Do not clean a shared target or a target from another worktree.
- Do not use the first cold-cache run as host-sensitive performance evidence.
- If safe cleanup does not provide enough space, record the resource blocker and stop.

## Records

- Update the active issue after each merged item.
- Update `internal_docs/architecture.md` only when architecture changes.
- Update `plans/roadmap.md` only when roadmap status changes.

## Safety

- Do not use destructive git operations unless explicitly requested.
- Do not revert unrelated user changes.
- If unexpected repo modifications appear, stop and ask before proceeding.
