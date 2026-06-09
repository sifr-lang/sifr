PASS

Verification notes:
- Column math: `from sifr.ipc import ` is 21 chars, so the imported member begins at col 22 for all six new fixtures (`Queue`, `Pipe`, `Pool`, `fork`, `forkserver`, `shared_memory`) — matches the existing `ProcessPoolExecutor`/`Process` fixtures.
- Design row at `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` now lists all eight focused fixtures and retains the "Missing-member diagnostics keep CPython-shaped process-pool and multiprocessing names out of the native IPC module." boundary statement; no public API additions implied.
- The follow-up paragraph in the same doc swaps the speculative "M6 may add focused fixtures…" sentence for a factual one that points at the new `sifr.ipc` fixtures — consistent with the row.
- Ledger entries record pending PR (no PR URL yet, no merge ledger), targeted `cargo run -- check` per fixture, full `test_e2e_fail` count (461), `cargo fmt --check`, `git diff --check`, file-size guardrail, and per-file line counts. M6 status line remains `pending`.
- Diff is scoped to the two docs plus the six new fixture files (the seventh `ipc_multiprocessing_process_unsupported.sifr` shown in the listing is the pre-existing Process fixture, not introduced here).
