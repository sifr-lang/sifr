# AGENTS.md

## What is Sifr?

Sifr is a compiled language with Python syntax that compiles to Rust, producing native binaries. It enforces static typing, safe error handling (Result/Option instead of exceptions), and ownership semantics at compile time. Core guarantee: "if it compiles, it works" — no user-triggerable runtime panics.

## Build & test commands

```bash
# Build the compiler
cargo build --release

# Compile and run a .sifr file
cargo run -q -p sifr -- run demos/<file>.sifr

# Other CLI modes
cargo run -q -p sifr -- build <file>.sifr   # Build native binary
cargo run -q -p sifr -- check <file>.sifr   # Type-check only
cargo run -q -p sifr -- emit <file>.sifr    # Show generated Rust code

# Unit tests only (skips slow e2e pass suite)
cargo test -p sifr -- --skip test_e2e_pass

# Single test
cargo test -p sifr -- <test_name>

# E2E pass suite only
scripts/run_e2e_pass.sh

# Linting
cargo clippy --workspace -- -D warnings
cargo fmt --check
python3 scripts/check_hir_maintainability_guardrails.py
```

## Local validation (authoritative gate — run before PRs)

Before considering any task done, run local validation on your changes:

```bash
scripts/run_all_tests.sh --profile create-pr  # Fast signal — use for PRs
scripts/run_all_tests.sh                      # Merge gate — default
```

CI mirrors these exact scripts — no CI-only behavior. Do not wait on CI; validate locally first.

## File-size guardrail

Hand-maintained first-party source files must stay under **900 lines**. Generated files, lockfiles, snapshots, baselines, `target/**`, and `third_party/**` are excluded.

Run the file-size guardrail before considering work complete. If a touched file exceeds the cap, refactor it by responsibility rather than adding more code to an oversized module.

Use the existing HIR lowering and package-manager module layouts as examples of responsibility-based decomposition: split by compiler concern and ownership boundary, not by alphabetical order or line-count chunks.

## Compiler pipeline

Which crate to touch for a given task:

```
Source (.sifr)
  → sifr_python_parser / sifr_python_ast   (Ruff fork submodule, currently based on Ruff 0.15.12)
  → sifr_hir                                (name resolution, type checking, ownership tracking)
  → sifr_codegen                             (HIR → Rust IR → syn AST → prettyplease output)
  → sifr_driver                              (orchestration, rustc invocation)
  → sifr                                     (CLI binary: build/run/check/emit/test)
```

See `internal_docs/architecture.md` for full architectural detail.

## Key conventions

- **Workspace lints**: Clippy pedantic enabled. `unsafe_code`, `print_stdout`, `print_stderr`, `dbg_macro` are warned.
- **No monolithic files**: All crates should be decomposed into small, focused files — monolithic files are banned. HIR lowering has automated guardrails enforced by `check_hir_maintainability_guardrails.py`.
- **Snapshot testing**: Uses `insta` for e2e and unit test snapshots. E2E fixtures are discovered lexicographically, expectations follow declaration order.
- **No panics in user paths**: No data-dependent `.unwrap()` or `.expect()` in generated runtime code. `assert!` is only for programmer invariants.
- **Cargo lockfile**: `Cargo.lock` is tracked for this compiler workspace. Treat lockfile diffs as intentional dependency graph changes and validate with the local facade before PRs.

## Workspace structure

- `crates/` — Rust workspace (see pipeline above)
- `demos/` — Milestone demo files (*.sifr) showcasing language features
- `scripts/` — Build/test automation
- `verification/` — E2E test infrastructure
- `plans/` — Roadmap, phase plans, issue plans, and review artifacts
- `internal_docs/` — Durable architecture, accepted decisions, and current technical references
- `docs/` — Public/site-facing docs such as Sifr documentation and CLI... etc

## Core expectations

- Solve root causes, not superficial symptoms.
- Do NOT create fallback paths or solutions unless explicitly requested.
- No laziness and no shortcuts, make sure to ideally fix the root cause.
- Keep changes focused on the requested milestone/issue.
- Prefer small, reviewable PRs with clear validation.
- Do not wait on CI; run local validations.

## Required workflow

- Follow `.cursor/skills/project-workflow/SKILL.md`.
- Execute items one by one in a loop:
  1. Plan and define to-do list for all the parts of the item.
  2. Implement and validate locally (demo + tests).
  3. Open PR for that item.
  4. Review and merge.
  5. Move to the next item.
- Keep docs updated with status, checklist state, and merged PR links.

## Planning and tracking files

Update corresponding docs after each item is completed (as applicable):

- Architecture: `internal_docs/architecture.md`
- Roadmap: `plans/roadmap.md`
- Phases: `plans/phases/`
- Issues: `plans/issues/`

## Safety rules

- Do not use destructive git operations unless explicitly requested.
- Do not revert unrelated user changes.
- If unexpected repo modifications appear, stop and ask before proceeding.
