# Agent instructions

Sifr compiles Python syntax to native binaries through Rust. See
[architecture](internal_docs/architecture.md),
[verification commands and contracts](verification/README.md), and the
[active roadmap](plans/roadmap.md).

## Working rules

- Work on one bounded item at a time and fix its root cause. Follow the
  [phase closure loop](.cursor/skills/phase-closure-loop/SKILL.md) for phase work.
  Follow the user's authorized scope and sequencing when it spans multiple items.
- Own your branch, index, worktree, temporary paths, and build artifacts. Preserve
  unrelated changes; do not clean shared targets or another session's inputs.
- Do not let another session mutate owned paths during validation or review.
  If unexpected repository changes appear, stop and ask before proceeding.
- Record out-of-scope failures with their owner. Preserve failed/incomplete
  evidence; missing execution or a required skip never counts as a pass.
- Do not absorb unrelated failures or externally owned dependencies. If an
  external failure blocks the item, record it and stop.
- Do not add compatibility or fallback paths unless requested. Keep `check`
  separate from codegen/runtime and use the canonical frontend authority.
- Prevent user-triggered compiler/runtime panics. Generated runtime code must not
  use data-dependent `.unwrap()`/`.expect()`; `assert!` is for programmer invariants.
- Keep hand-maintained first-party source below 900 lines; split by responsibility.
  Markdown, generated files, lockfiles, snapshots, targets, and third-party files
  are excluded. Treat tracked `Cargo.lock` changes as intentional dependency work.
- Use `insta` for snapshots; preserve declaration order in snapshot expectations
  and lexical E2E fixture ordering. Respect the workspace lint policy.
- Do not perform destructive Git operations without explicit authorization.

## Minimum validation

Run checks for the work you changed, then the applicable acceptance gate.
Commands below run from the repository root; named phase/area contracts may add
affected checks. Do not run the whole table for every edit.

| Work | Run |
|---|---|
| Planning, prose, or review records only | `git diff --check`; verify changed local links, scope, and status. No compiler build or broad gate. |
| Rust/compiler code | Focused affected crate/test, e.g. `cargo test -p <crate> <test>`; `cargo fmt --check`; `python3 scripts/check_file_size_guardrails.py`; affected area/regression checks. |
| Verification runner or policy | Affected self-tests and area/workflow contracts; file-size guardrail. If profile selection changes, run `uv run --project verification --locked python -m sifr_verify profiles check` and compare emitted plans. |
| Implementation PR candidate | `scripts/run_all_tests.sh --profile create-pr`. |
| Final implementation merge candidate | `scripts/run_all_tests.sh --profile merge` once; correct failures and rerun affected checks after relevant changes. |
| Final implementation merge candidate on a shared Linux VM | `scripts/run_all_tests.sh --profile cloud --require-performance --compact-resources` once, with `SIFR_CLOUD_PERFORMANCE_RECEIPT` pointing to the complete candidate-bound paired receipt; same merge correctness selection. |
| Actual release or explicit live integration | Required `release`/artifact qualification or `python-interop-live` contract. |

Use the exact tool versions in `verification/pyproject.toml` and
`rust-toolchain.toml`. CI mirrors local validation; do not substitute waiting for
CI for required local checks. Reuse evidence only when the declared input-bound
protocol proves it still applies; keep the observed candidate identity.

An explicitly applicable canonical phase exception overrides the PR/merge rows.
The approved [Phase DX policy](plans/issues/archive/ad-hoc-compiler-dx-and-toolchain-reuse.md#global-execution-rules)
retains named intermediate checks/review and a final phase merge gate. It does
not waive other phases' requirements. Record-only updates need docs checks only.

Before long builds, check free disk, target size, effective CPU/memory, and the
planned allocation/reserve. Reclaim only inactive, obsolete, session-owned
artifacts when needed; preserve leases, consumers, reports, and failure evidence.
Do not use a cold build as host-sensitive performance evidence. Record an actual
capacity blocker when safe cleanup cannot provide the required reserve.

Update the active plan as work is delivered. Update architecture or roadmap
records only when their architecture or execution status changes.
