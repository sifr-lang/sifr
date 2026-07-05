## M12 Wave 1 Review — Retained Compiler-Native Stdlib Intrinsic Allowlist

**Focused validation** already run locally by the user (guard, self-test, resource-cert gate, coverage matrix readiness, git diff --check, milestone grep) is consistent with what I re-verified: guard passes with `exact=177, prefix=5, registry=28, preamble=16`, matching a direct count of the dispatcher/registry/preamble surfaces.

### Findings, severity-ordered

**LOW — not blockers, but worth capturing before opening the PR**

1. `guardrails.json:54-59` documents `args: []` only, while `profile_runner.py:294-295` runs the guard twice (bare + `--self-test`). This is the same documentation drift previously accepted for `source-crate-dependency-direction` (M11 review pass 2 finding #5). Prior precedent is "non-blocking documentation drift" because `guardrails.json` is not actually consumed by the runner today; adding a second entry with `args:["--self-test"]` would keep the registry honest.

2. `internal_docs/stdlib_retained_compiler_intrinsics.toml:96-112` names `collections/counter_defaultdict_intrinsics.rs` and `collections/set_and_list_intrinsics.rs`, but both files contain only `lower_counter_*` helpers. The retention reason ("Counter/defaultdict semantics and core collection behavior…") is accurate, but reviewers reading the allowlist may be surprised by the file names. Preexisting from M10 wave 12; this PR is not the place to rename, but flagging it as a minor "misleading about ownership" risk.

3. Scope caveat: the guard reads only `crates/sifr_codegen/src/intrinsics/registry.rs`, `intrinsics/registry/**/*.rs`, and `preamble/**/*.rs`. It does not observe `intrinsic_method_emitters/`, `methods/`, or `preamble.rs` (the module aggregator). The docs (`internal_docs/sifr_sysroot_and_stdlib_architecture.md:152-157` and `749-752`) correctly describe the guard's scope as dispatcher + registry files + preamble files, so this is intentional — but a new stdlib-native behavior added via a method emitter would bypass the guard. Non-blocking for the freeze goal since the current dispatcher is the sole `intrinsics::lower_intrinsic` entrypoint, and method emitters lower type methods rather than stdlib functions.

4. `EXACT_INTRINSIC_RE = r'"([A-Za-z0-9_]+)"\s*(?=\||=>)'` (`check_stdlib_native_intrinsic_allowlist.py:21`) matches any snake_case string literal in `registry.rs` followed by `|` or `=>`. Today `registry.rs` contains only match-arm string literals, so it is correct and produces the expected 177 exact matches. Fragile if a future contributor introduces a non-arm string literal that happens to precede `|`/`=>` (would produce false-positive "missing allowlist entries" or accidentally satisfy a duplicate check). Non-blocking.

5. Self-test coverage (`check_stdlib_native_intrinsic_allowlist.py:169-232`) exercises missing-exact, stale-prefix, duplicate, and missing-reason paths. It does not cover: unknown/duplicate `id`, non-list `surface`, new-preamble-file detection, or the "surface has no retained files or intrinsics" branch. Representative but not exhaustive — acceptable for wave 1.

6. Cosmetic: `_validate` (line 111) checks `if has_items and not reason` after `_required_text` (line 85-90) has already appended a failure for a missing reason. The second check is unreachable when reason is missing because `_required_text` returns `""` only after logging. Minor dead branch.

### Answers to review dimensions

1. **Guard accuracy vs. current sifr_codegen surface** — Correct. The dispatcher scan (line 21) accurately mirrors `registry.rs` (177 arms + 5 prefix), and the two rglobs mirror registry (28) and preamble (16). One dispatcher, one entry point (`lower_intrinsic` at `registry.rs:39`).

2. **Allowlist completeness/explicitness** — Complete for wave 1 scope. Every retained surface has a `reason` field that plainly states "compiler/runtime-owned pending certification" or "language-owned glue"; the resource-shaped groups (`_sifr.process/net/tls/signal/http/python/task`) are explicitly labeled as pending certification, matching what M11 established. No migrated leaves (platform, html, calendar, uuid, math, hash, base encoders, regex, url, toml, json, encoding, unicode, i18n, gzip/zipfile, datetime, bytes helpers, collections helpers) appear in the allowlist.

3. **Missing/stale/duplicate/reason coverage** — Symmetric set diff handles missing+stale for all four dimensions; duplicates are caught with owner attribution; empty reason is rejected. Adequate for M12 wave 1.

4. **Create-pr/merge wiring** — Correct. `profile_runner.py:293-295` runs the guard + self-test inside `run_core_guardrails`, which executes on both `legacy-facade` execution modes (create-pr and merge). Placed alongside the existing sysroot-stdlib resource certification gate.

5. **Docs framing** — Correct. Architecture doc (`sifr_sysroot_and_stdlib_architecture.md:149-157, 749-752`) explicitly frames this as a freeze/allowlist guard, distinct from the resource certification gate, and does not claim deletion or resource migration. Issue plan (`ad-hoc-sifr-sysroot-stdlib-toolchain.md:1943-1965`) makes the "deliberately broad because M11 proved resource surfaces cannot safely move yet" framing clear.

6. **Blockers** — None.

### VERDICT: PASS

Ready for PR. Consider the two LOW documentation items (add `--self-test` sibling to `guardrails.json`; note file-naming drift in `_sifr.collections`) as non-blocking cleanup, either now or as a wave-follow-up.
