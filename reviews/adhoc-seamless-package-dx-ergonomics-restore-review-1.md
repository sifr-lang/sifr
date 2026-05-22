

Now I have a full picture. Let me do the systematic review across all 6 areas.

**VERDICT: READY**

The recovered design is correct and complete. Below are the detailed findings:

---

## 1. Scripts Design: Ergonomic and Deterministic?

**Yes.** The `[scripts]` design (lines 188-202, 250-254, 264-279, 273) is restored correctly:

- Structured `command` + `args` argv array, not shell strings — correct per Non-Goals line 56.
- Script names share a namespace with discovered app target names, with explicit ambiguity reporting (line 251, 277) — deterministic.
- `sifr run --script <name>` always selects a script, `sifr run --bin <name>` always selects an app target (line 278) — no hidden priority.
- `command` is constrained to an explicit allowlist (`run`, `check`, `build`, `test`, `fetch`, `tree`, `package`, `publish`, `vendor`, `repair`) at line 273 — prevents arbitrary shell injection.
- Script expansion is visible in verbose output (line 276) — good for debuggability.
- `sifr --explain <diagnostic-code>` is also restored at lines 100, 656, 673, 913-917 as a separate Sifr-owned diagnostic help command — not confused with scripts.

One small observation: `repair` is in the script command allowlist (line 273) but `sifr run --script` is awkward UX for a repair alias. However, it's not a blocker — `sifr run` scripts are for workflow commands, and repair is legitimately a workflow command (e.g., `sifr run --script repair-locked` → `sifr check --locked`).

---

## 2. `sifr run` App/Script Ambiguity Handling: Complete?

**Yes.** The resolution order (lines 246-257) is complete:

1. Explicit `.sifr` file → ephemeral target ✓
2. `--bin <name>` → explicit app target ✓
3. `--script <name>` → explicit script expansion ✓
4. Positional matches both → ambiguity diagnostic (requires `--bin` or `--script`) ✓
5. Positional matches app target only → app target ✓
6. Positional matches script only → script expansion ✓
7. `[package].default-run` → named app target ✓
8. `src/main.sifr` → default app target ✓
9. Exactly one discovered target → that target ✓
10. Otherwise → `SIFR-PACKAGE-0605` ✓

The `--bin`/`--script` disambiguators are sufficient. No priority rules needed — explicit flags always win, ambiguity is reported, never resolved silently.

---

## 3. Scripts: Cargo Alignment and Scope?

**Correct.** Scripts are explicitly scoped as Sifr-owned pre-resolution (line 279: "After script expansion, any nested Cargo-backed command is validated against the Cargo CLI alignment matrix exactly like a direct command invocation"). This means:

- Script `command` names (`run`, `check`, `build`, `test`, `fetch`, `tree`, `package`, `publish`, `vendor`, `repair`) are Sifr-owned command identifiers.
- They are NOT Cargo subcommand names — they happen to share the same names as Cargo subcommands where Cargo delegates, but the script system itself is a Sifr-owned layer above Cargo.
- `repair` is not a Cargo command — correctly in the script command allowlist as a Sifr-owned command.
- The allowlist at line 273 includes only commands implemented by this phase or explicitly allowed by the schema — no arbitrary shell, no arbitrary binaries.

This is consistent with the Cargo alignment contract (lines 608-613): "Sifr-specific command surface is allowed only for Sifr-owned behavior Cargo cannot express: explicit `.sifr` file execution, named workflow scripts, Sifr source layout/privacy, Sifr diagnostics, and projection repair." Scripts are Sifr-owned and do not inherit Cargo semantics — they expand to validated command plans.

---

## 4. Any Other Good Ergonomics Still Removed?

**No.** The ergonomics audit section (lines 95-105) confirms all correctly restored/removed items. Checking against the round-1 review's removals:

