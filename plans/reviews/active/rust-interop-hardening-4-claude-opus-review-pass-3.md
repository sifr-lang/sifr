Execution errorl audit — PR #3023 (`3d87d922b`)

### Packaging

| Check | Result |
|---|---|
| Commits | exactly 1 — `3d87d922b "Make Rust interop rejection context structural"` |
| Base / head | `main` ← `agent/rust-interop-hardening-4`; remote head == local == `3d87d922b`; `MERGEABLE` |
| Files | 12, all in hardening_4 scope (scanner, manifest, area README, 3 internal/plan docs, 3 live MDX pages, 2 review artifacts). No unrelated files, no crate/code changes. |
| Working tree | clean vs `HEAD` except one untracked stray (below) |
| PR body claims | pass 1 = 2 findings corrected, pass 2 = 0/SATISFIED, area 8 variants / 0 failures — all verified true |

### Re-verified independently (read-only)

- **Scanner semantics.** `_is_rejection_context` and every lexical marker are gone (only the plan's own "removes …" bullet mentions the name). Fence state machine is exact: `is_rejected` requires `len(ticks)==3 and info=="sifr-rejected"`; `_looks_like_rejected_fence` catches near-miss spellings as `malformed rejected fence`; unclosed rejected fences are reported.
- **Suffix locality.** `INLINE_REJECTION_MARKERS` is suffix-keyed. Confirmed by probe: MDX form in `.md` → flagged; `{/*rust-interop-rejected*/}` without spaces → flagged; extra internal spaces → flagged; marker inside an inline-code span → inert (case 4 pins it). All 15 in-scope markers use the correct per-suffix form.
- **MDX render safety.** Compiled all three changed pages with `@mdx-js/mdx@3` (frontmatter stripped): `OK docs/rust-interop.mdx 17908`, `OK docs/python-interop.mdx 52251`, `OK docs/guides/interop/blake3.mdx 7706` — byte-identical to pass 2. The only `<!--` in any `.mdx` is `docs/rust-interop.mdx:55`, inside a code span.
- **Python-fence + panic behavior.** Python-fence rule is *not* suppressible by an inline marker (case 13); panic-surface check still fires on accepted examples (case 15) and is exempt under rejection (case 16).
- **Exclusions / required roots.** Real scan asserts all three roots and non-zero file count; self-test pins `plans/archive` + `plans/reviews` exclusion and `missing scan root: internal_docs|plans`.
- **20 cases, non-vacuous.** `--self-test` → `cases=20` (16 md + 2 mdx + exclusion + required-roots). Five injected mutations (loose fence match, suffix-agnostic marker, code-span ignored, unclosed check disabled, python-fence check disabled) each make the self-test exit 1 — the suite genuinely constrains behavior.
- **Manifest / profile execution.** `rust-interop-stale-drafts-self-test` / `area-check-self-test` present; `stale-drafts` selected by create-pr, merge, nightly, release. Area run: `suite=stale-drafts … cases=2`, `self-test ok: cases=20`, `variants=8, failures=0, blocking_failures=0`. `sifr_verify --self-test`: all 8 lanes pass incl. "Rust interop profile execution self-test".
- **Adversarial battery (18 hostile inputs, isolated temp trees).** All fail closed: tilde fence, uppercase info, 4-tick close of a 3-tick fence, unclosed python fence, stale text in HTML comment block / frontmatter, stray-backtick line, `.md` with MDX marker. No false positives on the checked-in tree (`scan ok`).
- **Gates.** File-size PASS (2828 files; scanner 355 lines), `py_compile` ok, manifest JSON valid, `git diff --check` clean, usage guard exits 2.
- **Docs/tracking.** Contract text in the plan, `internal_docs/rust_interop_architecture.md`, area README, and the new public "Documenting Rejected Syntax" section state the same suffix-specific rule. `hardening_3 → merged (#3022)` is factually correct (`20a9f55b4`).

No stale claim, unreviewed delta, false positive/negative, or regression found.

### Non-blocking observations

1. **PR is still `isDraft: true`** — must be marked ready before merge (state change only, not a code finding).
2. **Stray untracked artifact**: `plans/reviews/active/rust-interop-hardening-4-claude-opus-review-pass-3.md` is 0 bytes in the working tree. Not in the PR; delete it rather than committing an empty review file.
3. **No `sifr-rejected` fence exists in checked-in docs** — the block form is exercised only by the self-test. Correct today (all live mentions are inline), just worth knowing the fence path has no production instance.
4. **`hardening_4` row says `in progress`** with no PR link or review evidence, whereas `hardening_3`'s own PR recorded `review approved; PR pending` plus lane evidence. Consistent with the successor-records-merge convention, but slightly thinner evidence than the precedent.
5. **Marker inside an accepted `sifr` fence still suppresses** detection; spec-permitted, still unpinned by a self-test (carried from passes 1–2).
6. **Tilde fences and 4-space-indented fences**: `~~~sifr-rejected` fails closed with no diagnostic explaining backtick-only; a 4-space-indented ```` ```sifr-rejected ```` is honored because the line is `strip()`ed. Both narrow; authoring docs are silent on either.
7. **`scan roots contain no Markdown or MDX files`** branch remains unreachable from any self-test case (carried from pass 2).
8. Heavy lanes not re-run here per instruction; I relied on the authoritative create-PR report (22/22 lanes, `rust_interop_checks` 3244/5000 ms blocking, 8/8 Rust area, 6/6 hardening variants, cold-run wall-time advisory only) and re-measured the area lane directly at ~3.1 s warm.

Actionable findings: 0. SATISFIED.
