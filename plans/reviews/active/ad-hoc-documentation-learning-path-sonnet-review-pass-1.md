I now have everything I need. Let me write the third-pass review.

---

## Review: ad-hoc-documentation-learning-path (pass 3 — codebase scan)

This pass reads the planning artifact after two rounds of correction and validates it directly against the live codebase: `docs/language/type-system.mdx`, `docs/stdlib/collections.mdx`, `docs/introduction.mdx`, `cli_model_and_entrypoint.rs`, `lib/sifr/`, and `internal_docs/architecture.md`.

---

### Summary of the issue state

Pass-2's three conditional blockers are now mostly resolved. Finding 2 (Language Tour section) is gone — the issue no longer proposes `language/tour.mdx`. Finding 3 (the self-contradicting `<Note>` worked example) is fixed — the example now demonstrates clean, direct copy. Finding 1 (Slice 0 doesn't pick a side on dict indexing) is partially addressed but has a more precise resolution available from the architecture source, described below.

---

### Findings

**1. Slice 0 has a false premise: the architecture already picks a side on dict indexing. — blocking**

The planning artifact frames Slice 0 as "resolve the contradiction — either A or B." But `internal_docs/architecture.md` is unambiguous:

> "Where CPython raises `KeyError`, Sifr returns `Option[V]`. Example: `dict["missing"]` raises `KeyError` in CPython; in Sifr it returns `None`."

The confirmed live contradiction is that `docs/stdlib/collections.mdx:46` says "Direct `scores["missing"]` is a compile-time checked operation that produces a typed error if the key might be absent" — which contradicts the architecture, contradicts `introduction.mdx:35`, and contradicts `type-system.mdx:30`. There is no genuine ambiguity to resolve; there is a wrong page to correct.

Slice 0's framing should change from "decide and record which contract applies" to "correct `collections.mdx:46` to match the architecture contract (`[]` returns `T | None`), and confirm the corrected statement against a pinned demo or test." Leaving it as "either A or B" gives the implementer permission to go either way, which could produce a Slice 1 where `from-python.mdx` is drafted against the wrong baseline.

**2. `type-system.mdx` describes `int` as `64-bit signed integer` — blocking**

Line 147 in the live file reads: `| int | 64-bit signed integer |`. The architecture is explicit: `int` is exact, arbitrary-precision, backed by inline-small `SifrInt`. This description will directly mislead Python developers who encounter overflow behavior different from what the docs promised.

The Slice 0 acceptance criterion says "Confirm whether current docs still describe `int` as 64-bit anywhere." This scan confirms they do. The planning artifact should name this file and line explicitly so Slice 0 doesn't treat it as a question to investigate but as a specific edit to make.

**3. `type-system.mdx` "None Safety" examples still use `int | None` without a None check — blocking**

Lines 88–98 (live in the file) show:

```python
def find_value(x: int | None, target: int) -> str:
    if x == target:   # x is int | None, no None check
        return "found"

def is_positive(x: int | None) -> bool:
    if x > 0:         # same
```

If the compiler rejects these (as the safety guarantee implies it should), the examples are wrong. If the compiler accepts them via implicit narrowing on equality/comparison, that behavior needs to be documented explicitly, not left implicit. This was flagged in pass 1; it remains live. Slice 0 lists "Audit `int | None` examples" but does not name this specific location. Name it.

**4. `type-system.mdx` claims `int`, `float`, `bool` are Copy types — important**

Line 154 (live): "Primitive types (`int`, `float`, `bool`) are **Copy** types." The architecture is explicit: "`int` is not Rust `Copy`, but codegen owns the borrow/clone/primitive-local optimization." `float` and `bool` are Copy; source-level `int` is value-semantic at the source level but not Rust-`Copy` in codegen. This matters for the `from-python.mdx` borrow row and for any Rust-curious user who reads the emitted Rust. This is Slice 0 audit scope, but the specific sentence is not named.

**5. `parallel` module is missing from the stdlib coverage plan — important**

`lib/sifr/parallel.sifr` is a shipped module. The planning artifact's stdlib scan lists `sifr.task` and `sifr.sync` under concurrency, but never mentions `sifr.parallel`. The architecture and the structured runtime work model both reference `sifr.parallel` as a first-class concurrency surface. The Slice 3 module index will produce a gap if this module is absent from the tracking list. Add it to the "Concurrency" category in the stdlib scan section alongside `task`, `sync`, and `runtime`.

