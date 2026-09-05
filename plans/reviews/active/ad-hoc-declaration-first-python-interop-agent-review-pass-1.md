I've reviewed the three changed files against the implemented Python interop contract, the Rust interop reference architecture, and the current stdlib/runtime. Below is the review.

---

# Review: Declaration-First Python Interop Architecture

## Verdict

**Approve with changes.**

The design cleanly parallels Rust interop's package-author/consumer split, preserves the implemented safety contracts (non-Send objects, blocking-io effect, no silent zero-copy fallback, structured `PythonError`), and correctly identifies raw handles as the escape hatch rather than the primary API. The declaration grammar is minimal and signature-authoritative, which is the right call.

Several specific contracts are underspecified or ambiguous, and two milestones bundle several independent PRs' worth of work. These should be closed or split before M1 begins; none require a fundamental redesign.

---

## Blocking findings

### B1. Package-local Python bridge packaging/hermeticity is a placeholder

**Files/sections**: `internal_docs/python_interop_declaration_architecture.md`, "Package-Local Python Bridges" (lines 178–225); `plans/issues/active/ad-hoc-declaration-first-python-interop.md`, M3 (lines 168–197).

The doc gives a bulleted list of *what* the bridge system must do (embed, namespace, cache) but does not name a *mechanism*. In contrast, the Rust interop doc pins exact paths (`src/bridges/*.rs`, `crate::__sifr_bridge`), file ownership rules, and archive contents (`internal_docs/rust_interop_architecture.md:254–314`). Without an equivalent for Python bridges, none of the following are answerable:

- What module name does `bridge.identifiers.parse_gtin` register under at runtime? A reserved prefix such as `__sifr_bridges.<package_id>.identifiers`? The doc says "reserved package-specific module namespace" but does not name the namespace.
- How are bridge source bytes carried into the final binary — embedded as a resource + custom `importlib.abc.Loader`, or extracted to a per-process temp path? This has real deployment implications (single-file binaries, container FS constraints, PyInstaller-style extraction races).
- How is `sys.path` shadowing actually prevented if the Python interpreter is already initialized before bridge registration?
- What happens when two Sifr packages ship a bridge with the same last segment (e.g., both define `bridge.identifiers`)? Namespaces per package or global?

**Correction**: Add a "Bridge Module Registration" subsection modeled on Rust's `src/bridges/` + `__sifr_bridge` design. Fix the reserved importable namespace (proposal: `__sifr_bridges.<package_module_path>.<file_stem>`), state the loader mechanism (recommended: an in-process `MetaPathFinder` reading from an embedded `HashMap<str, str>` of `(module_name, source)` captured in generated runtime metadata), and specify that plain-file resolution by `sys.path` for those reserved names is a `SIFR-PYIMP-*` diagnostic. Include one sentence in the archive contents section pinning the bridge source list.

### B2. `@python.attr` and `@python.item` grammar is incomplete

**File/section**: `internal_docs/python_interop_declaration_architecture.md`, "Minimal Decorator Grammar" (lines 106–124) and the `Bic` example (lines 83–95).

- `@python.attr(Self.country_code)` uses structured-path syntax for what is fundamentally a string attribute name. The doc says "attribute access remains fallible even when it resembles a field," but does not say what the compiler does when the attribute is not statically introspectable (descriptors, `__getattr__`, C-extension classes with no `.pyi`, dynamically added attributes). "Stub-Assisted Authoring" says runtime introspection *may* confirm but cannot be required — so what is the checking behavior for an unresolvable `Self.foo`? Accept + defer to runtime? Reject? Warn?
- `@python.item` is listed without any example. There is no rule for how it binds to a Sifr `def __getitem__(self, key: K) -> Result[V, PythonError]: ...` signature: does the declared parameter list drive positional/keyword? Is the `key` type in the Sifr signature the sole conversion contract, or is there a decorator arg?

