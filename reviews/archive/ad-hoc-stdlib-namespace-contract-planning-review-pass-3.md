I've verified the plan against the codebase. All Pass 2 blocking findings and non-blocking tightenings are now reflected.

# Planning Review — Stdlib Namespace Contract — Pass 3

## Verdict: `READY`

## Pass 2 follow-ups verified

- **M4 guardrail narrowed to `math|heapq|collections`** — `issue.md:209` now uses `rg "__compat_sifr_(math|heapq|collections)_" crates/sifr_lowering/src crates/sifr_codegen/src -g '*.rs'`. The retained async/task aliases (`task_scope_calls.rs:391` `strip_prefix("__compat_sifr_sync_")`, `blocking_executor_calls.rs:121` `strip_prefix("__compat_sifr_concurrent_")`) and the generic `is_compat_stdlib_alias` (`leaves_and_plain_calls.rs:47-49`) are no longer caught by this regex.
- **M2 Rust unit test cleanup in scope** — `issue.md:172` explicitly requires removing/rewriting the tests that hard-code `__compat_sifr_math_*`/`__compat_sifr_heapq_*`/`__compat_sifr_collections_*`. Verified the impacted call sites exist at `narrowing_helpers.rs:66,71`, `leaves_and_compound_tests.rs:162`, and `collections_and_stdlib_codegen_tests.rs:281,305`.
- **Generic `is_compat_stdlib_alias` retention noted** — `issue.md:173` keeps it intentionally for retained async/task aliases.
- **Grep-driven fixture classification required** — `issue.md:175` requires grepping `math\.`, `heapq\.`, `deque(`, `Counter(` under e2e pass/fail and demos, classifying every hit.
- **Dedicated transitional defaultdict helper** — `issue.md:168` specifies extracting the `collections.defaultdict(...)` and bare `defaultdict(...)` short-circuits into a small dedicated helper during M2 to avoid a half-deleted `resolve_python_compat_call_alias` body.

## Spot-checked code anchors

- `SIFR-IMPORT-0008` is the next free code (`registry.rs:34` ends at `IMPORT_CYCLE` = SIFR-IMPORT-0007).
- `compat_imports.rs:21-23` and `:44-45` carry the defaultdict short-circuits that route to typed lowering without synthetic imports — M3-owned.
- `compat_imports.rs:24-29` and the `deque`/`Counter` arms at `:46-47` are the synthetic-producing paths — M2-owned.
- `mod_impl.rs:647` is the sole synthetic-imports consumer — M2 explicitly removes it.
- Production strip-prefix sites enumerated by grep: `narrowing_helpers.rs:25-29`, `expr_call_and_literal_helpers.rs:3-13`, `queries_impl.rs:322`. All covered by M2's "remove codegen canonicalization" bullet.

The plan is implementable with low ambiguity. Proceed.
