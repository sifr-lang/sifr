# Planning Review — Stdlib Namespace Contract And Compatibility Cleanup

## Verdict: `CHANGES_REQUESTED`

The plan's locked decisions, exit gates, and milestone shape are sound, and the no-codemod / no-bare-alias posture is consistent throughout. But there are concrete code-reality mismatches and underspecified surfaces that would force the implementer to make non-trivial judgment calls. Fix the items below before declaring `READY`.

---

## Blocking findings

### 1. Phase intro misstates the current contract
> "lowering still contains a hidden call-expression compatibility path... while equivalent bare stdlib imports are rejected."

Bare stdlib imports are **not actually rejected today**. `resolve_imports_early` in `crates/sifr_lowering/src/lower/imports.rs:58–189` only iterates `Stmt::ImportFrom`, and for any `module_key` not present in `externals.*` (e.g. `"math"`, `"collections"`, `"json"`) it simply no-ops. There is no `IMPORT_BARE_STDLIB` diagnostic in `import_diagnostics.rs`; bare imports silently succeed and the failure surfaces downstream as an unknown-name error at the use site (or, today, succeeds via the compat path for `math.fmod` / `deque(...)` / etc.).

**Fix:** Reword the framing as "bare stdlib imports today silently no-op and only fail downstream; this phase makes them an explicit diagnostic." Then state in M1 that a new diagnostic is being *added*, not rerouted.

### 2. `Stmt::Import` is entirely unaddressed
Every example in the Diagnostics Contract is a `from X import Y` form. `resolve_imports_early` only handles `Stmt::ImportFrom`, so plain `import math`, `import collections`, `import json` go completely unvisited. After M2 removes the compat path, `math.fmod(x)` becomes a name-error on `math` rather than producing the promised "use `sifr.math`" diagnostic.

**Fix:** In M1, explicitly cover both shapes:
- `import math` / `import math as m`
- `from math import sqrt`
- `import collections.abc` (dotted) — specify whether this is also matched and how the suggestion reads.

Add positive coverage for at least one `Stmt::Import` case.

### 3. The two `defaultdict` early returns bypass `ensure_synthetic_stdlib_import` and are not removed by M2's stated scope
Inspect `crates/sifr_lowering/src/lower/compat_imports.rs:21–23` and `:44–45`:

```rust
if name.id.as_str() == "collections" && attr.attr.as_str() == "defaultdict" {
    return Some("defaultdict".to_string());           // collections.defaultdict(...)
}
...
"defaultdict" => return Some("defaultdict".to_string()), // bare defaultdict(...)
```

These short-circuit before ever calling `ensure_synthetic_stdlib_import` and route directly to `lower_defaultdict_constructor_call`. M2's bullet ("Remove the lowering path that maps `math.*`, `heapq.*`, `collections.*`, `deque(...)`, and `Counter(...)` to hidden `sifr.*` imports") technically covers `collections.*` but the implementer cannot tell whether removing the `collections.defaultdict` short-circuit belongs to M2 (since it is under `collections.*`) or M3 (since it is the defaultdict binding work).

**Fix:** Explicitly say:
- M2 removes only synthetic-import-producing paths: `math.*`, `heapq.*`, `collections.*` non-defaultdict, bare `deque`, bare `Counter`.
- M3 removes the two defaultdict short-circuits (`collections.defaultdict` attribute call and bare `defaultdict` name) together with the unconditional builtin recognition, and replaces them with the imported-binding check.

This sequencing also matters because between M2 and M3 the bare `defaultdict(list)` form must keep working (otherwise the in-tree fixtures temporarily break across PRs).

### 4. No registry of "known stdlib module tails" — the diagnostic has nothing to decide against
The Diagnostics Contract requires the diagnostic to only fire when the bare root "match[es] a known Sifr stdlib module tail," and to *not* mask future user/package modules with the same name. Today there is no centralized list of public `sifr.*` module names — `sifr_stdlib::STDLIB_SOURCES` is the closest thing, but its keys are full `sifr.foo` paths.

**Fix:** Specify in M1:
- The diagnostic consults the set of module tails derived from `sifr_stdlib::STDLIB_SOURCES` keys (strip `sifr.` prefix), with explicit handling for dotted submodules (e.g. `sifr.collections.abc` → tail `collections.abc`).
- Internal-only modules and `_sifr.*` are filtered out of this set.
- Resolution order: top-level user/package resolution attempted first; bare-stdlib diagnostic fires only when top-level resolution fails. State this ordering in the contract — the current bullet ("unless they are resolved as user or package modules through normal top-level resolution") is correct but not pinned to an order.

### 5. Missing diagnostic code name and machine-readable shape
> "include machine-readable diagnostic data for the bare module and suggested `sifr.<module>` path"

The codebase has a consistent diagnostic-code convention (e.g. `IMPORT_FORBIDDEN_INTRINSIC` for `_sifr.*` rejection). The plan does not pin a code name or JSON field shape, leaving both as PR-time bikeshedding.

**Fix:** Specify the code (`IMPORT_BARE_STDLIB` or equivalent) and the structured fields (e.g. `data.bare_module`, `data.suggested_module`, `data.symbols: [...]`). Reference how it mirrors `IMPORT_FORBIDDEN_INTRINSIC` so the implementer can copy the registration shape.

### 6. M2 leaves the consumer of `synthetic_imports` undocumented
The `synthetic_imports` field is *consumed* at `crates/sifr_lowering/src/lower/mod_impl.rs:647` (`imports.extend(ctx.synthetic_imports.clone())`), and `synthetic_import_aliases` is consumed inside `ensure_synthetic_stdlib_import` itself. The bullet "Remove `synthetic_imports` and `synthetic_import_aliases` from lowering if no remaining producer uses them" only addresses producers.

