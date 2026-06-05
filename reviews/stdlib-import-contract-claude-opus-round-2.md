# Sifr stdlib import contract — Round 2 (stress-test)

## What changed since Round 1

Round 1 closed on **keep `sifr.*` as the permanent contract; reject bare stdlib imports; build a codemod for migration; improve the diagnostic; document the three-tier model.**

Two facts surfaced in re-grounding that sharpen Round 2:

1. **The compat layer is louder than Round 1 acknowledged.** `crates/sifr_lowering/src/lower/compat_imports.rs:11-52` already auto-synthesizes stdlib imports for *call sites* — bare `math.sqrt(x)`, `heapq.*`, `collections.defaultdict`, and bare names `defaultdict`/`deque`/`Counter`. This means today's policy is **internally inconsistent**: `math.sqrt(x)` compiles with no import, but `from math import sqrt` is rejected as `unknown import target: 'math'`. A Python user copying code experiences randomized success/failure depending on whether they imported.
2. **The bare-import diagnostic is generic** (`IMPORT_UNKNOWN_SOURCE_MODULE` → `"unknown import target: 'math'"`). No hint about `sifr.math`. The promised "dedicated diagnostic" from Round 1 has not landed.

These two facts shift the question. Round 2 isn't "should we add bare imports?" — it's "do we close the bare-name leak, or widen it?"

---

## Best case for CPython-style bare stdlib imports

Steelman, no hedging:

- **Adoption funnel.** The first line of nearly every Python file is `import …`. That line is also the most common thing a Python dev pastes into a Sifr file when evaluating the language. Every `sifr.` prefix is a "this isn't really Python" signal at the moment of evaluation. If Sifr's growth depends on Python developers, the import line is where you keep the friction lowest.
- **The compat-call shim is already evidence of intent.** The lowering layer already accepts `math.sqrt(x)` without imports. Symmetric treatment for `from math import sqrt` only finishes a job we've half-started. The asymmetry today is the worst of both policies.
- **The "future CPython interop namespace" argument is speculative.** Phase 37 interop is unbuilt. If a CPython bridge never ships, we paid a permanent ergonomic tax to reserve a door nobody walks through. Even if it does ship, the bridge could use a different spelling (`from cpython.math import …` or `from py.math`) without losing anything material.
- **Collisions are rare.** CPython has lived with bare stdlib in a global namespace for 30 years. The actual reported pain is small and tooling-tractable (linters flag shadowing).
- **A codemod is real engineering work.** A robust `sifr migrate` translator competes with stdlib coverage, language features, and tooling for the same team-hours. Bare aliasing solves the most common migration case (the import line) with effectively zero ongoing cost beyond the alias table.
- **Teaching.** "Look — your Python file compiles, but Sifr caught these bugs." That demo only works if the imports look like Python. `sifr.*` breaks the spell on line one.

That is the strongest honest case. It is non-trivial. Round 2's job is to test it against Sifr's guarantees.

---

## Variant-by-variant evaluation

### Variant 1 — bare aliases only for CPython-compatible modules/members

Curated allowlist of "identical name + identical signature + identical semantics" entries gets bare spelling; everything else stays `sifr.*`.

- **Runtime panics / Result-Option contract.** `math.sqrt(-1.0)` in CPython returns `nan` or raises `ValueError`. Sifr's safety lens routes domain errors through `Result`. Almost every interesting stdlib function diverges *somewhere* — return shape, error model, naming (`abs_val`, `pow_val`, `remove_file`, `move_file`, `is_match`). The set of "truly identical" calls is small and gets smaller as Sifr matures.
- **Docs / discoverability.** The rule becomes "it depends on the function." Users learn the allowlist by trial and error. That's a strict regression in clarity over today.
- **Maintenance.** Every new stdlib module ships with a classification step ("is this CPython-compatible?"). Every safety refinement (turn a panic-on-bad-input into a `Result`) potentially *removes* a name from the allowlist — a quiet breaking change. The allowlist becomes a permanent governance artifact.
- **Resolver.** Two name spaces now overlap (top-level user/third-party and a curated subset of stdlib bare names). Diagnostics must disambiguate.

**Verdict: worst of the five.** Maximum complexity, minimum honesty, no clear migration story.

### Variant 2 — bare aliases gated by `sifr.toml` setting or edition

`[language] python_compat_imports = true` (or an edition string) opts a package into bare names.

- **Ecosystem stability.** Packages with the flag look syntactically different from packages without. A reader of one package can't tell which spelling is in force without flipping to the manifest. Code review, search, refactor tooling, and LSP all have to track the per-package mode.
- **Cross-package dependencies.** A package with the flag on, depending on a package with the flag off, is fine at runtime but jarring at the source level. The first time a user imports a function from a flag-off package and has to switch styles mid-file, the rationale collapses.
- **Permanence.** Once any third-party package adopts the flag, removing it is a breaking change in the ecosystem. The "edition" is forever, like Python 2/3 `__future__`. Sifr does not have the user base to absorb that kind of split.
- **Package-manager namespace stability.** With the flag on, the bare top-level is occupied by stdlib *for that package's source*, which means a third-party package literally named `math` either can't be installed there or must shadow stdlib. The package manager must take a position; either choice is bad.
- **Interop reservation.** Same problem as universal bare imports, with extra surface area.

