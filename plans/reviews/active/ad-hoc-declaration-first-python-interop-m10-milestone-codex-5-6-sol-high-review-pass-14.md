# M10 milestone review pass 14

Reviewer: Codex CLI `gpt-5.6-sol`, high reasoning, fast service tier

Scope: complete `origin/main...cc13a8a49255c5534f451705f12edbfc280a19dd`
M10 milestone implementation after pass-13 remediation

Verdict: **CHANGES REQUIRED**

The synthesized try-carrier implementation leaves valid nested-function carriers undefined and can strip required traits from ordinary unions sharing the same generated enum. Both produce check/build parity failures.

Full review comments:

- [P1] Scan nested functions for synthesized try carriers — /private/tmp/sifr-m10-review-pass14-cc13a8a49/crates/sifr_codegen/src/hir_analysis/queries/queries_impl.rs:819-823
  When a nested function contains a try block that can raise two exact error types, this `LOCAL_SCOPE_ONLY` traversal skips its body, and nested functions are not scanned separately by `collect_union_types`. Codegen still references the synthesized carrier enum while no definition is emitted, so valid code passes checking but fails Rust compilation; include nested function bodies when collecting these carriers.

- [P1] Keep ordinary union traits on reused carrier enums — /private/tmp/sifr-m10-review-pass14-cc13a8a49/crates/sifr_codegen/src/union_type_helpers.rs:197-202
  When the same error union is used both as a try carrier and as an ordinary source-level union, `try_error_carrier_enums` marks their shared generated enum and these conditions suppress `PartialEq`, `Eq`, and `Hash` globally. The type checker still permits equality and hash-based collections when all member errors support those traits, so such programs pass checking and then fail native compilation; carrier-specific conversion support should not remove traits required by ordinary uses of the same enum.

## Remediation

Carrier discovery now traverses nested-function bodies. Union registration also
records whether an enum is used as an ordinary source value in addition to its
compiler-only carrier role, so only reused ordinary unions regain their proved
`PartialEq`, `Eq`, and `Hash` derives while carrier-only enums retain the
minimal trait surface.

Permanent unit and native E2E regressions cover nested exact-error routing and
one structural union shared by handler dispatch, equality, and hash-set use.
The complete codegen suite passes `854/854`; the native
`nominal_identity_alias_paths.sifr` fixture compiles and runs. The authoritative
create-PR facade passes every blocking lane in `612.79s`: Python interop
`12/12`, runtime platform `28` variants with one capability-gated skip, and E2E
`131/131` with signature `7c39b8c1dd4fec7c` and `41/42` cache hits. The warm
wall-time notice is a non-blocking advisory. Full merge validation and a fresh
whole-diff re-review remain pending.
