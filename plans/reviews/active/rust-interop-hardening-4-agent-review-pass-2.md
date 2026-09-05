Full re-audit complete. Both round-1 blockers are closed, and I found no new actionable defects.

## Blocker verification

**Blocker 1 — MDX marker (CLOSED).** Compiled all three changed pages with `@mdx-js/mdx@3`, frontmatter stripped:

```
OK   docs/rust-interop.mdx len 17908
OK   docs/python-interop.mdx len 52251
OK   docs/guides/interop/blake3.mdx len 7706
```

`INLINE_REJECTION_MARKERS` (`check_stale_drafts.py:15-18`) is now suffix-keyed. The three live pages carry `{/* rust-interop-rejected */}` (`docs/rust-interop.mdx:9`, `docs/python-interop.mdx:9`, `docs/guides/interop/blake3.mdx:15`). The only `<!--` remaining in any `.mdx` is `docs/rust-interop.mdx:55`, inside an inline-code span — compiles fine and is inert to the scanner. The contract (`plans/issues/.../hardening.md:119-124`), architecture (`internal_docs/rust_interop_architecture.md:45-53`), area README (`README.md:94-101`), and public authoring section (`docs/rust-interop.mdx:49-58`) all state the suffix-specific rule consistently.

**Blocker 2 — self-test wiring (CLOSED).** `manifest.json:86-93` declares `rust-interop-stale-drafts-self-test` / `area-check-self-test`, and `stale-drafts` is selected by all four authoritative profiles (`create-pr.json:100`, `merge.json:78`, `nightly.json:80`, `release.json:79`).

```
suite=stale-drafts owner=compiler/package-management cases=2
rust interop stale draft scan ok
rust interop stale draft self-test ok: cases=20
rust interop verification ok: variants=8, failures=0, blocking_failures=0
```

## Checks run (read-only)

`--self-test` → `cases=20`; checked-in scan → ok; area → 8/8; `sifr_verify --self-test` → all lanes pass incl. "Rust interop profile execution self-test"; `py_compile` ok; `check_file_size_guardrails.py` PASS (2828 files, scanner 355 lines); `git diff --check` clean; `_is_rejection_context` gone from all source (only historical mentions in plans/reviews).

I also ran a 19-case adversarial battery in isolated temp trees. Every hostile input fails **closed** (flags the stale spelling): MDX-form marker in `.md`, uppercase marker, extra internal spaces, marker embedded in a larger HTML comment, `{/*rust-interop-rejected*/}` without spaces, marker on a wrapped continuation line, marker in a double-backtick span, tilde `~~~sifr-rejected` fence, 4-tick close of a 3-tick rejected fence, stale syntax inside an HTML comment or frontmatter, and `from rust import` in a `python` fence. Legitimate list-indented rejected fences still pass.

## Non-blocking observations

- **Self-test asymmetry on wrong marker type.** `mdx_cases` (`:306-316`) pins that the HTML form is inert in `.mdx`, but no case pins the mirror direction (MDX form inert in `.md`). Verified manually — it correctly flags — so this is coverage, not behavior.
- **`scan roots contain no Markdown or MDX files`** (`:75-76`) is unreachable from any self-test case; the `required-scan-roots` case only exercises the per-root branch.
- **Marker inside a non-rejected `sifr` fence still suppresses** (```` ```sifr ```` + `extern rust crate  <!-- rust-interop-rejected -->` → no failure). Spec-permitted as an "inline prose/code occurrence", still unpinned by a self-test. Carried over from round 1.
- **Tilde fences fail closed with no explanatory diagnostic**, and the authoring docs never say backtick-only. Carried over from round 1.
- **Stray unmatched backtick can make a real marker look code-spanned**, e.g. `Use \` for code. <!-- marker --> \`extern rust\`` → spurious failure. Fail-closed and the line is malformed markdown anyway.
- **create-pr budget headroom.** `rust_interop_checks` is `budget_ms: 5000, blocking` (`create-pr.json:15`); the warm area run now measures 3.59s wall (~3.2s in-process), of which the new case is ~82ms. ~28% headroom — worth confirming `name=rust_interop_checks ... status=pass` when the create-PR lane runs.
- **Heavy gates not run here:** `scripts/run_all_tests.sh --profile create-pr` and the merge gate. The diff is docs plus one Python check, and the focused area lane passes.
- `plans/issues/.../hardening.md:242` marking `hardening_3` as merged is now factually correct (`20a9f55b4`, PR #3022), retiring the round-1 housekeeping note.

Actionable findings: 0. SATISFIED.
