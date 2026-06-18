Now I have everything needed for a thorough review. Here is my assessment.

---

## Wave 3 Documentation Review

**Verdict: NOT PR-READY** — three blockers, two important improvements, and optional polish below.

---

### Blockers

#### B1 — Family-shared examples are wrong for most codes in a family (all 170 pages)

The generator uses one erroneous/fixed pair per family. The result is that the code shown is only accurate for the first or most-prominent code in the family — and actively misleads developers for the rest.

The clearest failure is ASYNC. All seven ASYNC codes (`SIFR-ASYNC-0001` through `SIFR-ASYNC-0007`) render:

```python
# Erroneous
async def cached() -> int:
    return 1
```

This is the `SIFR-ASYNC-0001` ("no suspension") example. A developer hitting `SIFR-ASYNC-0003` ("blocking I/O called from async context") will see code that has no blocking I/O call at all, and the "fix" (add `await task.sleep(0.0)`) is wrong guidance for their actual problem. The real fixture (`blocking_io_direct_call_in_async_rejected.sifr`) clearly shows what should be there instead.

The same problem spans every family:
- **PACKAGE** (34 codes): all show `[package]\nname = "demo"` — meaningless for import-scope, trust-policy, archive-integrity, or publish-guard errors
- **OWN** (13 codes): all show the use-after-move pattern — wrong for borrow-conflict, loop-move, sendability, or IPC codes
- **TYPE** (13 codes): all show `count: int = "three"` — wrong for TypeVar, overflow, or reveal-type codes
- **FLOW** (9 codes): all show the missing-return example — wrong for break/continue, for-loop, or unsupported-form codes

This is a blocker because it degrades the key user promise of the per-code reference — that each page tells you what's wrong *for this specific code*. The ASYNC family has real per-code fixtures in the e2e suite; using them is feasible.

**Minimum acceptable bar:** fix ASYNC (7 pages) with per-code examples from the e2e fixtures before merging. Document the remaining families as a known gap with a follow-up ticket.

---

#### B2 — Wrong code fence language on 42 pages

BUILD codes (`SIFR-BUILD-0002` through `SIFR-BUILD-0901`, 6 pages) tag their erroneous/fixed examples as ` ```python ` but the content is a shell invocation:

```python   ← wrong
sifr build app.sifr --output /root/out
```

Should be ` ```bash `.

PACKAGE and WORKSPACE codes (34 + 8 pages) tag TOML content as ` ```python `:

```python   ← wrong
[package]
name = "demo"
```

Should be ` ```toml `.

Mintlify renders these with Python syntax highlighting, which is visually confusing and makes the code look broken. The validation pass does not catch language-tag mismatches.

---

#### B3 — "Why It Happens" boilerplate is factually wrong for operational error families

All 170 pages use this identical text:

> Sifr reports this diagnostic when **the program shape would make compilation ambiguous, unsafe, or inconsistent with the language contract**. The compiler stops at this point so the generated Rust does not rely on a hidden runtime fallback.

This is wrong for:
- **BUILD** codes — these are filesystem, Cargo, or Rustc execution failures entirely unrelated to program shape. `SIFR-BUILD-0002` fires because `--output /root/out` is unwritable, not because the source is ambiguous.
- **INTERNAL** codes — these are compiler bugs, not user program errors. `SIFR-INTERNAL-0001` explicitly says "indicates a compiler bug" in the `error-codes.mdx` Warning callout, yet the same page then claims the diagnostic fires because "the program shape would make compilation ambiguous." These two statements contradict each other on the same page.
- **PACKAGE** / **WORKSPACE** codes — these are manifest and configuration errors, not source-language contract violations.

At minimum the INTERNAL pages need the boilerplate replaced (two pages). The BUILD and PACKAGE families need distinct "Why It Happens" text.

---

### Important Improvements

#### I1 — `self update` / `self version` flags are completely undocumented (`cli/packages-workspaces.mdx:78–84`)

The Self Update section shows only:

```bash
sifr self version
sifr self update
```

