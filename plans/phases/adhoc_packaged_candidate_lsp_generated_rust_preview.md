# Ad Hoc Phase: Packaged Candidate LSP Generated-Rust Preview

Status: deferred follow-up; not a prerequisite for Phase 40.

## Problem

The exact packaged Sifr `0.1.0` candidate starts `sifr lsp --stdio`, completes
initialization, publishes diagnostics, and serves formatting requests, but the
`sifr.server.showGeneratedRust` request does not return within the deterministic
90-second protocol timeout.

Phase 40 must not advertise that editor action until the packaged-candidate path
passes. The compiler's separate `sifr emit` command remains qualified and is
not affected.

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

## Scope

- Reproduce the hang against an installed release artifact on every supported
  host target.
- Isolate the deadlock or unbounded operation in generated-Rust preview.
- Preserve request cancellation and deterministic shutdown behavior.
- Add a packaged-candidate regression test that executes the preview request.
- Restore the editor action to stable documentation only after the exact
  packaged candidate passes.

## Definition of Done

- Generated-Rust preview returns within the governed LSP timeout for an
  installed release artifact on all supported targets.
- Timeout and cancellation tests prove the server exits without forced cleanup.
- The full LSP protocol suite and packaged-candidate editor qualification pass.
- Stable documentation and release notes may then advertise the editor action.