**Verdict: the Python 2/3 trap dressed as flexibility.** Don't ship.

### Variant 3 — bare aliases only in `--python-compat` migration mode, with warnings

CLI/env flag, not a per-package setting. Compiler accepts bare stdlib imports but emits warnings.

- **Strictly weaker than a codemod.** A `--python-compat` mode lets users *run* unmodified Python source. A codemod (`sifr migrate`) *transforms* it into canonical Sifr source. The codemod's output is reviewable, diffable, and lives in source control. The compiler mode's output is invisible. Every problem the mode solves, the codemod solves better.
- **Warnings rot.** "Deprecated from day one" warnings get silenced or ignored. Code written under the mode escapes into production via "I'll fix it later."
- **Diagnostic clarity.** When a user runs Sifr without the flag, they get a different result for the same source. Bug reports become "works on my machine + my flag."
- **One legitimate use:** running upstream Python during a *single* migration session, then turning the flag off. But that's exactly what the codemod does, and the codemod produces durable source artifacts instead of an ephemeral flag state.

**Verdict: dominated by the codemod.** Build the codemod; don't build this.

### Variant 4 — bare aliases as deprecated transitional sugar with canonical diagnostics

Bare `from math import sqrt` is accepted, emits a `DEPRECATED_BARE_STDLIB_IMPORT` diagnostic, points at `sifr.math`.

- **Either you remove it on a date or you don't.** If you remove it on a date, you're Variant 3 (a migration mode, with the same codemod-dominates problem). If you don't, you've shipped permanent dual spelling with a guilt label.
- **Search / grep / refactor.** Two spellings in the wild, forever. Every diagnostic must pick one canonical form. Every doc example must pick one. Code review picks fights about it. New contributors ask which is "right."
- **Adoption pressure.** Third-party packages that pick the deprecated form ("looks like Python") resist removal because their users have written code against it.
- **Better than Variant 1** only because it's uniform across the stdlib, not curated. That's a low bar.

**Verdict: permanent dual-spelling with extra steps.** Don't ship.

### Variant 5 — bare imports reserved for future real CPython interop

Status quo, formalized as policy. `sifr.*` is the only spelling for Sifr stdlib. Bare top-level names are reserved for: (a) user/third-party packages today, (b) eventual real-CPython-via-bridge imports if Phase 37 ships interop.

- **Runtime guarantees.** Untouched. `sifr.*` is the only spelling that lowers into Sifr's Result/Option API; no path exists where a user accidentally hits a CPython-shaped API thinking it's Sifr's.
- **Resolver.** The clean three-tier (`_sifr.*` / `sifr.*` / top-level) holds. Each tier has one owner. Diagnostics stay sharp.
- **Package-manager.** Third-party packages own bare top-level cleanly forever.
- **Interop optionality.** If Phase 37 ever lands a CPython bridge, `from math import sqrt` is the *natural* spelling for "real CPython's math," and that door is still open. If the bridge never ships, the door cost nothing.
- **Docs.** One rule, one example, one canonical spelling.
- **Cost.** The ergonomic loss on the import line. The codemod debt. The need for a proper diagnostic.

**Verdict: this is the right policy.** It's what we already do *for imports*. The work is to (a) fix the diagnostic, (b) ship the codemod, (c) **decide what to do about the compat-call shim**, because that shim is silently widening the contract in a direction we just rejected.

---

## The compat-call shim — the unforced wart

The thing Round 1 underweighted: `compat_imports.rs` makes bare `math.sqrt(x)`, `heapq.*`, `collections.defaultdict`, and bare `defaultdict`/`deque`/`Counter` *just work* without imports. That's a hidden form of Variant 1, scoped to call expressions, undocumented as policy.

It's worse than explicit bare aliasing because:
- It's invisible at the source level (no import statement to grep for).
- Users learn it works for calls, then are surprised that `from math import sqrt` is rejected — exactly the inconsistency that motivates this whole debate.
- It silently establishes "bare CPython names are fine" for a subset of the stdlib, which leaks the namespace reservation we want to keep for future interop.
- It contradicts the architecture doc the moment we write the `sifr.*`-only rule down.

Two viable answers:

- **(A) Remove it.** Force `from sifr.math import sqrt` and `from sifr.collections import deque`. Demos and stdlib internals get updated; the codemod handles upstream Python during migration. Cleanest.
- **(B) Keep it as a *typed lint* (warn but compile) that the codemod will rewrite.** Pragmatic if removing it churns too many demos/docs in one PR. Must be on a removal clock, otherwise it becomes Variant 4.

Either way, the shim's behavior must be reconciled with the headline policy — leaving it in its current undocumented form is the worst option.

