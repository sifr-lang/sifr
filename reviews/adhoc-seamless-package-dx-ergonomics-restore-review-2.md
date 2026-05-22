

Now let me do the targeted second-pass check across the 6 areas from round 1.

---

## Second-Round Verification

### 1. `[scripts]` — Restored as Sifr-owned workflow aliases, not shell scripts?

**Verified.** Lines 188-202, 264-279, 1055, 1105 all reference `[scripts]`. Structured `command` + `args` argv array, no shell syntax. Script names share namespace with app targets, ambiguity is reported, not resolved silently (line 251, 277). `sifr run --script <name>` always selects a script (line 278). Script expansion is visible in verbose output (line 276). `sifr run --bin <name>` always selects an app target. ✅

### 2. `sifr run <script>` and `sifr run --script <script>` — Ergonomic, deterministic, ambiguity handled?

**Verified.** Resolution order (lines 246-259) is complete and deterministic:
- Explicit `.sifr` → ephemeral target
- `--bin <name>` → explicit app target
- `--script <name>` → explicit script expansion
- Positional matches both → ambiguity diagnostic (requires `--bin` or `--script`)
- Positional matches app only → app target
- Positional matches script only → script expansion
- `[package].default-run` → named app target
- `src/main.sifr` → default app target
- Exactly one discovered target → that target
- Otherwise → `SIFR-PACKAGE-0605`

Explicit flags always win. Ambiguity is reported, never silently resolved. ✅

### 3. Cargo-backed commands — Cargo names/semantics preserved; scripts don't bypass alignment after expansion?

**Verified.** Line 279: "After script expansion, any nested Cargo-backed command is validated against the Cargo CLI alignment matrix exactly like a direct command invocation." The script command allowlist (line 273) is constrained to Sifr-owned commands (`run`, `check`, `build`, `test`, `fetch`, `tree`, `package`, `publish`, `vendor`, `repair`). The Cargo alignment contract (lines 608-616) applies to all delegated Cargo subcommands. Scripts are a Sifr-owned pre-resolution layer; after expansion, Cargo commands are validated normally. ✅

### 4. `sifr --explain <diagnostic-code>` — Sifr-owned, safe, never performs package operations?

**Verified.** Lines 100, 656, 673, 913-917: `sifr --explain <diagnostic-code>` is documented as accepting stable Sifr diagnostic codes, printing meaning/common causes/docs links/recovery commands, and **never performing package operations**. `SIFR-PACKAGE-0101` explain output explicitly covers redaction behavior, generic recovery steps, and `sifr fetch --locked` guidance. ✅

### 5. Removed items — Still out?

| Item | Status | Evidence |
|---|---|---|
| Dependency groups | ✅ Removed | Line 474: "Sifr v1 does not add `[test-dependencies]`, custom dependency groups, group composition, or uv-style group selection." |
| `[test-dependencies]` | ✅ Removed | Line 101, 474: Cargo-compatible `[dev-dependencies]` only. |
| `--filter` | ✅ Kept removed | Line 104: "Keep advanced Phase 37 selectors such as `--filter` out of the public package CLI." |
| `--sifr-only` | ✅ Not present | No reference; line 104 context implies kept-out. |
| `sifr package --dry-run` | ✅ Removed | Line 103: "Keep `sifr package` without `--dry-run`"; line 672: "intentionally absent." |
| `sifr fix` | ✅ Kept removed | Line 102: "Keep `sifr repair` instead of `sifr fix`." |

All removed items remain out. ✅

### 6. Milestones and validation — Include scripts and explain?

**Mostly verified.** Scripts appear explicitly in milestone 1 (line 1055), milestone 3 (line 1105), and validation test `sifr run <script>` / `sifr run --script <name>` (line 1118). The `--explain` command is documented in CLI semantics (line 673) and the ergonomics audit (line 100), but the milestone 3 scope does not explicitly name `--explain` as a separate implementation item — it falls under the broader "Sifr diagnostics" surface covered by "Sifr diagnostics, and projection repair" in the CLI alignment contract (line 613). This is a minor documentation gap, not a blocker.

---

## VERDICT: **READY**

All 6 areas pass. No exact blockers.

**Non-blocking observation:** The milestone 3 scope (line 1099-1121) covers `--explain` implicitly through "Sifr diagnostics" (line 613) but does not explicitly name `sifr --explain` as a scope item. A reviewer implementing milestone 3 might look for it and find no explicit mention. If you want formal coverage, add to milestone 3 scope: "Implement `sifr --explain <diagnostic-code>` as Sifr-owned diagnostic help, including `SIFR-PACKAGE-0101` wrapper docs, redaction behavior, and safe recovery guidance." — but this is a style concern, not a functional gap.
