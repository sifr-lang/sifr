# M10 Wave 3 Codex Review Pass 2

- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Scope: complete committed `main...c8ec73113` Wave 3 diff for PR #2989
- Verdict: changes requested

## Findings

1. **Medium — release evidence was incompatible with CPython 3.11.** The
   compiled bridge and runtime exporter used Python-level PEP 688 hooks, which
   were introduced in Python 3.12 despite the verification project's Python
   3.11 minimum.
2. **Medium — fabricated ownership remained admissible.** Matrix owners only
   had to resolve to any existing file, so replacing all owners with the README
   still passed validation.
3. **Low — compiled-fixture identity claims exceeded their assertions.** The
   exact pointer comparison existed in runtime tests, not in every compiled
   bridge and aggregate fixture.
4. **Low — the phase status contradicted itself.** It described Wave 3 as both
   implemented and not implemented.

## Reviewer validation

- Runtime buffer tests passed `30/30`.
- Focused lowering passed `34/34`; focused code generation passed `10/10`.
- Runner self-test, five compiled examples, and the M10 demo passed.
- Selective bridge copying and least-authority manifests were verified.
- Formatting, HIR maintainability, Python syntax, shell syntax, and file-size
  guardrails passed.

## Remediation

- Replaced PEP 688 test exporters with a PyO3 C-level buffer-protocol exporter
  compatible with CPython 3.11 while retaining exact acquisition, release, and
  pointer evidence.
- Replaced compiled PEP 688 bridges with retained builtin `bytearray`
  producers; shared mutation and post-release resizability now prove explicit
  and aggregate automatic cleanup on Python 3.11.
- Locked every matrix row's exact evidence description and exact owner/symbol
  set, with adversarial existing-file and fabricated-description mutations.
- Narrowed compiled evidence claims and repaired the phase status.

The complete remediated diff requires a fresh whole-diff review pass.
