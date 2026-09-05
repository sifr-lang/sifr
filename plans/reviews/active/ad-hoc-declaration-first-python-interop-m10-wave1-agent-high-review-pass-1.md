# M10 Wave 1 review — agent 5.6 Sol high, pass 1

- Date: 2026-07-14
- Pull request: [#2987](https://github.com/sifr-lang/sifr/pull/2987)
- Reviewer: agent
- Reasoning/service tier: high / fast
- Scope: complete M10 Wave 1 diff against `main`
- Verdict: **blocked**

The reviewer confirmed that `@python.buffer` remained reserved, the capability
ledger remained gated, Python and `Py_buffer` operations did not run while the
global buffer-store mutex was held, and all changed Rust files remained below
the 900-line limit. It identified six actionable correctness gaps:

1. `access=read` was not retained or enforced by write accessors, and writable
   acquisition did not request `PyBUF_WRITABLE` from the exporter.
2. Typed compatibility inherited PyO3 0.29.0's reversed little-endian predicate,
   which could reject native buffers and accept byte-swapped buffers.
3. An `Arc` snapshot could keep the exported view live and successfully access
   it after `release_buffer` returned.
4. PyO3's typed acquisition rejected valid zero-dimensional buffers with null
   shape and stride vectors.
5. Metadata validation used floor-divided item counts instead of enforcing
   `product(shape) * itemsize == len` exactly.
6. `layout=any` admitted non-contiguous views even though the accessors only
   supported contiguous slices.

Required remediation was to implement correct C-API acquisition flags, an
independent primitive PEP 3118 format/endian validator, scalar-aware raw
metadata validation, per-resource release/admission linearization, exact byte
length checking, and stride/suboffset-aware bounded access, with focused
regression coverage for every issue.
