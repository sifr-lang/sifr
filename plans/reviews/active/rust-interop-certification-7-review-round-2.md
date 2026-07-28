# Rust Interop Certification 7 Review — Round 2

- Reviewed commit: `951565850`
- Base: `origin/main`
- Reviewer: Claude Opus 5, medium effort
- Verdict: findings; not satisfied

## Round-1 closure

The reviewer confirmed all six round-1 findings closed. It independently
passed the 10-variant Rust-interop area, 139/6/5/20/33 checker self-tests, all
three mandatory generated-build tests, 61 focused driver tests, formatting,
Clippy, file-size and maintainability guardrails, exact inventory counts, safe
Rust policy, and preservation of the unrelated worktree paths.

## New findings

### 1. Medium — unsupported return reason can be masked

Running exact view identity before recognizing a Result contract's propagated
`unsupported_reason` can emit a misleading zero-copy missing-rendering
diagnostic and return before the accurate bridge-contract diagnostic. Detect
propagated unsupported contracts before view identity and add an end-to-end
diagnostic regression.

### 2. Low — probe-source test matches helper definitions

The four-way source test looked for the helper name, so it could pass if the
bound helper definition remained but the actual assertion call disappeared.
Match the exact `::<__SifrView>();` invocation for both Send and Sync.

### 3. Low — scenario checker has no size headroom

`_scenario_checks.py` was 899 lines. Move additional per-fixture inventory into
its responsibility-specific modules so ordinary maintenance cannot
immediately trip the 900-line gate.
