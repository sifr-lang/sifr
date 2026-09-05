# Ad Hoc Large LSP Long-Session Verification

Status: ready for PR

This is a post-phase verification hardening task following the completed
TypeScript-Go compiler architecture transfer. It does not reopen the transfer
phase; it adds large-codebase and long-session editor coverage for the LSP
architecture delivered there.

## Goals

- Publish a public synthetic Sifr corpus repository under `sifr-lang`.
- Pin that repository as a submodule under `verification/`.
- Add a rerunnable long-session LSP verifier that exercises a large project
  graph over many edits and requests.
- Record latency and sampled RSS evidence, including peak memory and RSS growth
  slope, so slow leaks are visible.
- Keep a bounded smoke mode in local validation and a fuller mode available for
  explicit long-session qualification.

## Acceptance Criteria

- `verification/sifr-large-lsp-verification` is a git submodule pointing at
  `sifr-lang/sifr-large-lsp-verification`.
- The subrepo contains a deterministic committed corpus plus a generator drift
  check.
- `verification/tooling/lsp_large_session.py --mode smoke` runs in the quick
  validation lane.
- `verification/tooling/lsp_large_session.py --mode full` is documented and
  writes JSON evidence under `target/lsp_large_session/`.
- The verifier samples LSP RSS during the session, not just at process exit.
- agent review passes are recorded under `reviews/` and the final pass is
  satisfied.

## Progress

- Created `sifr-lang/sifr-large-lsp-verification` as the public synthetic
  corpus repository.
- Added a deterministic corpus generator and drift checker in the subrepo.
- Added the submodule and initial long-session LSP verifier in this branch.
- Found and fixed an LSP analysis availability gap for secondary project files:
  if a project host cannot map an opened file into the project graph, the LSP
  now keeps the project and serves that open file through standalone document
  analysis instead of leaving it unavailable.
- Found and fixed avoidable project refreshes on same-project opens/edits:
  open/change/save now use incremental analysis paths and only refresh project
  membership when the incremental path cannot attach.
- Current local evidence:
  - `python3 verification/tooling/lsp_large_session.py --mode smoke --require-submodule`
    passed with 42 operations, p95 5.645 ms, peak RSS 2.1 MiB.
  - `python3 verification/tooling/lsp_large_session.py --mode full --require-submodule`
    passed with 1702 operations, including 480 change notifications and 30
    `textDocument/diagnostic` requests, p95 6.363 ms, peak RSS 19.0 MiB, RSS
    slope 29.34393 MiB/min.
  - `cargo test -q -p sifr_lsp unmapped_project_file_fallback_survives`
    passed.
  - `cargo test -q -p sifr_analysis` passed.

## Notes

- The large-session verifier runs with push diagnostics disabled
  (`diagnosticsMode=off`) so the long edit/query session measures LSP
  responsiveness over a very large codebase without blocking each notification
  on synchronous diagnostic publication. Full mode still issues periodic
  `textDocument/diagnostic` pull requests over edited files, while existing
  protocol smoke/stress checks continue to cover diagnostic publication
  behavior.
