# M10 Wave 2 review pass 21

- Reviewer: Codex CLI, `gpt-5.6-sol`, high reasoning, fast service tier
- Scope: complete committed `main...HEAD` diff after pass 20 remediation
- Verdict: **CHANGES REQUIRED**

## Findings

1. High: re-export identity remained path-dependent. A class exported through
   one facade and its factory through another retained different identities;
   split-path root/leaf ancestry failed for the same reason.
2. High: user-module generic method requirements existed in HIR but
   `collect_module_exports` did not copy `generic_functions` or
   `type_param_bounds`, so downstream lowering accepted specializations whose
   emitted Rust methods were unavailable.
3. High: class-pattern inference and final lowering selected the first matching
   member of `Box[int] | Box[str]`, inferred only `int`, and native codegen
   emitted duplicate unspecialized `Box` union variants.
4. Medium: permanent evidence covered a common facade and one specialization at
   a time, so the closure claims overstated split-path, exported-bound, and
   same-class specialization-union coverage.

## Required remediation

- Preserve stable canonical class identity independently of the import path and
  emitted local spelling.
- Export and re-export generic callable and method-requirement metadata with the
  same identity mapping as types.
- Combine all matching generic specializations during pattern typing and emit a
  valid representation, or reject the unsupported union during checking.
- Add split-path factory/ancestry, cross-module generic-bound, direct/nested
  specialization-union, and native backend regressions.

The reviewer confirmed full frontend, lowering, codegen, and type-system suites;
buffer runtime `25/25`; the existing pass-20 project tests; formatting, diff,
HIR/driver maintainability, and file-size guardrails. No new buffer lifecycle,
affine release, or overlap-admission defect was found.

## Remediation disposition

- Added stable declaration identity to class types while retaining the local
  spelling used by Rust emission. Export canonicalization and import
  localization preserve that identity and canonical ancestry through unrelated
  facade paths.
- Exported and re-exported generic function/class metadata and exact per-method
  bounds; specialization retains declaration identity during type-variable
  binding.
- Rejected unions containing multiple concrete specializations of one generic
  class during annotation checking, before pattern inference or Rust union
  generation can discard a member or emit duplicate variants.
- Converted project evidence to split type/factory and root/leaf facades, added
  a multi-hop generic-bound negative, direct/nested union-pattern negatives,
  stable-identity unit coverage, and a native split-path build.
- Full affected suites pass: type system `103/103`, lowering `756` with one
  ignored, frontend `47/47`, codegen `825/825`, and driver `347` with `22`
  ignored. Clippy, formatting, maintainability, and source-size guardrails pass.
- The clean-build authoritative create-PR gate passes every blocking lane:
  Python interop `11/11` in `106.89s`, runtime platform `28` variants with one
  capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` after rebuilding all `42` groups. The `911.81s` wall time
  produced only the expected non-blocking warm-target advisory after
  `cargo clean`.
- Reviewer satisfaction is tracked in pass 22.