**6. `logging`, `argparse`, and `timeit` are absent from the stdlib scan — important**

`lib/sifr/` contains `logging.sifr`, `argparse.sifr`, and `timeit.sifr`. None of these appear in the planning artifact's module inventory section. `logging` and `argparse` are modules every serious Python developer reaches for immediately. Their absence from the tracked module surface risks the Slice 3 module index omitting them. They belong in a "Developer tooling" or "System utilities" category in the scan section.

**7. CLI inventory plan omits `--explain` and `--diagnostic-format` global flags — optional**

From the CLI source, `--explain <CODE>` and `--diagnostic-format human|json|compact` are global flags on the `sifr` command itself (not subcommands). `--explain` provides inline diagnostic lookup from the terminal — exactly the kind of developer ergonomic detail that a "Tooling, CI, and Production User" persona needs. Slice 4 focuses on subcommands but the global flags are undocumented and not tracked. A single sentence in `cli/overview.mdx` and a note in the Slice 4 acceptance criteria would close this.

**8. Internal `.md` files in `docs/` are not acknowledged — optional**

`docs/` contains eight `.md` files outside the Mintlify navigation: `concurrency_runtime.md`, `formatter.md`, `network_http.md`, `package_management.md`, `self_update.md`, `stdlib_imports.md`, `text_i18n.md`, `cli_command_semantics.md`. These appear to be internal design references. They are not in `docs.json` and will not be published. The planning artifact does not acknowledge them, so a future implementer adding a page with a conflicting name might be confused by their presence. A one-line note in the Non-Goals or in Slice 1 acceptance — "existing internal `.md` files in `docs/` are not part of the Mintlify nav and are not published" — would prevent confusion.

---

### Copywriting and style assessment

The Content Principles section (10 rules) is the right scope and tone, but two are weak:

- Rule 2 ("Use comparison tables only when they reduce surprise") is self-referential. A writer cannot apply it without first knowing what counts as surprising. Rewrite as: "Use comparison tables where Python developers would otherwise infer the wrong thing — not for every difference, and not to be exhaustive." This gives implementers a judgment rule.

- Rule 9 ("Avoid exhaustive spec language in first-pass learning pages") doesn't say what to do with detail that doesn't fit — defer to accordions, deep reference pages, diagnostics docs? The adjacent rules don't answer this. Add a one-sentence pointer: "Move spec-level detail to diagnostics, CLI references, or architecture docs; a learning page links to those rather than inlining them."

The `From Python` table's "Bytes" row ("First-class `bytes` | Binary/text boundaries are explicit and typed") is the weakest cell. The other rows name a concrete behavioral change; this one names a property. Better contrast: `bytes` in Python is sequence-of-int but often treated as opaque; in Sifr, encode/decode boundaries are typed operations that produce explicit errors — there is no platform-dependent default encoding. That fits on one line and tells a Python developer something actionable.

---

### Codebase coverage verdict

The four new audience personas and the six documentation gap sections collectively cover the important developer surfaces. After this scan, the surfaces that remain undiscovered in the plan are:

- `sifr.parallel` (important, missing entirely)
- `logging`, `argparse`, `timeit` (important for everyday use, missing from module inventory)
- `--explain` and `--diagnostic-format` global flags (optional, useful for tooling users)

The structural split (Get Started → Learn Sifr → Standard Library → CLI Reference → Packages → Diagnostics → Project) is correct and the Slice ordering (0 → 1 → 2 → 3 → 4) is sound.

---

### Go / no-go

**Conditional go.** The planning artifact is ready to execute with three targeted edits before Slice 0 starts:

1. Change Slice 0's dict-indexing framing from "decide between A or B" to "correct `collections.mdx:46` to match the architecture contract already established in `internal_docs/architecture.md`."
2. Name `type-system.mdx:147` and `type-system.mdx:154` as specific confirmed edits in the Slice 0 checklist, not as "check whether these need fixing."
3. Add `sifr.parallel`, `logging`, `argparse`, and `timeit` to the stdlib scan inventory under appropriate category headings.

Findings 7 and 8 can be picked up during Slice 1 and Slice 4 drafting without blocking adoption of the plan.