**Recommendation:** ship (A) in the same milestone as the dedicated diagnostic and `sifr.*` documentation, sequenced behind the codemod's first pass over `demos/` so the changes are mechanical, not hand-edited.

---

## Firm recommendation

**Adopt Variant 5 as written policy. Close the compat-call shim. Ship the dedicated diagnostic. Plan the codemod.**

Nothing in Round 2 weakens Round 1's conclusion; the stress-test strengthens it. The best case for bare imports is real but small (adoption friction on the import line) and is better solved by a codemod than by a permanent contract change. The compat-call shim is a silent leak in the same direction we just rejected and should be closed.

---

## Exact policy wording for `internal_docs/architecture.md`

Drop this block under the import-resolution / module-ownership section:

> ### Module namespace ownership (permanent invariant)
>
> Sifr reserves three tiers of import namespace, each with a single owner:
>
> | Prefix          | Owner                          | Resolution                                   |
> | --------------- | ------------------------------ | -------------------------------------------- |
> | `_sifr.*`       | Compiler intrinsics            | Embedded; never resolves to filesystem.      |
> | `sifr.*`        | Sifr standard library          | Embedded registry (`sifr_stdlib::STDLIB_SOURCES`); never resolves to filesystem. |
> | (top-level)     | User code and third-party packages | Filesystem / package-manager resolution.  |
>
> **Sifr stdlib is reachable only through the `sifr.*` prefix.** Bare module names (`math`, `os`, `collections`, etc.) are never aliased to `sifr.*` in any compilation mode, edition, manifest setting, or deprecation track. Code that writes `from math import sqrt` is rejected with `IMPORT_BARE_STDLIB_PREFIX` (see diagnostics), regardless of whether `math` matches an embedded stdlib module name.
>
> **Rationale.**
> - Sifr stdlib diverges from CPython in name (`sifr.math.abs_val`, `sifr.os.remove_file`), error model (`Result`/`Option` instead of exceptions), and ownership semantics. Bare spelling asserts an equivalence that does not hold.
> - The top-level namespace belongs to user code and third-party packages. Aliasing stdlib into it removes ~50 names from the available top-level forever and forces the package manager to disambiguate on every install.
> - If a future CPython interop layer ships (see Phase 37), `from math import sqrt` is the natural spelling for "real CPython's math." Reserving bare top-level keeps that option open at zero cost.
>
> **Type-system support imports.** `typing` and `enum` are bare-named *by exception* and do not contradict the rule above: they emit no Rust, have no embedded stdlib source, and are processed entirely by the frontend's type layer. They are not runtime stdlib and are not aliases for `sifr.typing` / `sifr.enum`. This classification is closed — no other bare-named modules are accepted on these grounds.
>
> **Migration.** Translation of pre-existing Python source into canonical `sifr.*` form is the responsibility of the `sifr migrate` codemod (Phase 37 / interop work), not the compiler. The compiler does not accept Python-shaped imports under any flag or mode.
>
> **Compat-call lowering.** Implicit synthesis of `sifr.*` imports for bare *call expressions* (`math.sqrt(x)`, `defaultdict(...)`, etc.) is **not** an exception to this policy and is being removed (see issue `ad-hoc-stdlib-bare-call-compat-removal`). Such calls will require explicit `from sifr.<module> import <name>` statements once the migration lands.

And in `internal_docs/phases/17_import_and_externals_correctness.md`, reframe the typing/enum carve-out from "bare imports allowed for these" to "type-system support imports, processed before the stdlib resolver."

---

## Action items (ordered)

1. **Dedicated diagnostic.** Replace the generic `unknown import target: 'math'` with `IMPORT_BARE_STDLIB_PREFIX` for any bare module name that matches an entry in `sifr_stdlib::STDLIB_SOURCES` after stripping the `sifr.` prefix. Suggestion text: `use 'from sifr.<module> import …' — Sifr stdlib lives under 'sifr.*' (semantics diverge from CPython; see <link>)`. Touches `crates/sifr_lowering/src/lower/import_diagnostics.rs:14` and the import-handling site that currently routes through `unknown_import_target`.
2. **Policy commit.** Land the architecture-doc block above and the phase-17 reframing.
3. **User-facing doc.** One-page `docs/` entry titled "Why `sifr.*`?" — three-tier model, semantic-honesty rationale, future-interop point. So this debate doesn't recur.
4. **Compat-call shim removal.** Issue + milestone to delete `resolve_python_compat_call_alias` and `resolve_bare_python_compat_call_alias` in `crates/sifr_lowering/src/lower/compat_imports.rs:11-52`. Update demos and stdlib-internal callers to use explicit `from sifr.<module> import <name>`. Sequence behind the codemod's first pass so the demo updates are mechanical.
5. **Codemod scoping.** Add `sifr migrate` to the Phase 37 / interop milestone with explicit scope: (a) rewrite bare stdlib imports → `sifr.*`, (b) flag CPython APIs without Sifr equivalents, (c) surface Result/Option contract differences at call sites it can detect. This is the migration story; aliasing is not.

No backwards-compatibility track, no flag, no deprecation window. The contract is set today.
