# DXF.3 housekeeping review follow-ups

Status: nonblocking follow-up work, separate from merged DXF.3.

Source: [scoped Opus review of #3879](https://github.com/sifr-lang/sifr/pull/3879#issuecomment-5749706095),
candidate `0bc95340dfe6a5e4e32e0824a7b62455dae24df7`.
Verdict: **SATISFIED**, no blocking findings. Complete read-only review evidence
is preserved outside the Git tree in the candidate's `dxf-evidence/opus-review.md`.

| ID | Classification / owner | Finding and disposition |
| --- | --- | --- |
| DXF3-F1 | Suggestion / affected lint reconciliation | Opus suspected `ReadDir.into_iter()` in `housekeeping.rs` may trigger `clippy::useless_conversion` under `-D warnings`. Clippy was not among DXF.3's mandated checks; this was a suspicion, not a measured failure. Closed by the bounded affected-lint reconciliation below: the selected Rust 1.98.1 targets pass and this expression produces no warning; separate measured DXF warnings were corrected. |
| DXF3-F2 | Suggestion / future storage lifecycle work | Permanent generation locks are established before staging so read paths do not create lock inodes. A failure after lock creation but before publication can leave a zero-byte generation lock without a generation. Live cleanup intentionally preserves generation lock inodes; orphan context cleanup can reclaim them under exclusive namespace and context leases. Any additional live lock reclamation requires a separate bounded design that preserves concurrent readers. |
| DXF3-F3 | Documented lifecycle limitation / future owner-reset policy | A deleted and recreated directory at the same path has a different owner identity. Its old namespace remains protected and cannot be reused or pruned as an orphan while the replacement exists. This is deliberate replacement-path preservation, documented in the architecture, not permission to invent automatic owner takeover. A separate explicit reset/recovery policy would need scoped authorization. |
| DXF3-F4 | Documented filesystem limitation / qualification | Directory creation time is required for the owner identity. Filesystems/kernel combinations without usable btime leave optional project persistence unavailable. Current evidence is the remote qualified Linux filesystem only; overlayfs/NFS/older-kernel behavior is not claimed as tested. Retain this limitation when making supported-host/storage claims. |
| DXF3-F5 | Suggestion / CLI contract owner | A busy context produces zero examined/eligible/deleted counters because cleanup cannot inspect it under the writer lease. This is truthful but indistinguishable from an empty scope. An explicit contention indicator is an optional future refinement for the CLI owner, not a new DXF.6 requirement. |

No finding reopens DXF.3, adds a current gate or authorizes unrelated work.
The canonical sequential plan remains authoritative.

## Affected Clippy reconciliation — 2026-09-21

Scope: the user-authorized follow-up closes the actual affected lint gap after
DXF.1–6. Old Phase DX Clippy evidence does not qualify the new source. Actual
changed Rust paths select `sifr_syntax`, `sifr_frontend`, `sifr_driver`,
`sifr_lsp`, and `sifr`. Source and analysis are unchanged dependency owners,
not extra top-level selections. Default features select all targets of the four core packages and the CLI
production binary plus its explicit test profile. This covers the changed
production and inline-test configurations without a workspace gate.

The current CI advisory log for run 35533124294 / job 106137284240 and the
first remote selected run both establish two driver production failures:
`ProjectPruneReport::add` unnecessarily consumed a report, and the DXF.2
borrowed-record loop passed `&&Record` to interface restore. The fix borrows
the report and removes the redundant reference; counters, cache proof and
ownership/lease behavior are unchanged. DXF3-F1's `ReadDir.into_iter()` is not
a measured warning on Rust 1.98.1 and is left unchanged.

Selecting test targets additionally exposed pre-existing test-only style lints.
The bounded acceptance repair preserves assertions and fixtures while moving
helper/test declarations, making formatting/variant/reference choices explicit,
and using the first vector entry directly. Metadata evidence still writes to
stderr with explicit I/O error handling. Hex output uses `write!` into the same
string. The byte-stream test counts NUL bytes with `memchr_iter`; its existing
workspace-pinned `memchr` becomes a driver dev dependency, with exactly one
intentional lockfile dependency-edge addition and no version change. No lint
suppression, relaxed warning policy, fallback, runtime redesign or extra feature
configuration is introduced.

The external evidence root is
`/home/yaser5/projects/sifr/affected-clippy-evidence/`. It retains the selector
inventory, CI diagnostic log, every failed/intermediate command and the final
candidate-bound results. A production-only intermediate pass is not substituted
for affected test-target completion. The broader all-target attempt also
found 46 pre-existing errors in the unchanged standalone formatter integration
target, recorded with its [owning follow-up](ad-hoc-formatter-integration-test-lints.md). No new performance, Windows, release or whole-workspace
lint claim follows from this bounded work. CI admission/provisioning is separately
closed; the recorded cold-host performance, interrupted determinism and Windows
portability failures remain with their owning issues.

CLI inline acceptance also replaces narrowing/sign-changing boundary casts with
the exact three usize lengths, and makes existing formatter test pointer, byte
array and Result assertions explicit. No boundary or assertion is removed.

## Merged lint record — 2026-09-21 (Europe/Stockholm)

- [PR #3895](https://github.com/sifr-lang/sifr/pull/3895) merged at
  2026-09-20T23:25:20Z as `dbdfbd925ba1ddc409ce0b4f2cafd09185b1f5d5`.
  Exact reviewed/validated candidate: `f6ffa577a7a5b379a90a8a3ea38e70fbabf2bd83`;
  base: `d144d5ae2d7501712c55480e6fa35af4bccb4102`.
- [Scoped Opus verdict](https://github.com/sifr-lang/sifr/pull/3895#issuecomment-5753489680):
  SATISFIED, no blockers. External candidate-keyed `opus-review.md` SHA-256:
  `0de17d74cb1f6150c1a69643ff6ff618352beefd0002f9ea18db7e15d2e10faf`.
  The reviewed `evidence-index.json` SHA-256:
  `83f601c6a145fe5368f20eeaf871509751edbcc5e9e042adaff7b6a46df3b5f9`.
  Both live under the evidence root above / candidate SHA.
- Strict Clippy passes these exact default-feature selections:
  `cargo clippy --locked -p sifr_syntax -p sifr_frontend -p sifr_driver -p sifr_lsp --all-targets -- -D warnings`;
  `cargo clippy --locked -p sifr --bin sifr --profile test -v -- -D warnings`;
  `cargo clippy --locked -p sifr --bin sifr -- -D warnings`.
  The CLI test-profile compiler command explicitly contains `--test`.
- Exact list guards each found one test, and each test passed:
  driver `process_execution::tests::dx3_program_streams_and_cancellation_state_are_scoped`;
  CLI `trace_artifacts::tests::final_timing_width_at_byte_boundary_truncates_truthfully`.
  Each uses `cargo test --locked -p PACKAGE --lib/--bin sifr FILTER -- --exact --nocapture`
  with the corresponding concrete target and filter recorded in the index.
  Formatting, 4,110-file size guard, HIR maintainability, diff and local links pass.
  Rust 1.98.1, existing private target, two jobs; no cleanup or performance claim.

Review follow-up dispositions: cached core replay is accepted under identical
input hashes; its strongest compiling evidence is the preserved broader run
that failed only on the excluded formatter integration target. That broader
run is still a failure, never relabeled as a full pass. Explicit stderr I/O
preserves permanent test evidence without a lint attribute; it changes the
emission construct rather than prohibiting all stderr output. The hex helper's
byte equivalence is established by inspection, not a claimed executed test.
Dates use Europe/Stockholm; the UTC candidate/review/merge receipts fall on
September 20, already September 21 locally. No additional implementation arose
from review, so this record-only closure needs documentation checks only.

The formatter integration target remains with its linked owner; other CI
performance/determinism/Windows limitations remain unchanged. DXF3-F1 and the
affected lint reconciliation are closed, with no selected-target blocker.
