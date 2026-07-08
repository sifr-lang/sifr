## Review: M2a private declaration diagnostics slice

**Scope and correctness**

- `is_sysroot_private_declaration()` (mod_context.rs:201) is a clean accessor gated on the enum variant. It sits parallel to `is_stdlib_lowering()` without conflating public/private stdlib — appropriate, since M2a only wants to tighten the private declaration surface.
- `reject_unsupported_private_declaration_constant` (module_constants_lowering.rs:141) early-returns for anything that isn't `SysrootPrivateDeclaration`. `User` and `SysrootPublicStdlib` origins keep their existing silent-drop behavior — matches the "normal user behavior unchanged" requirement.
- The diagnostic is only reached after `validate_annotated_constant_initializer` has settled: the code checks `error_count()` first and returns None if new errors landed, and returns early with any `folded_value`. So no double-report on genuine type mismatch, and no misfire on a folded literal.
- The integer-constant alias fast path (`lower_module_integer_const_expr` at module_constants_lowering.rs:56) is untouched, so `const_integer_values` compile-time integer facts continue to work.
- Diagnostic code `SIFR-TYPE-0012` (`TYPE_UNSUPPORTED_EXPRESSION_FORM`) is already registered (registry.rs:51, 683). Reusing it is consistent with other "declaration form isn't supported here" sites.
- The unchanged `stdlib/_sifr/math.sifr` only contains literals and `1.0/0.0` binops, and `stdlib/sifr/math.sifr` uses `from _sifr.math import …` rather than local scalar aliases — so nothing in the current sysroot regresses.

**Tests**

- `private_declaration_scalar_module_constant_alias_is_diagnostic` (module_constants_lowering.rs:308) asserts the diagnostic's code and message text — good.
- `private_declarations_collect_annotated_scalar_module_constants` (line 275) confirms literal and BinOp initializers still lower cleanly.
- The existing `annotated_scalar_module_constant_type_mismatch_is_diagnostic` (line 291) still passes through the same lowering path with `TYPE_MISMATCH` semantics, so the ordering (mismatch first, then unsupported form) is exercised implicitly.
- Gaps worth noting (not blockers): no explicit negative test that a user module (`lower_module`) with `alias: float = pi` still drops silently — the guard is very small, but a one-line positive test would lock the "user behavior unchanged" invariant. Also no test for a private declaration constructor-call initializer (that path is listed as supported in the message and in `is_supported_annotated_module_constant_expr`).

**Housekeeping**

- File sizes noted (326 / 563 lines) — both well under the 900-line cap.
- The change adds ~30 net lines of straightforward logic and one test; no responsibility-boundary changes that would nudge maintainability guardrails.
- Comment-free code is fine here — the intent is clear from names.

**Merge safety**

This is a well-scoped M2a sub-PR: it closes the "silently dropped private-declaration alias" gap with a structured diagnostic while preserving user, public-stdlib, and integer-facts paths verbatim. Recommended follow-up (not blocking): add a user-mode negative test and a constructor-call positive test in a later slice.

READY