**Correction**: Add a canonical example for `@python.item` (e.g., binding to `object[key]` with the receiver's `__getitem__` and a specific Sifr signature). For `@python.attr`, define exactly one resolution policy: static existence check *if* the target is introspectable (has `.pyi`, `py.typed`, or a Python-level attribute of the correct kind), otherwise emit a `SIFR-PYCALL-*` "target not statically verifiable" diagnostic that is downgraded only when the class is annotated `@python.opaque(introspectable=False)` (or similar named opt-out). No silent fallback.

### B3. "No-panic deferred release path" for owned Python handles is undefined

**File/section**: `internal_docs/python_interop_declaration_architecture.md`, "Ownership And Cleanup" (lines 152–176).

The doc says handles drop "while attached to the interpreter, or uses a no-panic deferred release path owned by the runtime." Sifr's core guarantee is "no user-triggerable runtime panics." Without a defined queue/drain policy, this claim is not verifiable. The current runtime (`crates/sifr_runtime/src/python/object_ops.rs`) uses an `OBJECT_STORE` with explicit handles, not PyO3 `Py<T>` in the user-facing surface; migrating to sealed handles will introduce new Drop-context possibilities:

- Drop called from a thread that never held the GIL.
- Drop during panic unwind (though Sifr disallows those in user code, PyO3 internals may still call it).
- Process shutdown / thread exit while refs are outstanding.
- Reentrant drop while executing a Python callback.

**Correction**: Add a "Deferred Release Queue" paragraph that names the mechanism (e.g., "when Drop runs without the GIL, the reference is appended to a lock-free MPSC queue that is drained by the next GIL-holding operation and by an explicit `sifr_runtime::python::drain_pending_releases()` shutdown hook") and states what happens if the queue is drained after the interpreter is torn down (the current spec says `Py_FinalizeEx` is not called during normal shutdown, so the queue is naturally quiescent — say so). This is prerequisite to shipping sealed handles.

### B4. Closed record → Python mapping contradicts the existing `py_from_record` intrinsic

**File/section**: `internal_docs/python_interop_declaration_architecture.md`, "Conversion Contract" (lines 126–150).

The table says a closed record becomes "String-keyed Python mapping with exactly the declared fields and recursively converted values." That reads as `dict[str, ...]`. But the current runtime already distinguishes `py_from_dict_str` from `py_from_record` (see `stdlib/sifr/python.sifr:678–699`), which implies records are *not* just dicts — otherwise the two intrinsics would collapse.

If the intent is "records become plain dicts," M3 will need to delete `py_from_record`/`py_copy_record_fields`, and libraries expecting an object with attributes (Pydantic models, dataclasses, TypedDicts, SQLAlchemy row shapes) will silently receive a dict. If the intent is "records become a distinct kind" (namedtuple, dataclass instance, `types.SimpleNamespace`, or a bridge-owned struct), the doc must say which.

**Correction**: Pick one of {plain dict, `SimpleNamespace`, bridge-owned struct proxy}, spell out the reverse direction (Python object → Sifr record: `getattr` vs `__getitem__` vs both), and reconcile with the existing `py_from_record` intrinsic — either keep it and describe how the declaration path uses it, or delete it in M3 and explain in the phase doc.

### B5. Removing `allow-imports` conflates two intentionally-separate policies

**File/section**: `internal_docs/python_interop_declaration_architecture.md`, "Environment And Trust" (lines 226–250); `plans/issues/active/ad-hoc-declaration-first-python-interop.md`, M4 (lines 199–234).

Current policy (`docs/python-interop.mdx:39–52`) explicitly distinguishes:

- `[python].allow-imports` — roots Sifr *source* may import/call.
- `[python].requires-imports` — roots a *library* requires the app to provide.
- `[trust].python` — roots authorized to *execute* Python code in process.
- `[trust].python-native` — roots authorized to load native extensions.

The declaration doc says "the source/package graph states what is required, while `[trust].python` states what the root author authorizes." That covers `requires-imports` and `[trust].python` but not the source-gate role of `allow-imports`. Today, a library can `import pandas` at source but only a root that lists `pandas` in `allow-imports` can compile against that library — this is a defense-in-depth check independent of runtime trust.

Under the new model, is that source gate gone? If so, say so and justify: "static imports are already visible in the package graph, and `[trust].python` gates all execution, so the second source-level allowlist adds nothing." If not, say what replaces `allow-imports` for the source gate. Also state whether `[python].requires-imports` remains in the manifest (I read the doc as "yes, libraries still declare requirements; only root `allow-imports` goes away" but this is ambiguous).

**Correction**: In the "Environment And Trust" section, keep a one-paragraph explicit table showing which of the four current keys survive (my read: `requires-imports`, `[trust].python`, `[trust].python-native`; drop `allow-imports`). State the reasoning ("execution trust subsumes the source allowlist because static imports are visible in the package graph and cannot execute without `[trust].python`"). Ensure M4 tasks mirror that.

### B6. Milestones M3 and M4 bundle several independent PRs

**File/section**: `plans/issues/active/ad-hoc-declaration-first-python-interop.md`, M3 (lines 168–197), M4 (lines 199–234).

M3 combines: recursive container/record conversion IR, hermetic bridge module packaging (a substantial subsystem — see B1), cache-key work for bridge/distribution drift, *and* migration of the dataframe/ML/web/database/cloud examples off raw handles. Any one of those is a reviewable PR on its own; together they are neither reviewable nor bisectable.

M4 combines: uv discovery + lock consistency check, removal of `allow-imports` from the manifest, `sifr python check`, `sifr python doctor`, `sifr python bind`, and LSP completion/navigation/diagnostics. Same problem.

The predecessor phase (`plans/issues/active/ad-hoc-embedded-python-interop.md`) covered a comparable surface in 13 milestones. Compressing to 6 without any sub-milestone breakdown contradicts the "small, reviewable PRs" instruction in `AGENTS.md`.

**Correction**: Split M3 into M3a (typed conversion), M3b (bridge packaging + registration), M3c (ecosystem example migration + zero-leak assertions). Split M4 into M4a (uv discovery + manifest cleanup), M4b (`check` + `doctor`), M4c (`bind` scaffold), M4d (LSP surfaces). Each sub-milestone should map to one PR with its own acceptance/validation block.

### B7. Decorator target root resolution is unstated

**File/section**: `internal_docs/python_interop_declaration_architecture.md`, "Declaration Syntax" (lines 62–124).

The doc says allowed roots are "a statically declared Python import root", `bridge`, and `Self`. It does not say *where* Python roots are declared. Rust interop uses Cargo dependency names as roots (a Cargo.toml lookup); the Python equivalent has three plausible sources: (1) explicit `[python].imports` in the manifest, (2) uv-installed packages in the resolved lock, (3) `import` statements at the top of the Sifr source. Different choices give different diagnostics for typos, and different behaviors for libraries transitively depending on other libraries.

Also unaddressed: if `allow-imports` is gone (B5), what makes `math` a valid root in `@python(math.sqrt)`? The declaration itself? Then the root is inferred, which is fine — but say so.

**Correction**: Add one paragraph to the "Declaration Syntax" section: "A dotted path `pkg.a.b` in `@python(...)`, `@python.opaque(type=...)`, or `@python.attr(...)` is valid when `pkg` is either the top of a `bridge.*` or `Self.*` reserved root, or a resolvable module in the root-selected uv environment. Roots discovered by decorators are added automatically to the package's required import list and gated by `[trust].python` (native use is separately gated by `[trust].python-native`). No manifest declaration is needed; the source is the declaration."

---

## Non-blocking refinements

- **N1** (`declaration_architecture.md`, lines 105–124): Add one paragraph comparing the Python opaque grammar to Rust's, so a reader familiar with `@rust.opaque` understands what is intentionally absent. `sync=` is unnecessary because the GIL is the sync primitive; `clone=` is unnecessary because Python objects have no structural copy; `borrow=` is unnecessary because all receivers are `&PyRef`-shaped; `thread_affinity=` is redundant with non-Send by default plus the single-interpreter rule. This preempts the natural question.

- **N2** (`declaration_architecture.md`, "Blocking And Async"): State explicitly "There are no `@python.async` declarations in v1 because every Python call is blocking from Sifr's perspective." That fact is currently only inferable from a future-tense sentence.

- **N3** (`declaration_architecture.md`, "Conversion Contract" table): Add "Python side sees `X`" column so record vs dict vs opaque is unambiguous (see B4).

- **N4** (`declaration_architecture.md`, "Verification Contract" line 340): rename `supported-through-python-bridge` to `supported-through-bridge` to match Rust interop's category name — the "python" qualifier is redundant inside a Python interop area.

- **N5** (`declaration_architecture.md`, "Durable Decision Summary"): This 12-line list duplicates most of `plans/.../ad-hoc-declaration-first-python-interop.md` "Core Decisions". Keep the plan doc's list authoritative and let the architecture doc end at "Verification Contract" — or vice versa. The current state invites drift.

- **N6** (`declaration_architecture.md`, "Package-Local Python Bridges" example, lines 188–198): The Python bridge's `dict[str, object]` return type is unenforced advice; the Sifr `GtinInfo` record is the contract. One sentence reminding readers of that will prevent authors from over-annotating Python bridges and expecting Sifr-side enforcement of Python hints.

- **N7** (`declaration_architecture.md`, "Two-Level User Model" and "Declaration Syntax"): These two sections back-to-back repeat the split between typed API and escape hatch. Merge into one section for tighter prose.

- **N8** (`declaration_architecture.md`, line 137): "Explicit owned construction on input and checked copy on output" for containers is prose-y compared to the table row for scalars. Consider a small sub-table or a footnote naming the runtime intrinsics used (e.g., `py_from_list`, `py_copy_list_*`) so the compiler team has a concrete lowering handoff.

- **N9** (phase doc, line 33): "Ordinary reference drop is distinct from semantic `close`, `aclose`, context-manager exit, buffer release, capsule release, and callback shutdown." Excellent point; consider promoting to the architecture doc's "Ownership And Cleanup" section as an explicit list — it currently only appears as prose there.

- **N10** (phase doc, "Verification Policy" line 275): "Live service evidence must distinguish actual compiled Sifr execution from Python-client execution or source-presence checks." This is a direct improvement over the current live gate (which treats source-checked callback + Python-client execution as evidence — see `verification/areas/python_interop/README.md:44–48`). Cross-link this into M5 so it becomes a hard acceptance criterion rather than a policy sentence.

---

## Proposed final architecture summary (to check the docs communicate this)

Sifr Python interop has two levels. Package authors write **ellipsis-only Sifr declarations decorated with `@python`**; the Sifr signature is the single conversion contract. Consumers call those declarations as ordinary typed Sifr functions and never touch handles. `sifr.python` remains the raw dynamic escape hatch over the same sealed handle representation.

Decorator targets are structured dotted paths — never strings — resolved against three reserved roots: static Python imports (inferred from decorators, gated by `[trust].python`), `bridge.*` (hermetic package-local `.py` files carried in the compiled binary under a reserved importable namespace), and `Self` (methods of a `@python.opaque` class). Grammar is deliberately minimal: `@python`, `@python.opaque(type=..., close=..., send=...)`, `@python.attr`, `@python.item`. Callbacks, context managers, async, and advanced-data protocols keep their existing explicit contracts and are not folded into the initial decorator grammar.

Owned Python references drop automatically via a compiler-owned sealed handle backed by a defined no-panic release queue; `close`/`aclose`/context exit/buffer release/capsule release/callback shutdown remain semantic operations distinct from drop. Every boundary operation is fallible and returns `PythonError`; exceptions never unwind. Every `@python` call carries `blocking_io` implicitly; non-Send Python values cannot cross task/thread boundaries.

Conversion covers scalars, `Option`, `list[T]`, `tuple[...]`, `dict[str, T]`, closed records, opaque classes, and `py.Object`. Any/`Object` fallback, unbounded generics, unconstrained unions, iterators, uncontracted callables are declaration-time diagnostics. Container/record returns imply checked copies; zero-copy always requires an explicit contract.

Root applications continue to own one uv-created CPython env; Sifr verifies and never mutates it. Import requirements are inferred from decorators + bridge sources + library `requires-imports`; the redundant root `allow-imports` list is removed. Trust remains explicit through `[trust].python` and `[trust].python-native`.

Compatibility categories are `supported`, `supported-through-bridge`, `dynamic-only`, `unsupported-by-design`, `future`. Support requires executable positive *and* negative evidence produced by an actual compiled Sifr binary; matrix inventory alone does not certify declaration behavior. Migration migrates the existing runnable ecosystem examples to declaration-first calls and asserts zero outstanding ordinary Python references at every exit path.

If the three documents read this back cleanly after B1–B7 are closed, they communicate the intended design.
