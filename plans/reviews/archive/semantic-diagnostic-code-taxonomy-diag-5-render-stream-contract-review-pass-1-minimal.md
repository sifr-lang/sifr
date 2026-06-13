# DIAG-5 Slice 3 — Render Stream Contract — Review Pass 1 (Minimal)

**Branch:** `codex/diag-5-render-stream-contract`
**Scope reviewed:** uncommitted changes in `crates/sifr/src/main.rs` (138 lines, +117/-21).
**Lens:** correctness, regression, scope. No source files modified during review.
**Verdict:** Approve. No blocking issues; minor non-blocking observations only.

---

## 1. Contract claim and how it is met

Claim: CLI `human`/`json`/`compact` diagnostics must render from one canonical sorted-and-capped stream, with no fallback/legacy path.

The change establishes this with three named layers in [crates/sifr/src/main.rs:398-455](crates/sifr/src/main.rs:398):

| Function | Role |
| --- | --- |
| `canonical_diagnostic_stream` (398) | Single source of truth for ordering + capping — wraps `apply_diagnostic_recovery_limits`. |
| `render_diagnostic_stream` (402) | Pure renderer: takes the already-canonical slice + a `DiagnosticFormat`, returns `Result<String, serde_json::Error>`. No I/O, no recomputation. |
| `render_diagnostic_output` (433) | Composes the two above. The only place format and canonical stream meet. |
| `render_diagnostics` (441) | Thin stderr+exit-code adapter over `render_diagnostic_output`. |

All `cmd_*` paths route through `render_diagnostics`:
[cmd_build:463/475](crates/sifr/src/main.rs:463), [cmd_run:485/507](crates/sifr/src/main.rs:485), [cmd_check:517/537](crates/sifr/src/main.rs:517), [cmd_test:547/557](crates/sifr/src/main.rs:547), [cmd_emit:567/574](crates/sifr/src/main.rs:567). I grepped the workspace for direct uses of `apply_diagnostic_recovery_limits`, `to_string_pretty` on diagnostics, and `render_compact_diagnostics`; the only production callsite of the recovery limits outside its own crate is `canonical_diagnostic_stream`, and `render_compact_diagnostics` is invoked exclusively from `render_diagnostic_stream` (test-only references aside). **No alternate render path survives.**

## 2. Behavioral parity with the prior implementation

I diffed the old vs. new bytes-on-stderr for each format:

- **Human** — old: `writeln!(io::stderr(), "{label}: {message}")` per item; new: `writeln!(output, …)` per item, then `write!(io::stderr(), "{output}")`. Net byte stream identical (each line ends in `\n`, no extra trailing `\n`).
- **JSON** — old: `writeln!(io::stderr(), "{json}")`; new: `writeln!(output, "{json}")` + `write!(io::stderr(), "{output}")`. Identical (`to_string_pretty` body + one trailing `\n`).
- **Compact** — old: `write!(io::stderr(), "{compact_output}")`; new: `write!(output, "{}", render_compact_diagnostics(diagnostics))` + `write!(io::stderr(), "{output}")`. Identical (compact output already ends in `\n` per group line; no extra newline added).
- **JSON serialization failure** — old emitted `"build error: failed to serialize diagnostics as json: {e}"` to stderr and returned `EXIT_INTERNAL_COMPILER_FAILURE` ([crates/sifr/src/main.rs:447-451](crates/sifr/src/main.rs:447)); new does the same via the `Err(e)` arm of `render_diagnostics`. Same exit code, same message. Note: only the `Json` branch can produce this `Err`; Human/Compact never enter that arm — same as before.

Exit-code wiring is unchanged: `diagnostic_exit_code(errors)` is still computed from the *unfiltered* input slice ([crates/sifr/src/main.rs:454](crates/sifr/src/main.rs:454)), preserving the "internal panic ⇒ EXIT_INTERNAL_COMPILER_FAILURE" rule even if the panic diagnostic survives or is grouped by the recovery limits.

## 3. Test added — what it actually proves

[crates/sifr/src/main.rs:1428-1504](crates/sifr/src/main.rs:1428) — `test_diagnostic_formats_share_canonical_sorted_capped_stream`.

