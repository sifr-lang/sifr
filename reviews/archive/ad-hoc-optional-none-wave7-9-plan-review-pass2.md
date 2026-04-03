## Review Pass 2: Optional/None Wave 7-9 Plan

Reviewed artifact:
- `issues/ad-hoc-optional-none-and-narrowing-wave7-9-root-cause-plan-2026-03-29.md` (revised draft)

Verdict:
- **Ready**

Blocking issues:
- none

Non-blocking improvements:

1. Make per-wave negative-test matrices explicit:
   - unproven Optional sites must continue to fail
   - reassignment/join paths that are genuinely nullable must remain `T | None`
   - call-boundary/container cases must reject nullable values when declarations are non-nullable
2. Make the `infer.rs` contingency in wave-8 explicit with an evidence threshold before touching that layer.

Principle compliance:
- revised plan is compliant with Sifr principles:
  - explicit `Option`/`Result` preserved
  - no implicit `Option[T] -> T` coercion
  - no hidden unwrap path
  - truthiness handling remains sequence-scoped
  - no fixture-specific hacks

