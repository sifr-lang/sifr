# M10 Milestone agent Review Pass 1

- Reviewer: agent
- Reasoning: high
- Service tier: fast
- Scope: complete merged M10 implementation, PRs #2987 through #2989
- Verdict: **CHANGES REQUESTED**

## Findings

1. **High — writable `Self` buffers did not exclusively borrow or freeze their
   opaque exporter.** The declaration used immutable borrowed `self`, generated
   acquisition used `&self`, and runtime admission tracked only buffer views.
   Owner methods, including consuming close, therefore remained accepted while
   the writable view was live.
2. **High — the buffer error channel accepted a shadow class named
   `PythonError`.** Lowering validated the name only, buffer method typing fell
   back to `Any`, and code generation required a richer field shape. A source
   could consequently pass `check` and fail `build`.

## Reviewer validation

- All 31 Python buffer runtime tests passed.
- All 34 buffer lowering contract tests passed.
- All 10 buffer code-generation tests passed.
- The Python interop evidence self-test passed.
- Documentation, evidence ownership, and activation profiles were inspected.
- Adversarial writable-owner and shadow-error sources reproduced both findings.

## Required remediation

- Either tie a writable receiver view to an exclusive owner freeze or reject
  writable `Self`; add permanent owner-use/move/close negative coverage.
- Use one canonical Python error predicate across buffer lowering, method
  typing, and code generation; remove the `Any` fallback and reject incompatible
  shadow classes before code generation.
