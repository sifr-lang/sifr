I've completed the review. Let me now write the response summarizing the findings.

# M6 Milestone Closure Review — Round 4

## Round-3 Observations, Now Verified As Fixed

**doc-drift (round-3 obs #1) — fixed.** `internal_docs/python_interop_declaration_architecture.md:400` now reads `SIFR-PYIMP-0001` and matches `PYIMP_INVALID_TARGET` at `crates/sifr_diagnostics/src/codes/python_interop_codes.rs:17` and the emission site at `crates/sifr_lowering/src/lower/python_interop.rs:590`. All other PYRES-0002 references in the tree are either the code's own registry/doc rows or historical review artifacts — none stale.

**dynamic-import-detection-limit (round-3 obs #2) — fixed.** `crates/sifr_package/src/python/bridge_inventory/imports.rs:130–150`:

- `record_imported_dynamic_aliases` now accepts `name == expected || name == "__import__" || name == "*"`. So `from importlib import __import__ as load` inserts `load`; star imports insert the module's dynamic function name (`import_module` for `importlib`, `__import__` for `builtins`).
- `record_assignment` at :172 adds an explicit `value_name == "__import__"` check, so `load = __import__` propagates the alias during the `ImportCollector` pass — closing the ordering bug where `DynamicImportVisitor::new`'s unconditional `__import__` seed only ran after assignments were visited.
- `dynamic_callable_name`/`record_assignment` at :175–178 and :224–226 now also match `member == "__import__" && importlib_aliases.contains(prefix)`, so `importlib.__import__` and `getattr(importlib, '__import__')` are rejected symmetrically with the `builtins` forms.

**Regression coverage — present.** `crates/sifr_package/src/python/bridge_inventory_tests.rs:111–135` adds `importlib_dunder`, `importlib_dunder_alias`, `builtin_assignment`, `importlib_star`, and `builtins_star` cases. I traced each against the collector/visitor and each correctly produces a diagnostic with the expected substring in `diagnostics[0].message`.

## Independently Re-Verified This Round

- **Correctness of the `matches!` guard** at imports.rs:136. The bound name `name` captures `alias.name.as_str()`; the guard evaluates `name == expected || name == "__import__" || name == "*"`. For the star branch the insertion is `expected.to_string()` (`import_module` or `__import__`), which is exactly the local binding a `from <mod> import *` produces. No unbound-name or shadowing issue.
- **No false positives regressed.** `from importlib import UNKNOWN`, `from builtins import import_module` (non-existent), and `__import__ = harmless` all remain no-ops relative to `dynamic_function_aliases`. The unconditional `"__import__"` seed in `DynamicImportVisitor::new` is defense-in-depth; the additional `record_assignment` seed does not make any *new* name a call site — only propagates aliases.
- **No API-shape drift.** No other file passes `discover_source_paths` or otherwise depends on the pre-fix signature.
- **File-size guardrail intact.** Largest touched first-party file remains `python_interop.rs` at 839 lines; `bridge_inventory/imports.rs` grew to 268, `bridge_loader.rs` to 398, archive tests to 230. All well under 900.
- **All round-3 verified items still hold** in the current tree (bridge loader ordering, cache identity, symlink-ancestor rejection, `reset_for_tests` cleanup, capability matrix honesty, biip archive proof, PYIMP/PYCALL/PYCONV/PYCTX taxonomy).

## Non-Blocking Observations (LOW / INFO — do not gate closure)

1. **round-4 review file is a 0-byte placeholder** at `plans/reviews/active/ad-hoc-python-interop-m6-milestone-closure-review-round4.md`. Expected — this review will populate it.
2. **plan-link-format-nit** (carried over from round-3 obs #3, `plans/issues/active/ad-hoc-declaration-first-python-interop.md:139`): the M6 row omits a top-level PR link because this branch *is* the closure PR; the wave breakdown at :375–422 links each sub-PR. Add the post-merge link when this branch lands.

## Verdict

Both round-3 non-blocking observations are now closed with matching regression tests. No new issues introduced. The M6 closure — bridge inventory, resolution, embedding, loader, archive proof, two-package isolation, cache identity, diagnostic taxonomy, evidence, capability matrix, docs, and roadmap — remains sound.

SATISFIED
