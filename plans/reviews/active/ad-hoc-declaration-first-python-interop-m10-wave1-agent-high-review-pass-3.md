# M10 Wave 1 review — agent 5.6 Sol high, pass 3

- Date: 2026-07-15
- Pull request: [#2987](https://github.com/sifr-lang/sifr/pull/2987)
- Reviewer: agent
- Reasoning/service tier: high / fast
- Scope: complete M10 Wave 1 diff against `main`, including pass-1 and pass-2 remediation
- Verdict: **blocked**

The reviewer confirmed that the eleven findings from the first two passes were
represented by remediation code and tests, and that the public decorator and
capability ledger remained gated. It identified two remaining merge blockers:

1. `PyBUF_MAX_NDIM` has a `c_int` FFI type on CPython 3.8–3.10 and a `usize`
   type on CPython 3.11+, so comparing it directly with a `usize` dimension
   count did not compile across the supported CPython range.
2. Typed-access failures constructed `PythonError` while holding the
   per-buffer lifecycle mutex. Traceback formatting can execute Python and
   release the GIL, allowing a concurrent release to acquire the GIL and then
   wait on the mutex while the accessor waits to reacquire the GIL.

Required remediation was to normalize the cross-version dimension constant
through a checked conversion with compile-shape coverage for both FFI types,
and to return Python-free internal errors from the locked access section,
release the mutex, and only then perform Python error conversion. A
deterministic concurrent-release regression was also required.
