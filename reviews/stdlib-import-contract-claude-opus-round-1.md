I have enough context. The review file is empty — this is round 1 of analysis. The repo's actual stance: `from x import ...` is the only supported import form (bare `import x` is already a hard error), `sifr.*` is current canonical, `typing`/`enum` are bare-named exceptions, ~50 stdlib modules under `sifr.*`, multiple intentional name divergences (`sifr.os.remove_file`, `sifr.math.abs_val`, Result/Option vs exceptions), and Phase 37 anticipates future Python interop.

# Sifr stdlib import contract — design analysis

## Framing

Three real options, not two:

- **A. Bare-only**: `from math import sqrt`. Drop `sifr.*` entirely.
- **B. `sifr.*`-only** (status quo): `from sifr.math import sqrt`. Bare stdlib names rejected.
- **C. Dual / aliased**: both work, one canonical.

The status quo already has a wart: `from typing import ...` and `from enum import ...` are bare and "non-activating support imports" (phase 17). So Sifr today is technically option C-with-asymmetry, scoped narrowly to type-system support.

## Multi-angle analysis

### 1. User ergonomics
- **Bare**: 5–8 fewer characters per import, matches muscle memory of every Python dev. Real but small.
- **`sifr.*`**: Constant visual signal "this is Sifr, not Python." Forces deliberate engagement — desirable when names *and semantics* diverge.
- Verdict: **bare wins narrowly**; not decisive.

### 2. Migration / parity from Python
- The *dangerous* failure mode isn't typing `sifr.` — it's a user copying `os.remove("x")` from Python, hitting `sifr.os.remove_file` (renamed), `Result[None, IoError]` instead of an exception, or a missing API. Bare imports make the code *look* portable when it isn't.
- Aliasing the namespace fakes parity that doesn't exist behaviorally. The architecture doc has a whole table of divergences (`abs_val`, `pow_val`, `move_file`, `is_match`, `remove_file`, `shuffle returns new list`, etc.). Bare names amplify the surprise.
- A **codemod** (`sifr migrate`) that rewrites Python source into Sifr — translating `import math` → `from sifr.math import …`, flagging untranslatable APIs with diagnostics — is a *strictly better* migration story than aliasing. It produces correct code instead of code that compiles and lies.
- Verdict: **`sifr.*` wins**. Bare imports lower a real safety barrier for a small ergonomic gain.

### 3. Semantic honesty
- "Python syntax, Rust semantics, Sifr safety lens" is the language identity. The namespace prefix encodes that: `sifr.math.sin` ≠ `math.sin`. Different return shape, different error model, no panic.
- `sifr.*` is a name-mangling that mirrors the actual semantic divergence. Bare names assert an equivalence that isn't true.
- Verdict: **`sifr.*` wins decisively**.

### 4. Package / import resolution
- Resolver currently has clean three-tier ownership: `_sifr.*` (intrinsics), `sifr.*` (stdlib), top-level (user / future third-party). Each tier has one owner.
- Bare stdlib **collapses two tiers into one** — stdlib and third-party packages now share the top-level namespace. CPython lives with this and it's a known wart (consider what happens when a user wants a module named `string.sifr` or `time.sifr` locally).
- Demos already show `from helper import …` for local modules. Adding bare stdlib means resolution has to disambiguate `from math import …` between a hypothetical local `math.sifr`, a third-party `math` package, and embedded stdlib. Resolution rules become longer; diagnostics get worse.
- Verdict: **`sifr.*` wins strongly**. Three-tier separation is architecturally sound and worth preserving.

### 5. Future package manager (Phase 37)
- `sifr.toml` already exists; packages will arrive. With `sifr.*` reserved, third-party packages own the bare top-level cleanly. Without it, every new stdlib module *forever* removes a viable top-level package name.
- Phase 37 also mentions deferred Python interop (`pyproject.toml`, `uv.lock`). If Sifr ever wraps real CPython modules via a bridge, the *natural* spelling for "actually use CPython's math" is `from math import …` — i.e., bare. Squatting that namespace now with Sifr-flavored wrappers burns the most ergonomic door we have for future interop.
- Verdict: **`sifr.*` wins strongly**. This is the highest-leverage long-term argument.

### 6. Namespace collision
- Bare names that collide with built-ins, common variables, or user modules: `string`, `time`, `os`, `re`, `json`, `csv`, `io`, `bytes`, `random`, `secrets`. Several of these are extremely common as local variable / module names.
- `sifr.*` makes collisions impossible by construction.
- Verdict: **`sifr.*` wins**.

### 7. Teaching / docs
- Bare: docs read like CPython. Easier "first hour." Hides divergence.
- `sifr.*`: every example carries the prefix. Constant reminder of "this is Sifr." Slightly heavier first impression.
- Net: bare wins lesson 1, loses lessons 2–N when divergences surface unexpectedly.
- Verdict: roughly **even**, leans `sifr.*` once you account for the long tail.

