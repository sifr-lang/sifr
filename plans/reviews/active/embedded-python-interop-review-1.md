The review file exists but is empty. Here is my review of the phase contract.

# Review: Ad Hoc Embedded Python Interop

## 1. Blocking design issues

### B1. `foreign_blocking` contradicts the already-merged async classification taxonomy
Phase 32.1 sealed the async-effect-and-offload contract using `@blocking_io` and `@cpu_heavy` as the *only* valid annotations to authorize a direct blocking call inside async Sifr (`plans/roadmap.md:59`). This phase introduces a third classification keyword, `foreign_blocking`, used both in the Core Decisions (line 38) and the operation lowering table (line 278). Either:
- `foreign_blocking` is a new synonym, in which case it must be added to the Phase 32.1 taxonomy explicitly (and that's a Phase 32.1 amendment, not self-contained); or
- Python calls reuse `@blocking_io` (the natural fit), in which case all `foreign_blocking` mentions should be deleted.

As written the document is **incoherent with already-merged async semantics**. Pick one and propagate.

### B2. The `py.blocking` vs offload primitive choice is left open
Lines 343–344 say "`py.blocking` or the existing task offload primitive must run Python work in a blocking-safe region." Milestone_py_5 (line 750) repeats the same disjunction. A production-grade design contract cannot ship with "or". The async substrate (Ad Hoc 36.4) and Phase 32.1 already provide the offload primitive — commit to it. Introducing a Python-only `py.blocking` wrapper duplicates effect tracking and creates two places where the rule "Python calls require blocking offload in async" can be expressed and disagree.

Recommendation: drop `py.blocking`, declare that the Sifr runtime treats every `py.*` call as `@blocking_io`, and require user-defined wrappers around `py.*` to inherit/declare `@blocking_io`. State this explicitly in the lowering table.

### B3. `ThreadsafeCallback` semantics undercut the "`py.Object` is not Send" guarantee
- Line 38: `py.Object` is not `Send` by default and cannot cross task/thread boundaries without an explicit audited bridge.
- Lines 492–500: `ThreadsafeCallback` "may be called from Python-created threads, native extension callback threads, thread pools…" — i.e., the underlying Python callable (a `py.Object`) **is** crossing thread boundaries.

The phase never names the audited bridge or describes its safety contract. A `ThreadsafeCallback` is effectively that bridge, but the document does not say:
- whether constructing one requires `Send`-bounded captured Sifr state plus GIL re-acquisition on dispatch,
- what happens when a non-Python thread invokes the callback (does it acquire the GIL and execute Sifr inline? schedule onto a Sifr executor? both?),
- whether the captured `py.Callable` reference itself counts as a `Send` exception or a `ThreadsafeCallback`-internal owner.

This is the single biggest correctness gap. Without it, milestone_py_10 cannot specify what passes.

### B4. Native-extension crash terminates the process — directly contradicts Sifr's "if it compiles, it works" guarantee
Line 523: "Native crashes can terminate the process." This is true but it is a *new* failure mode for Sifr binaries and contradicts the AGENTS.md/CLAUDE.md core guarantee that no user-triggerable runtime panics exist. The phase must explicitly carve out the weakened guarantee:

> Sifr's "no user-triggerable runtime panics" guarantee applies only to Sifr-attributable code paths. Loading a trusted Python native extension delegates process-abort safety to that extension. The guarantee is preserved for the Sifr language surface; it is suspended for the in-process trust boundary that the user opted into via `[trust] python-native`.

Without that carve-out, claims of "production-grade" cannot be made.

### B5. Trust enforcement mechanism is unspecified
The phase introduces `[python] allow-imports`, `[trust] python`, and `[trust] python-native`, plus "Published libraries must not use wildcards" (line 91). It never says:
- Where wildcards are rejected (publish-time CLI? package-graph load? both?).
- How `py.import_module("torch")` is gated against `allow-imports` — at HIR static-analysis time? At runtime?
- What happens for **dynamic** import strings (`py.import_module(name_var)`). Either they are rejected by static analysis, or trust is enforced at runtime — but neither is stated.
- The relationship between `allow-imports` and `trust.python`. Is one a superset, are they orthogonal, must they match?

This is a security surface. It must be specified before milestone_py_1.

### B6. GIL boundary is invisible in the user API
The user API has no `py.with_gil(fn) { … }` block. Every `py.call_attr` etc. presumably acquires and releases the GIL on each call. For numeric/tensor loops this destroys perf (and the document boasts zero-copy for tensors). There must be an explicit GIL-scoped batch primitive:

```sifr
try py.scope(lambda gil: …)   # holds GIL across multiple calls
```

Otherwise zero-copy tensor pipelines pay GIL ping-pong per op.

### B7. Coroutine bridge semantics are unspecified
Line 346: `py.run_coroutine_blocking` "should run the coroutine using Python-owned event-loop mechanics." The contract must answer:
- Does it create a fresh loop per call, or reuse a per-thread loop?
- Is reentry into a running loop (e.g. when invoked from a `ThreadsafeCallback` dispatched on a Python thread that already runs a loop) an error?
- uvloop vs stock asyncio: pinned, auto-detected, or user-configured?
- Is the coroutine guaranteed to run with the GIL held by the Sifr-bound thread, or may it migrate?

Currently every implementer would make a different choice.

### B8. Build/link contract for libpython is missing
The probe records SOABI, extension suffix, libpython path. The phase never states:
- Static or dynamic link to libpython.
- What ABI granularity invalidates the build cache. Minor (3.12→3.13)? Patch? SOABI string change?
- Whether `pyo3`'s `extension-module`/`auto-initialize` features are used, and whether `sifr build` records the chosen interpreter path into the binary (RPATH/RUNPATH or runtime resolution).

This is part of "verifies and consumes" and is implementation-determining. milestone_py_2 cannot start without it.

### B9. Self-containment claim is undercut by "Phase 27 non-regression baseline"
Lines 13 and 842 require Phase 27 to remain green. Phase 27 is completed, so this isn't a *blocker* operationally, but the wording reads as a soft dependency. Tighten it to: "Existing baseline gates remain green." Drop the explicit phase number — the phase contract should not name external phase IDs as preconditions if it claims self-containment.

### B10. The 43.1 numbering implies Phase 43 ordering
`plans/phases/index.md:53` lists this as "43.1 Ad Hoc Embedded Python Interop." The roadmap explicitly lists Phase 43 under *Deferred Planning Drafts (Need Alignment)* (line 121). Numbering 43.1 misrepresents this phase's claim of independence from Phase 42/43. Renumber to a non-43 slot (e.g., the next free 36.x or 37.x) or add a footnote in `plans/phases/index.md` clarifying that 43.1 is sequence-independent.

## 2. Important elegance/coherence improvements

### E1. Object lifetime contract for `BufferView`, `ArrowArray`, `DlpackTensor`
The phase says `py.Object` is non-`Send` by default. It doesn't say what zero-copy view types are. A `BufferView[T]` is a borrow against a Python exporter — its `Send`/`Sync` story matters for whether you can hand a tensor view to a Rayon-style data pipeline. Add an explicit row per view type stating Send/Sync and the rule (probably: non-Send unless the exporter participates in DLPack stream/device sync).

### E2. The conversion table conflates "default conversion" with "explicit-conversion API"
Most rows say "`py.Object` by default; explicit … conversion required." That's a sensible policy but the names of the explicit-conversion functions are scattered across other sections (`zero_copy_as`, `copy_as`, `to[T]`, `to_str`, `to_bytes`). Restructure as two columns: *default Sifr type* and *explicit conversion API*. That makes the contract atomic.

### E3. Tier 1 certification gate is too broad to be a single milestone gate
Tier 1 contains dozens of unrelated production surfaces — Django, FastAPI, SQLAlchemy, psycopg, pymongo, Confluent Kafka, OpenAI/Google clients, cryptography, pandas, pyarrow, opentelemetry. Treating all of these as "the gate" makes milestone_py_11 unbounded. Split Tier 1 into:
- **Tier 1a — interop primitives** (must pass at milestone exit): pydantic, httpx, cryptography, pandas, pyarrow, polars, numpy, torch (CPU), psycopg, sqlalchemy.
- **Tier 1b — ecosystem coverage** (must pass with skip evidence allowed where the host can't host the dep): the rest.

This preserves the comprehensive promise without making the gate impossible to hit deterministically.

### E4. `[python]` `enabled = true` is redundant
The TOML example sets both `enabled = true` and a `venv` path. The presence of `[python]` with `venv`/`interpreter`/`pyproject` should be the activation signal. Drop `enabled`; it invites confusion when libraries declare `requires-imports` against an app with `enabled = false`.

### E5. `py.with` lambda lifetime semantics
The current shape `try py.with(obj, lambda entered: …)` doesn't say:
- Whether `entered` escapes the lambda (must not).
- Whether exceptions thrown inside the lambda are converted via Python `__exit__(exc_type, …)` or just propagate.
- What return type the lambda has — generic `T`?

Specify these explicitly; "context managers" is the most error-prone seam in PyO3 usage.

### E6. `verification/python_interop/runner/run.py` vs `run.sh`
Lines 632, 666–670 reference `runner/run.py` and `run.sh` interchangeably. Pick one as canonical; the other is a thin wrapper.

### E7. Reserve diagnostic family codes now
Phase 31.7 standardized `SIFR-<FAMILY>-dddd`. Reserve `SIFR-PYENV`, `SIFR-PYIMP`, `SIFR-PYCALL`, `SIFR-PYCONV`, `SIFR-PYRES`, `SIFR-PYZC`, `SIFR-PYCB`, `SIFR-PYTRUST` in the design now, so milestones can land diagnostics without re-litigating the taxonomy.

### E8. `[python] interpreter = ".venv/bin/python"` looks like Unix-only
The probe handles Windows (line 102) but the example never shows a Windows interpreter path. Add a one-line note that `interpreter` is platform-conditional or resolved from `venv` by Sifr; otherwise users will hard-code a Unix path.

## 3. Specific wording / structural changes

- **Line 7 (Objective):** Replace "smoothly use Python code" with "call into Python code and Python ecosystem packages."
- **Line 38:** Change `foreign_blocking` → `@blocking_io` (or the chosen final term from B1). Apply globally.
- **Line 278 (lowering table):** Same.
- **Line 343:** Replace "py.blocking or the existing task offload primitive" with the chosen final primitive (B2).
- **Line 92 ("Sifr gates declared/root imports only…"):** Add: "Trust enforcement runs at HIR-static-analysis time over statically discoverable import strings; dynamic import strings are rejected unless the call site is annotated `@trust_python_dynamic`." (Or whichever rule is chosen — but pick one.)
- **Line 132:** Tighten "Treat the live interpreter probe as the source of truth. `uv.lock` digests are cache/diagnostic inputs, not correctness proof that the environment is synced." — good as is, preserve.
- **Line 154:** "Python native packages are trusted process-local code, not sandboxed code." Add a forward link to the carve-out from B4 in the Quality Contract.
- **Line 184 (PyObjectHandle in runtime API):** Add `PyGilGuard` so the GIL-scoped batch primitive (B6) has a representation.
- **Line 286 (conversion table):** Restructure per E2.
- **Section "Quality Contract" (~line 847):** Add the global invariant: "Sifr's no-user-triggerable-runtime-panic guarantee applies to Sifr-attributable paths; trusted Python native extensions are an explicit opt-in trust boundary that may abort the process."
- **Index file `plans/phases/index.md:53`:** Renumber (B10) or annotate independence.

## 4. Already strong — preserve as-is

- "Sifr consumes and verifies the Python environment; Sifr does not resolve, install, or sync Python packages by default" (line 31). Clean ownership boundary with uv.
- "No silent fallback from zero-copy APIs to copying is allowed" (line 43) plus the `zero_copy_as` vs `copy_as` split (lines 379–381). This is the right shape.
- Refusal to support subinterpreters, free-threaded CPython (without future audit), and multiple venvs per process (lines 32–34, 51–52). These are correct stake-in-the-ground decisions.
- `py.Object` opaque, foreign, non-`Any`, fallible operations only (lines 36–37, 215–222). Preserves Sifr's safety surface.
- Conservative conversion policy: no implicit deep conversion of containers/tensors/dataframes (lines 308–309). Right call.
- DLPack one-shot semantics and explicit dtype/device/stream tracking (lines 453–460). Correct.
- Per-resource documented double-close/release decision (line 370). Correct discipline.
- Probe JSON contents and cache-key inclusion (lines 105–131). Solid.
- The split between `LocalCallback` (scoped, may borrow) and `ThreadsafeCallback` (owned/cloned, registry-managed) as *kinds*, not as flags (lines 478–501). The taxonomy is right; only the Send-boundary mechanics (B3) need filling in.
- "Verification area" structure (lines 597–643). The fixture/group decomposition mirrors the design exactly and will translate into a real gate.
- "uv remains the Python package manager; Sifr verifies and consumes the environment" as a Quality Contract invariant (line 854). Keep verbatim.

---

The phase is fundamentally on the right track and the policy decisions are sharper than most ad-hoc phases. The blocking issues are concentrated in three places: the async-classification term mismatch with Phase 32.1, the `ThreadsafeCallback`/`Send` story, and the trust-enforcement mechanism. Once those three are nailed, plus the GIL-scope primitive (B6) and the libpython link contract (B8), this reads as a true design contract rather than a scoped MVP.