**Fix:** Add a bullet to M2: "Remove the consuming site that extends final `imports` with `ctx.synthetic_imports`, and verify by grep there are no other readers/writers of either field across the workspace." Cite the file path.

### 7. M4 guardrail grep won't match what the plan intends
> `rg "__compat_sifr_|resolve_python_compat_call_alias|resolve_bare_python_compat_call_alias" crates/sifr_lowering/src crates/sifr_codegen/src -g '*.rs'` returns no production hits except intentionally documented guardrail tests

After M2/M3, `resolve_python_compat_call_alias` and `resolve_bare_python_compat_call_alias` are *deleted*, so they cannot appear in guardrail tests; the grep needs no exception. The `__compat_sifr_` prefix is the only stable invariant. Separately, the "except intentionally documented guardrail tests" carve-out is vague — there is no such test today.

**Fix:**
- Narrow the guardrail to just `__compat_sifr_` substring in production lowering/codegen.
- Either drop the "guardrail tests" exception or specify exactly which file is the documented guardrail (e.g. `crates/sifr_lowering/tests/no_synthetic_stdlib_imports.rs`).
- Note that `__compat_defaultdict_*` is intentionally *not* in the regex (decision 7).

### 8. `__compat_defaultdict_*` naming becomes misleading after this phase
Decision 7 keeps `__compat_defaultdict_int|list|set` as typed representation aliases until a later phase. Once every other `__compat_*` is removed, the `__compat_` prefix is misleading — it now names typed-representation aliases, not compat shims. This is a documentation hazard for future readers.

**Fix:** Add a one-sentence note in the Defaultdict Contract section explicitly acknowledging that the `__compat_defaultdict_*` alias names are inherited from the prior compat scheme, are *not* a namespace-compat artifact, and will be renamed when the data-structure representation phase replaces them. Or, better, rename them in M3 (`__sifr_defaultdict_int` etc.) since you are already touching that lowering.

---

## Non-blocking but worth tightening

### 9. Specify the defaultdict factory whitelist explicitly
The plan repeats "`defaultdict(int/list/set)`" four times but never says what happens for `defaultdict(dict)`, `defaultdict(MyClass)`, `defaultdict(lambda: 0)`. Today `lower_defaultdict_constructor_call` matches `Expr::Name` then calls `defaultdict_alias_and_value_type(factory_name)` which only knows `int|list|set`. Anything else is silently rejected.

**Fix:** Add one sentence to the Defaultdict Contract: "Only `int`, `list`, and `set` factories are recognized; other factory expressions continue to produce the existing unsupported-factory diagnostic. Expanding the factory set is out of scope for this phase."

### 10. `from sifr.collections import defaultdict as defaultdict` aliased-to-self
Mention this is treated identically to a plain import (no aliasing). Trivial but worth pinning to avoid a follow-up question.

### 11. `import sifr.math` (statement form, dotted) under `sifr.*`
The plan covers `from sifr.math import ...` but not `import sifr.math` or `import sifr.math as m`. Specify whether these are accepted and how member access is resolved — this could otherwise become a follow-up issue immediately after M2.

### 12. Validation commands name non-existent test modules
M1 cites `cargo test -p sifr_lowering name_import_diagnostics_tests`. There is no such test file today (the existing diagnostic helpers live in `name_diagnostics.rs` and `import_diagnostics.rs`). Either rename to the *new* test file being added in M1 (e.g. `bare_stdlib_import_diagnostics`) and say so, or drop the specific name and say "the bare-stdlib diagnostic tests added in this milestone."

### 13. Existing demos/e2e relying on bare forms — count is non-trivial
The exploration counted roughly 20 e2e fixtures using bare `deque(`, `Counter(`, `defaultdict(`, `math.fmod(`, `heapq.*`. M2 and M3 both say "Update `.sifr` fixtures and demos to import stdlib symbols explicitly" without scoping. Spell out:
- Which milestone owns which file (e.g. M2 covers fixtures using `math.fmod`, `heapq.*`, bare `deque`, bare `Counter`; M3 covers fixtures using `defaultdict` or `collections.defaultdict`).
- Whether `crates/sifr/tests/e2e/fail/` needs *new* fixtures for the rejected forms (which it does — see M1 validation, which currently only says "verification fixture update for human/json/compact output if a CLI diagnostic fixture is added").

### 14. M1 docs target
"Add a concise user-facing docs page" — specify the path (e.g. `docs/language/stdlib-imports.md`) and whether it gets indexed from the docs nav. Otherwise the implementer has to make a navigation decision and may inadvertently orphan the page.

### 15. `typing` / `enum` exemption documentation
Locked decision 5 says `typing`/`enum` remain closed frontend imports. The current short-circuit lives at `imports.rs:74` (`if is_absolute_import && (module_name == "typing" || module_name == "enum") { continue; }`). The architecture-doc invariant table should call this exemption out explicitly so readers don't infer that *all* CPython names are rejected.

---

## Summary

Tighten the framing (1), close the `Stmt::Import` gap (2), pin the M2/M3 split for defaultdict (3), specify the stdlib-tail registry and resolution order (4), name the diagnostic code and shape (5), document the synthetic-imports consumer removal (6), make the guardrail grep match reality (7), and decide on the `__compat_defaultdict_*` naming (8). Once those are in, the plan is implementable with low ambiguity.
