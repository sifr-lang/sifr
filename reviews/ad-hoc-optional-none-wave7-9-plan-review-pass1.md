## Review Pass 1: Optional/None Wave 7-9 Plan

Reviewed artifact:
- `issues/ad-hoc-optional-none-and-narrowing-wave7-9-root-cause-plan-2026-03-29.md` (initial draft)

Verdict:
- **Not ready** (blocking corrections required before implementation)

Blocking findings:

1. Wave ownership was misaligned with real implementation loci.
2. Draft wave-8 shape risked operator-level Optional stripping (would violate explicit Option semantics).
3. Draft wave-9 shape targeted non-owning files for call-boundary/container refinement.
4. Wave-7 needed an explicit guardrail to keep truthiness-derived guards sequence-specific only.

Required corrections:

- Move assignment/flow lane to:
  - `crates/sifr_hir/src/lower/assignment_widening.rs`
  - `crates/sifr_hir/src/lower/statements.rs`
  - `crates/sifr_hir/src/lower/tuple_unpack.rs`
- Move call-boundary lane to:
  - `crates/sifr_hir/src/lower/method_call_args.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
- Keep container/key refinement lane anchored in:
  - `crates/sifr_hir/src/lower/sequence_guard_detection.rs`
  - `crates/sifr_hir/src/lower/guarded_index.rs`
- Add explicit no-hidden-unwrap and no-global-truthiness guardrails.

Status after pass-1:
- Plan updated per findings.
- A second reviewer pass is required before any implementation wave starts.

