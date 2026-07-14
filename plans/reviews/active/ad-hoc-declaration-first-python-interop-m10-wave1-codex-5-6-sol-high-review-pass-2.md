# M10 Wave 1 review — Codex 5.6 Sol high, pass 2

- Date: 2026-07-15
- Pull request: [#2987](https://github.com/sifr-lang/sifr/pull/2987)
- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning/service tier: high / fast
- Scope: complete M10 Wave 1 diff against `main`, including pass-1 remediation
- Verdict: **blocked**

The reviewer confirmed that all six pass-1 remediations were present and that
the public decorator and capability ledger remained gated. It identified five
remaining actionable issues:

1. `PyBuffer_GetPointer` used pointer types that compile on CPython 3.11+ but
   not against PyO3's CPython 3.8–3.10 FFI signature.
2. A malformed exporter could report success with `Py_buffer.obj == NULL`,
   bypassing exporter retention and release before entering safe typed access.
3. Shape-product validation could overflow before observing a later zero
   dimension in a valid empty buffer.
4. Metadata validation accepted a non-null suboffset vector whose entries were
   all negative, which the buffer protocol requires exporters to represent as
   a null vector.
5. Unsafe logical access needed explicit negative-stride and indirect-pointer
   regression coverage.

Required remediation was to use cross-version-compatible mutable FFI pointers,
reject null ownership before storing a successful view, short-circuit empty
shape products, enforce the suboffset representation invariant, and add real
negative-stride and indirect memoryview fixtures covering reads, writes,
bounds, logical ordering, and release.
