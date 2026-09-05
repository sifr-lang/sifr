## Review: `hardening_4` (branch `agent/rust-interop-hardening-4`)

Scope reviewed: full uncommitted diff vs `origin/main` (9 files), the frozen *Structured Rejected-Syntax Contract*, and the scanner's behavior under adversarial inputs (isolated temp trees only; no repo files modified).

### Verified as claimed
- `check_stale_drafts.py --self-test` → `ok: cases=13` (prompt said 11 — see observations); truly isolated (`_scan_repository(root)` is parameterized, `REPO_ROOT` only used by the real scan path).
- Checked-in scan passes; `areas run --area rust_interop` → `variants=7, failures=0`; file-size guardrail PASS (2828 files); `git diff --check` clean; bad-arg usage guard returns 2.
- `_is_rejection_context` and its lexical markers are gone. All 15 in-scope stale mentions carry the same-line marker (enumerated by re-running the pattern table over scan scope). Fence parser is fail-closed on tilde fences, uppercase/suffixed info strings, wrong tick counts, and nested openers. The `python`-fence check is now *stricter* than `origin/main` (it splits the info string, so `python title` is caught).

---

## Actionable findings

### 1. Blocking — the `<!-- rust-interop-rejected -->` marker makes three published MDX pages fail to compile (regression)
`docs/rust-interop.mdx:9`, `docs/python-interop.mdx:9`, `docs/guides/interop/blake3.mdx:15`

MDX (v2/v3, which Mintlify uses — `docs/docs.json`, `docs/AGENTS.md`) does not support HTML comments. Compiled the actual files with `@mdx-js/mdx@3` (frontmatter stripped):

```
docs/rust-interop.mdx           FAIL line 4  : Unexpected character `!` (U+0021) before name …
docs/python-interop.mdx         FAIL line 4  : Unexpected character `!` …
docs/guides/interop/blake3.mdx  FAIL line 10 : Unexpected character `!` …
origin/main rust-interop.mdx OK / python-interop.mdx OK / blake3.mdx OK
```

The error text itself says "note: to create a comment in MDX, use…". Corroborating repo evidence: `origin/main` has **zero** `<!--` occurrences in any `.mdx`; the established idiom is `{/* … */}` (`docs/errors/*.mdx:7`, `docs/AGENTS.md`). All three files are live navigation pages (`docs/docs.json:74,75,181`). Nothing in `run_all_tests.sh` compiles MDX, so the local gate cannot catch this — the breakage surfaces only at docs deploy.

Root cause is in the contract, not just the migration: `INLINE_REJECTION_MARKER` (`check_stale_drafts.py:15`) accepts only the HTML form, so *no valid inline marker exists for `.mdx`*. Worse, the new authoring guidance actively instructs authors to break the build — `docs/rust-interop.mdx:52-54`, `internal_docs/rust_interop_architecture.md:45-48`, `verification/areas/rust_interop/README.md:95-99`.

Fix requires accepting an MDX-safe form (verified: `{/* rust-interop-rejected */}` compiles OK) for `.mdx` files, migrating those three lines, adding a self-test case per file type, and amending the frozen contract in `plans/issues/active/rust-interop-verification-matrix-hardening.md:114-131` (which froze the HTML form) — the spec as written is unimplementable in MDX.

### 2. Blocking — the new isolated `--self-test` is not executed by the area or any profile lane
`verification/areas/rust_interop/manifest.json` (`stale-drafts` suite)

The `stale-drafts` suite declares one case (`area-check`). The three sibling suites each declare a paired `area-check-self-test` case (`rust-interop-matrix-self-test`, `rust-interop-tiers-self-test`, `rust-interop-compatibility-matrix-self-test`). Evidence from the area run:

```
suite=stale-drafts owner=compiler/package-management cases=1
rust interop stale draft scan ok
```

versus `rust interop tier self-test ok: cases=6` for `tiers`. The adapter command already exists (`verification/runner/sifr_verify/area_adapter.py:45`, `AREA_CHECK_SELF_TEST_COMMAND`), and `hardening_1` wired `stale-drafts` into create-pr/merge/nightly/release — so a one-case manifest addition is all that's missing. As it stands, the milestone's exit gate ("the scanner rejects an unmarked stale spelling even when nearby prose says…") is proven only by a manual command that no authoritative lane runs, and a future regression in the fence/marker parser would pass every profile. This directly undercuts `hardening_4`'s stated purpose (mechanical rather than prose enforcement) and mirrors the defect `hardening_2` explicitly fixed for `check_tiers.py`.

---

## Non-blocking observations

- **Marker inside inline code is honored**, so a line that merely *documents* the marker self-exempts: `` The marker is `<!-- rust-interop-rejected -->` and `extern rust` is stale. `` → no failure. This already applies to `docs/rust-interop.mdx:54`. Requiring the marker outside backticks would close it.
- **Marker inside an accepted `sifr` fence suppresses detection** (`` ```sifr `` / `extern rust crate  # <!-- rust-interop-rejected -->`) → no failure. Spec-permitted ("inline prose/code occurrence"), but no self-test pins the behavior either way, so it can drift silently.
- **Stale spellings on a fence *info* line are never scanned** — the opener `continue`s before pattern matching (`check_stale_drafts.py:96`). `` ```text extern rust `` passes. Narrow, but a real hole.
- **No valid rejected form for tilde fences.** `~~~sifr-rejected` fails closed (content flagged) with no diagnostic explaining that only backtick fences are supported, and the authoring docs don't say so.
- **Panic-surface check has zero self-test coverage.** Verified manually that it still fires, and that an inline marker now suppresses it (`origin/main` ran it regardless of rejection context — `check_stale_drafts.py:106-111` is a deliberate-looking but untested semantic change).
- **`_scan_repository` silently skips missing scan roots** (`:59`, needed for temp trees). No assertion that the real scan visited a nonzero/expected file count, so renaming a `SCAN_ROOTS` entry would pass green. Same weakness existed on `origin/main`, but the new self-test harness makes a coverage assertion cheap.
- **Bundled housekeeping**: the `hardening_3` row change to `merged` / PR #3022 (`plans/issues/.../hardening.md:238`) belongs to the previous item; harmless, and the `hardening_4` → `in progress` row is correct.
- **Prompt evidence drift**: the review brief states the self-test passes 11 cases; the current entrypoint reports `cases=13` (12 tuple cases + the exclusion case).
- I did not run `scripts/run_all_tests.sh --profile create-pr` or the merge gate (heavy); the diff is docs plus one Python check, and the focused area lane passes.

Actionable findings: 2. CHANGES REQUIRED.
