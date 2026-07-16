# M10 Milestone Codex Review Pass 4

- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Scope: complete M10 implementation plus all prior milestone-review
  remediations, range `e4fdc942ed..c41fcfe08`
- Review tree: clean detached worktree at committed HEAD
- Verdict: **CHANGES REQUESTED**

## Finding

1. **High — duplicate fields bypassed the exact `PythonError` contract.** The
   shared structural predicate required all five canonical names and rejected
   unknown names, but did not constrain field count or uniqueness. A local
   `PythonError` with a duplicate `message: str` field therefore passed checking
   and reached the generated-Rust duplicate-field assertion as
   `SIFR-INTERNAL-0001`.

## Reviewer validation

- The prior writable-`Self`, structural error-channel, tracking, and clean
  checkout findings were confirmed fixed.
- All `19,730` checksum-listed files across `442` vendored manifests existed,
  matched their SHA-256 digests, were tracked, and had no extra unchecksummed
  payloads.
- Python interop runner adversarial self-tests, the `900`-line file-size
  guardrail, and HIR maintainability guardrails passed.
- The authoritative clean-checkout create-PR gate passed before review: Python
  interop `12/12`, runtime platform `28` variants with one capability-gated
  skip, and E2E `131/131` with signature `7c39b8c1dd4fec7c`.

## Required remediation

- Require exactly five unique canonical string fields in the shared
  `PythonError` predicate.
- Add duplicate-field regressions that prove checking and compilation return
  the same structured diagnostic without reaching code generation.