The actual CLI has significant flags that developers need: `--dry-run`, `--format json`, `--channel`, `--version`, `--force` on `self update`; and `--short`, `--format json` on `self version`. The `--dry-run --format json` output is stable and machine-readable — it's exactly what CI tooling would consume. This section should document at least `--dry-run` and `--channel`.

#### I2 — `error-codes.mdx` description claims "24 families" but the catalog exports 23 (`error-codes.mdx:3`)

The frontmatter reads:

```
description: "A complete reference of all 24 Sifr diagnostic code families..."
```

The code catalog (`code_catalog.json`) exports 23 families. CODEGEN is listed in the accordion as "no active codes in the current release." The count is technically defensible (24 registered families), but the description leads developers to expect 24 entries with actionable codes. The fix is either to say "23 active families and 1 reserved" or keep 24 but make "reserved" explicit in the description.

---

### Optional Polish

**P1** — `SIFR-ASYNC-0003` (`docs/errors/SIFR-ASYNC-0003.mdx:7`): the family display name from the `.md` source says "Async effect and awaitability" — the description is correct but the "Why It Happens" body then contradicts the title by describing a "program shape" issue rather than an effect-system violation. Even when the boilerplate is replaced (B3), this family deserves wording specific to the effect-tracking model.

**P2** — `docs/cli/packages-workspaces.mdx:19` — The command table omits `sifr repair` from the summary sentence but includes it below. The table entry for `sifr vendor [PATH]` says the default path is `vendor`; the CLI code at `cli_model_and_entrypoint.rs:241` (`#[arg(default_value = "vendor")]`) confirms this is accurate. No correction needed, just note it's verified.

**P3** — `docs/stdlib/module-index.mdx:119–132` — The CardGroup footer links to `/stdlib/collections`, `/stdlib/io-filesystem`, `/stdlib/networking`, `/stdlib/concurrency`. These pages are registered in `docs.json` under Standard Library, so the routes are valid. However the CardGroup is placed after "Additional Modules," making it read as an afterthought rather than a navigation entry point. Moving it above the categorized tables would improve scan-ability.

**P4** — Per-code pages omit stability from the **title/description** frontmatter even though `stability: stable` is shown in the details table. For future `unstable` codes this will matter for discoverability.

---

### Routing and Mintlify Assessment (Q4)

The per-code pages (`docs/errors/SIFR-*.mdx`) are **not** registered in `docs.json`. Mintlify serves unregistered `.mdx` files as routable pages — they just don't appear in the sidebar. The `.mintignore` correctly exposes `errors/*.mdx` while hiding `errors/*.md`. The `validate` pass confirmed no broken page references. Links in `error-codes.mdx` use `/errors/SIFR-<CODE>` which Mintlify resolves to `docs/errors/SIFR-<CODE>.mdx`. This is structurally sound; the 170 pages will resolve correctly as hidden-from-nav reference pages.

The `sifr --explain <CODE>` path (`cli_model_and_entrypoint.rs:664`) reads `docs/errors/<CODE>.md` (not `.mdx`) — so the new `.mdx` pages do not affect the terminal `--explain` output. Both systems coexist correctly.

---

### Summary

| Issue | Scope | Blocking? |
|---|---|---|
| Family-shared examples wrong for most codes | 170 pages | Yes — fix ASYNC at minimum |
| Wrong code fence language (`python` for bash/toml) | 42 pages | Yes — mechanical fix |
| "Why It Happens" boilerplate wrong for BUILD/INTERNAL/PACKAGE | ~50 pages | Yes — 2 INTERNAL pages minimum |
| `self update` flags undocumented | 1 page | Important |
| "24 families" vs 23 active | 1 page | Important |
| CardGroup placement, stability | 1–2 pages | Optional |

The nav structure, routing, module-index completeness, CLI flag accuracy for init/repair/vendor/package/publish, and stdlib module coverage are all correct and ready. The blockers are in the generated per-code pages — fixing the ASYNC family examples, the language tags, and the INTERNAL boilerplate would put this in a mergeable state for a Wave 3 that's explicitly scoped as "v1 reference pages with known example gaps."