Fixture: 49 distinct `SIFR-TYPE-0100..0148` errors (insertion-reversed) plus 8 duplicated `SIFR-TYPE-0002` errors. Asserts:

- `canonical.len() == 50` (top-level cap).
- `canonical[0..5]` are the 5 retained `SIFR-TYPE-0002` entries (per-group cap = 5).
- `canonical[5].message == "... +3 more similar diagnostics"` (summary slot at expected index — depends on `severity_rank=0`, code `SIFR-TYPE-0002` < `SIFR-TYPE-0100`, so the duplicated group sorts first).
- `SIFR-TYPE-0143` is included; `SIFR-TYPE-0144` is excluded (boundary of the 50-item top-level truncation: `5 retained + 1 summary + 44 distinct = 50`).
- JSON round-trip equals `canonical` exactly — i.e. `serde_json::from_str(&render_diagnostic_output(_, Json)?) == canonical`.
- Human output equals `canonical.iter().map(legacy_diagnostic_display).join("\n") + "\n"`.
- Compact: first line is the severity summary; the sum of `(xN)` group counts equals `canonical.len() == 50`; specific text `"error [SIFR-TYPE-0143] distinct diagnostic 43 (x1)"` is present; `"SIFR-TYPE-0144"` is absent.

The three format outputs are all derived from the *same* `canonical_diagnostic_stream(&diagnostics)` call by `render_diagnostic_output`, so the assertions transitively prove the contract: "all three formats render from one canonical sorted-and-capped stream." This is the right shape of test for this slice.

## 4. Non-blocking observations

1. **Buffering shift for Human format.** Old code wrote line-by-line directly to stderr; new code buffers the full body into a `String` before a single `write!`. With diagnostics bounded to ≤50, memory cost is trivial. Side effects: (a) Human output is now atomic per call rather than per line — minor improvement, no regression; (b) on a panic mid-render no partial Human output reaches stderr (old code could partial-print). I do not consider this a regression for the contract.

2. **`render_diagnostic_stream` error type leaks JSON-specifics.** Returning `Result<String, serde_json::Error>` from a function that handles three formats is a small abstraction leak — only the `Json` arm can fail. Acceptable: (a) the error is consumed locally in `render_diagnostics` and converted to a single user-facing message + exit code, and (b) introducing a wrapper enum would be a scope expansion. Worth noting only if the next slice needs to add another fallible renderer.

3. **`let _ = writeln!(output, …)` on `String`.** `writeln!` to `String` cannot fail (`fmt::Write` for `String` always returns `Ok`). The `let _ =` is harmless noise but inherited from the prior `writeln!(io::stderr(), …)` pattern in the surrounding code, so consistency wins. No action needed.

4. **Coverage gap on the non-`SIFR-` Human label branch** ([crates/sifr/src/main.rs:410-418](crates/sifr/src/main.rs:410)). The new test only uses `SIFR-TYPE-…` codes, so the production fallback to severity-based labels (`error`/`warning`/`note`) for non-SIFR codes is not exercised by this test. The `legacy_diagnostic_display` helper used by the assertion always calls `diagnostic_label_for_code_str` regardless of prefix, which means the assertion would *miss* a regression where a future change diverged the two branches for a non-`SIFR-` code. Out of scope for "render stream contract" (label selection is a separate concern), but flag for the taxonomy slice that follows.

5. **Empty-success JSON destination is asymmetric, pre-existing.** [cmd_check:526](crates/sifr/src/main.rs:526) writes `"[]"` to **stdout** when there are no diagnostics, while populated JSON goes to **stderr** through `render_diagnostics`. Not introduced by this slice; preserved exactly. If a later slice tightens the contract to "JSON always goes to stdout" (or "always to stderr"), this is the line to revisit. Mentioned for awareness only.

## 5. Validation

User-reported gates: `cargo fmt --check`, `git diff --check`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy -p sifr -- -D warnings`, `python3 scripts/run_verification_hardening.py --suite diagnostics`, `scripts/run_all_tests.sh --profile quick` — all passed. Not re-run during review per instructions.

## 6. Recommendation

**Approve and merge as-is.** The slice cleanly establishes the single-stream rendering contract, removes nothing user-visible, and is covered by a focused unit test that pins the contract structurally. The four non-blocking observations are notes for future slices, not changes to make here.
