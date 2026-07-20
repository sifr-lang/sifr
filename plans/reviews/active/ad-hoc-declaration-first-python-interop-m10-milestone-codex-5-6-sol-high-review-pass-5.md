# M10 Milestone Codex Review Pass 5

- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Scope: complete M10 implementation plus all prior milestone-review
  remediations, range `e4fdc942ed..4315388ff`
- Review tree: clean detached worktree at committed HEAD
- Verdict: **CHANGES REQUESTED**

## Finding

1. **High — writable producers could alias a borrowed Sifr-owned exporter.** A
   writable declaration accepted a borrowed opaque/Object parameter, generated
   code cloned that handle into the foreign producer call, and runtime admission
   tracked only buffer footprints. A producer returning its argument therefore
   left the original Sifr owner usable while a writable view was live.

## Reviewer validation

- The writable-`Self`, exact and duplicate-safe `PythonError`, tracking, and
  clean-checkout vendor remediations were confirmed fixed.
- The complete buffer lowering `37/37`, codegen `10/10`, type-system, and driver
  check/compile parity regressions passed independently.
- All `19,730` payloads in `442` vendored manifests existed, matched their
  checksums, and had no extra or untracked files.
- Formatting, evidence self-tests, HIR and driver maintainability, and the
  `900`-line file-size guardrail passed.

## Required remediation

- Require ownership transfer for every writable-producer parameter that can
  transitively carry an existing raw or opaque Python identity.
- Add a permanent borrowed-owner compile-fail case and correct the public,
  architecture, and machine-owned evidence contracts.