### 8. Implementation complexity
- Status quo (`sifr.*`-only) is what's built. Already enforced via embedded registry, codegen, resolver precedence.
- Bare-only: rewrite resolver, codegen module paths (`crate::sifr::math` → ?), update ~50 modules + ~100 demos + docs + intrinsic mapping. Touches `mod_impl.rs`, `module_prescan.rs`, `lib_modules_and_codegen.rs`, frontend, driver, package discovery. Sweeping but tractable.
- Dual / aliased: simplest patch (canonicalize bare → `sifr.*` at name resolution), but you now permanently maintain two spellings, must teach users which is canonical, every diagnostic must pick one, search/grep yields mixed hits, codemods must canonicalize. High *ongoing* cost for the lifetime of the language.
- Verdict: **`sifr.*`-only is cheapest**; dual is the most expensive long-term despite the smallest initial diff.

### 9. Language identity (long-term)
- Sifr's pitch: "Python you can trust; if it compiles, it works." The pitch is *not* "Python that secretly compiles." The prefix encodes that distinction at the import line — the place users look 50 times a day.
- Languages that tried to be source-compatible with another (e.g. CoffeeScript→JS, Crystal→Ruby) found that hiding the semantic gap caused real confusion. Languages that owned the distinction (Elixir/Erlang, Kotlin/Java, TypeScript/JS) thrived.
- Verdict: **`sifr.*` wins**.

### 10. Ecosystem expectations
- Python devs *expect* `from math import sqrt` to mean Python's math. If we give them that spelling for a non-Python module, we are setting up a "looks like a duck, ships exceptions" trap.
- Rust, Go, Swift, Kotlin, .NET — every modern stdlib namespaces under a vendor/language prefix (`std::`, `core::`, `Foundation.`, `kotlin.`, `System.`). Python is the outlier, and a notoriously contentious one.
- Verdict: **`sifr.*` wins**.

## Hidden risks

1. **Typing/enum precedent** — the existing bare-name exceptions can be cited as "you already broke the rule, finish the job." Address head-on by classifying them as *type-system support imports*, not runtime stdlib: they emit no Rust, have no embedded source, are processed by the frontend's typing layer. This is a principled distinction, not a fudge. Document it explicitly. Optionally rename to `sifr.typing` / `sifr.enum` later, but it's not worth churn — the precedent doesn't generalize.

2. **Codemod debt** — if you ever onboard real Python code, you owe a translator. If you skip building it, users will hand-translate badly and bug reports about silent semantic divergence will dominate. The codemod is the migration story; aliasing is a *substitute* for that story and a worse one.

3. **Phase 31.5 framing** — the question references "phase 31.5 closure of Python source parity." If "parity" is taken to mean *source-level lookalikeness*, that's a different goal than what the architecture actually pursues (behavioral parity through the safety lens). Lock the definition: **parity = behavioral, not lexical**. Source remains `sifr.*`; behavior matches CPython where safe, diverges where Sifr's invariants demand it. Otherwise this debate will recur every phase.

4. **Discoverability regression** — `sifr.<TAB>` in an editor is *better* than bare names for discovery (one prefix → list of all modules). Don't give that up.

5. **`_sifr.*` blocked → `sifr.*` deprecated → bare canonical** would imply the second tier becomes useless. Don't let aliasing slide into a deprecation track for `sifr.*`; if you ever ship dual, the docs must be unambiguous that `sifr.*` is canonical and bare is the alias, not the reverse.

## Recommendation

**Keep `sifr.*` as the permanent public contract. Do not add bare-name aliasing.** Close it as a deliberate language design decision.

Concrete actions:

1. **Promote the decision to the architecture doc** as a permanent invariant alongside the `_sifr.*`/`sifr.*`/user three-tier ownership model. Cite: behavioral semantic divergence, future Python interop preservation, third-party package namespace cleanliness.
2. **Reframe the typing/enum exceptions** in `internal_docs/architecture.md` and `internal_docs/phases/17_import_and_externals_correctness.md` as *type-system support imports*, not stdlib. Make the classification principled. (Renaming is optional; not worth the churn.)
3. **Improve the diagnostic** for `from math import sqrt`-style attempts. Currently this hits "unsupported import statement" or "unknown module". Make it a dedicated, suggestion-bearing error:
   `error: bare stdlib import 'math' — Sifr stdlib lives under 'sifr.*'`
   `help: use 'from sifr.math import sqrt' (note: names and error semantics diverge from CPython, see <link>)`
4. **Plan a Python-to-Sifr codemod** as the official migration story (own phase or milestone in 38 docs / 43 interop). Rewrites bare → `sifr.*`, flags untranslatable APIs, surfaces Result/Option contract differences. This is the *real* answer to "how do Python devs migrate."
5. **Document in user-facing docs** (`docs/`): one short page titled "Why `sifr.*`?" — explains the three-tier model, the semantic-honesty rationale, and the future-interop point. So this question doesn't get re-litigated.

## Migration / deprecation path

None needed. The recommendation preserves the status quo. The single deliverable that flows out is the improved diagnostic + docs page + (longer-term) the codemod tool. If at some future phase Python interop ships and the team wants bare `import math` to mean *real CPython's math via a bridge*, the bare namespace is still free to take on that meaning — which is exactly the optionality we'd lose by aliasing now.

**Bottom line**: the ergonomic case for bare is real but small; the long-term costs (semantic dishonesty, lost interop namespace, three-tier collapse, dual-spelling maintenance, collision surface) are large and irreversible. `sifr.*` is the right permanent contract.
