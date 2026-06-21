# Rust Interop Verification Matrix Hardening

## Status

Active follow-up created by Phase 39 final closeout review.

## Objective

Tighten Rust interop verification metadata so future promotion work cannot
accidentally treat documentation-only evidence, tier labels, or stale-draft
context as stronger evidence than the runner actually enforces.

## Scope

- Cross-validate `tier` against `execution_kind` in
  `verification/areas/rust_interop/checks/check_fixture_matrix.py`.
- Reject compiler-diagnostic rows that list runtime crate requirements unless
  the row explicitly documents why those crates are only diagnostic fixtures.
- Require compatibility rows that claim `supported` or
  `supported-through-bridge` to point at executable evidence owned by a local
  validation lane, not only README prose.
- Replace the stale-draft rejection-context heuristic with a structured marker
  or stricter parser so accepted examples cannot pass by incidental nearby
  wording.

## Acceptance Criteria

- `check_fixture_matrix.py` rejects tier/execution-kind mismatches for all
  existing and future Rust interop fixture rows.
- `check_compatibility_matrix.py` rejects support claims whose cited evidence is
  not tied to an executable validation command or fixture runner.
- `check_stale_drafts.py` differentiates rejected examples from accepted prose
  without broad lexical prefix matches.
- The Rust interop README and compatibility docs describe the strengthened
  evidence rules after the validators land.
