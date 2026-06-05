I've verified the revised plan against the codebase. The blocking findings from pass 1 are addressed (framing corrected, both `Stmt::Import` and `Stmt::ImportFrom` covered, `SIFR-IMPORT-0008`/`IMPORT_BARE_STDLIB` is the next free code, stdlib tails derived from `STDLIB_SOURCES`, M2/M3 split is now explicit, synthetic-imports consumer removal called out at `mod_impl.rs:647`, `__compat_defaultdict_*` rename folded into M3). But re-reading M4 against the code surfaces a new unreachable guardrail that I'm calling out as the remaining blocker.

# Planning Review — Stdlib Namespace Contract — Pass 2

## Verdict: `CHANGES_REQUESTED`

The plan is close to implementation-ready and most pass-1 findings landed cleanly. One concrete blocker remains in M4, plus a couple of small tightenings that affect implementability.

---

## Blocking findings

### B1. M4 guardrail `rg "__compat_sifr_" ... -g '*.rs'` is unreachable

Per locked decision 8, async/task compatibility for `sifr.sync` / `sifr.concurrent` / `sifr.asyncio` / `sifr.task` is **explicitly out of scope**. Current code retains these consumers:

- `crates/sifr_lowering/src/lower/task_scope_calls.rs:391` — `name.strip_prefix("__compat_sifr_sync_")`
- `crates/sifr_lowering/src/lower/blocking_executor_calls.rs:121` — `name.strip_prefix("__compat_sifr_concurrent_")`
- `crates/sifr_codegen/src/lower_expr/leaves_and_plain_calls.rs:48` — `is_compat_stdlib_alias` uses `starts_with("__compat_sifr_")` as a generic prefix check

These will continue to match the M4 guardrail forever, so the bullet "returns no production hits" is unsatisfiable as written.

**Fix:** Either narrow the regex to the namespaces this phase actually eliminates, or pin an explicit exception list. Recommended:

```text
rg "__compat_sifr_(math|heapq|collections)_" crates/sifr_lowering/src crates/sifr_codegen/src -g '*.rs'
```

…and add a one-line note in M4 / decision 8 that `__compat_sifr_sync_`, `__compat_sifr_concurrent_`, and the generic `is_compat_stdlib_alias` prefix check are intentionally retained because the async/task compat scheme is out of scope, and will be revisited in a future async cleanup phase.

### B2. M2 codegen-removal bullet doesn't cover the Rust unit tests that hard-code compat names

The bullet "Remove codegen canonicalization for `__compat_sifr_math_*`, `__compat_sifr_heapq_*`, and `__compat_sifr_collections_*`" addresses the production strip-prefix sites, but the following Rust tests literally construct or assert on those names and will fail to compile / fail at runtime once the prefix is gone:

- `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs:281,305` — builds a `HirCall { func: "__compat_sifr_heapq_heapify", ... }` and asserts the alias appears in emitted output
- `crates/sifr_codegen/src/intrinsic_method_emitters/narrowing_helpers.rs:66,71` — asserts canonicalization of `__compat_sifr_math_fmod` / `__compat_sifr_heapq_heappush`
- `crates/sifr_codegen/src/lower_expr/leaves_and_compound_tests.rs:162` — string literal `"__compat_sifr_math_fmod"` in a test

The plan currently lumps these under "focused lowering/codegen tests touched by this milestone." Without explicit scope, an implementer might attempt M2 expecting the production deletion alone and discover four test-module breakages PR-time.

**Fix:** Add a bullet under M2: "Remove or rewrite the Rust unit tests in `sifr_codegen` that construct or assert on `__compat_sifr_math_*` / `__compat_sifr_heapq_*` / `__compat_sifr_collections_*` aliases — they lose meaning once the compat path is deleted." This is a routine cleanup but it should be in-scope explicitly so the M2 PR diff is anticipated.

---

## Non-blocking but worth tightening

### N1. Disposition of the generic `is_compat_stdlib_alias` fast-path skip

`crates/sifr_codegen/src/lower_expr/leaves_and_plain_calls.rs:47-48` and its caller at `:633` skip the fast plain-call path for any name starting with `__compat_sifr_`. After M2/M3, the only producers of that prefix in the codebase are the async/task surfaces (which never reach this codegen path with bare names anyway). The helper effectively becomes dead defensive scaffolding.

The plan doesn't take a position on this. It's fine to leave it (it's harmless and decision 8 keeps async/task off-table), but a one-sentence note in M2 — "the generic `is_compat_stdlib_alias` codegen guard remains, scoped to retained async/task aliases" — would prevent an implementer from rethinking it during the PR.

### N2. M2 fixture-update bullet is still file-list-free

Pass 1's finding 13 noted ~20 e2e fixtures touch the patterns this phase removes. The revised M2 keeps the bullet as "Update `.sifr` fixtures and demos that rely on …" without enumerating. Not a blocker — `rg` can find them — but worth saying "the implementer is expected to grep `deque(`, `Counter(`, `math\.`, `heapq\.` under `crates/sifr/tests/e2e/pass/` and `demos/` and update each one explicitly" so this doesn't become a hidden scope expander.

### N3. Verify behavior between M2 and M3 PRs for `collections.defaultdict(list)`

The plan correctly says M2 leaves `collections.defaultdict(...)` and bare `defaultdict(...)` working, and M3 removes both. But after M2 deletes the `collections.*` non-defaultdict synthesis (lines 24-31 of `compat_imports.rs`), the `collections.defaultdict` short-circuit at lines 21-23 of the same file becomes unreachable code if the entire `resolve_python_compat_call_alias` body collapses to just the early return. Either:
- Keep the early-return form intact in M2 and delete the rest of the function body (leaving an obviously-incomplete function), or
- Restructure so the defaultdict short-circuit moves to its own helper before M2 starts deleting.

The implementer can figure this out, but the plan should pick one to avoid a half-deleted function in an intermediate PR.

---

## What's fixed since pass 1

For the record: framing corrected at line 9; `Stmt::Import` + `Stmt::ImportFrom` both covered at line 106; resolution order pinned at lines 76 + 84; diagnostic code `SIFR-IMPORT-0008` / `IMPORT_BARE_STDLIB` pinned (verified next-free in `crates/sifr_diagnostics/src/codes/registry.rs:28-34`); machine-readable fields `bare_module` / `suggested_module` / `imported_names` pinned at line 113; M2/M3 split for defaultdict pinned at line 147; synthetic-imports consumer removal explicit at line 169; `__compat_defaultdict_*` → `__sifr_defaultdict_*` rename folded into M3 at line 186 with the secondary grep extended to `sifr_type_system` at line 206; factory whitelist + `defaultdict as defaultdict` pinned in the Defaultdict Contract; `typing` / `enum` exemption documented at line 80.

---

## Summary

Two concrete blockers remain: the M4 guardrail regex needs to be scoped to `math|heapq|collections` (B1), and the M2 scope needs to explicitly cover the Rust unit tests that hard-code compat names (B2). Once those are in, plus the three small tightenings (N1–N3), the plan is implementable with low ambiguity.
