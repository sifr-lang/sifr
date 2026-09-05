# Ad Hoc Phase: Packaged Candidate Generated Rust

Status: complete (2026-08-10). Implementation
[#3102](https://github.com/sifr-lang/sifr/pull/3102) merged as
`3426c7c53025c867c565cb6981cad3d1695b045b`. The final reviewed and gated
candidate was `eb0599f4f8ed06c1fae5e0055116849e3f4616e3`, based on
`8d1c71150e8e2a5718f0e85bb3d4166de3dc0521`.

## Resolution

Generated-Rust-only production now defers direct Cargo probes only for
integrity-checked packaged sysroot declarations. It preserves the complete
probe plan and keeps check, build, user-package, untrusted, and
integrity-mismatched paths fail-closed. The generated-Rust LSP request checks
cancellation before compiler work and retains deterministic shutdown.

The release qualification covers cold and warm installed artifacts on all four
supported targets. It exercises packaged `sifr emit`,
`sifr.server.showGeneratedRust`, request cancellation, bounded shutdown, editor
contracts, artifact assembly, and qualification-index custody. Stable editor
documentation and release notes now advertise the qualified action again.

## Closure Evidence

- The governed four-target release qualification
  [run 31368449311](https://github.com/sifr-lang/sifr/actions/runs/31368449311)
  passed every release target and the editor, assembly, and index contracts.
- The exact-candidate create-PR gate exited 0.
  - Python interop passed 19/19 in 330.329 seconds. Its read-only check took
    72.952 seconds and produced zero mutations.
  - Rust interop passed 10/10 in 6.742 seconds.
  - Developer tooling passed 18/18.
  - Generated-code quality passed 5/5 in 8.567 seconds.
  - Runtime platform passed 28 variants with one declared skip.
  - The E2E subset passed 140/140.
- The authoritative exact-candidate merge gate exited 0 in 6,332.84 seconds.
  - Python interop passed 25/25. Its read-only check took 61.828 seconds and
    produced zero mutations.
  - Rust interop passed 10/10.
  - Representative performance passed 10/10.
  - Developer tooling passed 32/32.
  - Generated-code quality passed 7/7.
  - Dedicated generated builds passed 68/68.
  - The full E2E corpus passed 694/694.
  - Hardening passed 268 variants with zero failures.
- Focused probe-policy, preserved-plan, cache, LSP cancellation, transfer,
  editor, release-workflow, format, HIR maintainability, and file-size checks
  passed. The implementation changed 23 paths with 258 insertions and 74
  deletions. Touched first-party source files remained below 900 lines.
- Milestone reviews and the final exact-head agent review returned
  `SATISFIED` with no blocking findings. The final review is recorded on
  [PR #3102](https://github.com/sifr-lang/sifr/pull/3102#issuecomment-5238480275).

## Remaining Work

No in-scope remediation, documentation exclusion, timeout waiver, or deferred
blocker remains.

## Problem

The exact packaged Sifr `0.1.0` candidate starts `sifr lsp --stdio`, completes
initialization, publishes diagnostics, and serves formatting requests, but
the cold first-run generated-Rust qualification did not complete through either
`sifr emit` or the `sifr.server.showGeneratedRust` request. The LSP request
exceeded the deterministic 90-second protocol timeout; the capability demo also
required bounded termination of the CLI command after the same interval.

Phase 40 must not advertise generated Rust for the packaged candidate until
both paths pass.

## Evidence

The capability demo built a release-mode `aarch64-apple-darwin` candidate from
source commit `67a40febc`, installed its bundled sysroot in isolation, packaged
`sifr.sifr-vscode` `0.2.0`, and launched that exact binary.

- LSP initialization returned the expected capability object.
- Opening a valid Sifr document published diagnostics normally.
- A fresh-session `workspace/executeCommand` request for
  `sifr.server.showGeneratedRust` then timed out after 90 seconds.
- The server did not terminate after the client closed stdin and required
  bounded process cleanup.
- Repeating the request after the broad protocol query sequence produced the
  same timeout, so the failure is not caused by earlier requests.
- After successful packaged-candidate `check`, format, format-check, and lint
  commands, `sifr emit` also failed to return within 90 seconds and required
  bounded cleanup.
- A reviewer observed the same shape with an in-tree debug binary: its first
  cold invocation exceeded 90 seconds, while identical warm repeats completed
  in roughly six seconds. This makes cold startup, cache population, or
  first-run work the leading hypothesis; the evidence does not establish a
  deadlock.

## Scope

- Reproduce cold and warm behavior against an installed release artifact on
  every supported host target.
- Isolate shared cold-start, cache-population, or unbounded first-run work in
  generated-Rust production.
- Preserve request cancellation and deterministic shutdown behavior.
- Add a packaged-candidate regression test that executes the preview request.
- Restore the editor action to stable documentation only after the exact
  packaged candidate passes.

## Definition of Done

- [x] `sifr emit` and generated-Rust preview return within their governed timeouts
  for an installed release artifact on all supported targets.
- [x] Timeout and cancellation tests prove the server exits without forced cleanup.
- [x] The full LSP protocol suite and packaged-candidate editor qualification pass.
- [x] Stable documentation and release notes advertise the editor action.