| Item from round-1 | Restored? | In doc? |
|---|---|---|
| `[scripts]` | ✓ restored | Lines 188-202, 250-279 |
| `sifr --explain` | ✓ restored | Lines 100, 656, 673, 913-917 |
| `[package].default-run` | ✓ kept | Lines 196, 200, 254, 212, 1054 |
| Nested `src/bin/` paths | ✓ kept | Line 243: `src/bin/tools/migrate.sifr -> "tools/migrate"` |
| `--message-format json` | deferred (not removed) | Lines 637-640 show `--message-format fmt` in CLI synopsis — acceptable as a deferred extension |
| Layout migration with rollback | v1 has it | Lines 982-1030: migration script, rollback descriptor, `--apply-partial` |
| `SIFR-PACKAGE-0707` | ✓ kept | Line 968 |
| `SIFR-PACKAGE-0708` | ✓ kept | Line 969 |
| Fallback scanning | ✓ kept | Lines 575-577 |
| Custom dependency groups | ✓ removed (correct) | Line 474 |
| `[test-dependencies]` | ✓ removed (correct) | Line 474 |
| `--filter`, `--sifr-only` | ✓ kept removed | Line 104 |
| `sifr fix` | ✓ kept removed | Line 102 |
| `sifr package --dry-run` | ✓ kept removed | Line 103, 672 |

All items match the user's stated intent precisely.

---

## 5. Dependency Groups / Test-Dependencies: Correctly Excluded?

**Yes.** Line 474 explicitly states: "Sifr v1 does not add `[test-dependencies]`, custom dependency groups, group composition, or uv-style group selection." The audit at line 101 confirms: "Keep Cargo-compatible `[dev-dependencies]` instead of `[test-dependencies]`, custom dependency groups, or uv-style group selection."

The dependency model correctly uses only:
- `[dependencies]` (runtime) — line 471
- `[dev-dependencies]` (Cargo-compatible dev) — line 472
- `build-dependencies` and target-specific sections are reserved (line 473), not exposed as Sifr import dependencies in v1

---

## 6. Implementation Readiness and Long-Term Maintainability?

**Ready.** The design is coherent, consistent, and bounded:

- **Well-scoped milestones** (lines 1048-1239): Each milestone has bounded scope, independent testability, and clear acceptance criteria. Milestone 1 covers layout/scripts, milestone 2 covers projection, milestone 3 covers CLI wiring, etc.
- **Consistent diagnostic range**: `SIFR-PACKAGE-07xx` (0701-0711) is fully allocated, no gaps, no duplicates. 0707 and 0708 are properly included (layout migration validation, Cargo alias conflict).
- **Script command allowlist is stable**: `run`, `check`, `build`, `test`, `fetch`, `tree`, `package`, `publish`, `vendor`, `repair` — all defined, all bounded.
- **Cargo alignment matrix is complete**: Lines 608-633 reference all 13 Cargo subcommands with authoritative docs links. No undocumented aliases.
- **`PackageSession` and `OperationPlan` architecture is sound** (lines 676-743): Single orchestration layer, consistent planning rules, redaction, lock modes.
- **Guardrails** (line 1039): After milestone 7, old Phase 37 layouts fail unless explicitly marked as parser/backfill regression — long-term maintenance path is covered.
- **No data-dependent unwraps in generated runtime** (lines 763-774): Consistent with the Phase 37 generated-runtime safety contract.

---

## Non-Blocking Observations

1. **`--message-format fmt`** (lines 637-640): The synopsis shows `--message-format fmt` but no accepted values are documented. The command semantics (line 670) says "follows Cargo's flag name and accepted values." This is fine for spec purposes, but the implementation will need to document which values are accepted (likely `human` and `json` when the feature is wired). Not a blocker.

2. **Script `command` = `repair`**: Valid but unconventional. A script named `repair` expands to `sifr repair`. This is fine as an alias mechanism but could confuse users who expect `sifr run --script repair` to run an app target. The ambiguity check at line 251 covers this: if a package has `src/bin/repair.sifr` AND `[scripts].repair`, the ambiguity diagnostic fires. If not, `sifr run --script repair` works. No change needed.

3. **Nested `src/bin/` paths**: Line 243 shows `src/bin/tools/migrate.sifr -> "tools/migrate"` as the target name. The round-1 review deferred this to v2, but the current restored doc still includes it. Per the user's intent (scripts are Sifr-owned, not Cargo), this is a layout-discovered target, not a script — so it stays. If `tools/migrate` conflicts with a script named `tools/migrate`, the ambiguity check fires. Consistent.

---

**All 6 areas pass. No blockers.**
