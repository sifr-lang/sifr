# Sifr Compiler Architecture - Python Feature Gap Review

Date: 2026-02-14
Reviewed file: `.cursor/plans/sifr_compiler_architecture_fa3c10ee.plan.md`

## Goal

Identify which frequently used Python language features are missing, deferred, or intentionally different in the Sifr architecture plan.

## Findings (ordered by impact)

### High 1: Python exception model is intentionally replaced

Sifr intentionally does not keep Python's exception-driven control flow model. It reinterprets `try`/`except` over `Result`, and `raise` returns `Err(...)`.

Why this is a gap for Python developers:
- A large amount of production Python relies on exception propagation patterns.
- Migration of existing code will require broad rewrites, not just syntax changes.

Plan evidence:
- Safety philosophy: "Exceptions are not errors."
- M4 language features: `Result[T, E]`, `Option[T]`, `try`/`except` as pattern match, `?` operator.

---

### High 2: Function call ergonomics are under-specified for common Python use

The plan introduces `*args`/`**kwargs` in M14 (late), but does not define complete call semantics early:
- keyword arguments
- keyword-only arguments
- positional-only arguments
- unpacking behavior in call sites

Why this matters:
- These are routine in modern Python APIs and frameworks.
- Delaying them to M14 creates an adoption cliff for real-world code.

Plan evidence:
- `*args` / `**kwargs` appears in M14 language features.
- No explicit earlier contract for keyword/positional calling rules.

---

### Medium 1: `break` / `continue` are inconsistent in spec text

The frontmatter todo states M2 includes `break/continue`, but the M2 language feature list does not explicitly include them.

Why this matters:
- Creates ambiguity for parser/checker/codegen ownership and tests.
- Encourages drift between implementation and plan.

Plan evidence:
- Frontmatter M2 todo: "While/for loops, break/continue, range() support."
- M2 feature list names loops/range but omits explicit `break` and `continue`.

---

### Medium 2: Slicing/indexing parity likely incomplete

The plan defines `[a:b]` slicing and safe indexing contract, but does not explicitly specify:
- negative indices (`a[-1]`)
- step slicing (`a[::2]`, `a[::-1]`)
- full slice combinations

Why this matters:
- Negative and stepped slices are heavily used Python idioms.
- Lack of explicit contract will produce inconsistent expectations.

Plan evidence:
- Slice semantics are documented as `list[a:b]` and `str[a:b]`.
- No explicit syntax/semantics for negative index or step.

---

### Medium 3: OOP model omits several common Python mechanisms

The plan covers classes, methods, properties, protocols, and single inheritance, but does not explicitly define support for:
- `super()`
- `@classmethod`
- metaclasses
- `__slots__`

Why this matters:
- These are common in framework code, domain models, and performance-sensitive classes.

Plan evidence:
- M5 explicitly states single inheritance via trait delegation.
- No dedicated sections describing the items above.

---

### Medium 4: Import side effects are intentionally removed

Sifr package init semantics explicitly avoid import-time side effects.

Why this matters:
- Python ecosystems often rely on import-time registration and module initialization.
- Porting such packages will require explicit initialization redesign.

Plan evidence:
- M6: `__init__.sifr` defines exported API only; no side effects on import.

---

### Medium 5: Numeric behavior diverges from Python `int`

Sifr uses checked `i64` behavior with `Result` on overflow, rather than Python's arbitrary-precision integer model.

Why this matters:
- Numeric-heavy Python code may need redesign for overflow handling.

Plan evidence:
- Safety adaptation rules explicitly call out checked `i64` arithmetic and overflow as `Result`.

## Additional commonly expected Python features not explicitly covered

No explicit plan coverage found for:
- `global`, `nonlocal`
- `finally`
- `yield from`
- `async with`
- full keyword argument model (keyword-only, positional-only)

Note: absence here means "not explicitly specified in this plan document", not necessarily "impossible to add."

## Recommended plan improvements

1. Add a "Python Compatibility Matrix" section:
   - feature
   - status (`supported`, `deferred`, `intentional divergence`)
   - milestone
   - migration note

2. Pull high-usage call semantics earlier than M14:
   - keyword args
   - unpacking behavior
   - argument mode constraints

3. Clarify loop and slicing contracts now:
   - make `break`/`continue` explicit in M2 language features + tests
   - specify negative index and step slicing behavior

4. Add an explicit "Divergences from Python" appendix:
   - exceptions -> `Result`/`Option`
   - no import side effects
   - checked integer overflow
   - ownership/move semantics

## Conclusion

The plan is strong and detailed, but the largest practical migration gaps for everyday Python developers are:
- exception model replacement,
- incomplete early function-call ergonomics,
- under-specified slicing and some runtime language behaviors.

Making these explicit in a compatibility matrix will reduce implementation drift and give users a clear migration path.
