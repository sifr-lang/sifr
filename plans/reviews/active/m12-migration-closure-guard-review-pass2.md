## Findings (ordered by severity)

**1. Missing retired intrinsic: `encode_utf8` — MEDIUM**

`scripts/check_stdlib_migration_closure.py` (RETIRED_INTRINSICS set, lines ~19–150)

The tracker's PR log explicitly records M10 wave 11 (PR #2789) as: "*splitting `_sifr.bytes` so `encode_utf8` and `bytes_to_hex` move to private `sifr_stdlib::bytes` adapters*." The retired set includes `bytes_to_hex` but omits `encode_utf8`. If a future edit reintroduces `"encode_utf8" => lower_encode_utf8(args)` in `registry.rs`, this guard passes silently — defeating the wave's stated closure invariant for exactly one of the two names it names.

**2. Potentially missing wave 12 collections intrinsics — LOW/UNCERTAIN**

`scripts/check_stdlib_migration_closure.py`

M10 wave 12 (PR #2791) "*set helpers and legacy JSON-string `defaultdict_*` helpers move to private `sifr_stdlib::collections` adapters*." No `defaultdict_*` or set-helper names appear in RETIRED_INTRINSICS. If those helpers were codegen dispatch names (as `bytes_to_hex` was for the bytes wave), they should be retired here for symmetry. If they were only Rust-internal helpers without dispatch entries, this is a non-issue — but the diff gives no way to tell, and the pattern from wave 11 suggests coverage is inconsistent.

**3. Regex misses guarded / non-adjacent dispatch arms — LOW**

`scripts/check_stdlib_migration_closure.py:16` (`EXACT_INTRINSIC_RE`)

`r'"([A-Za-z0-9_]+)"\s*(?=\||=>)'` requires the arrow/pipe to follow the string literal (with only whitespace between). Patterns that would slip through: `"sqrt" if cfg!(...) =>` (guarded arm), and any `IntrinsicKind::Named("sqrt") =>` style enum-pattern dispatch. The seed test only exercises the canonical `"name" => lower_x(args),` shape. Guarded/enum patterns are unusual in intrinsic dispatchers so this is more a hardening note than a live bug — but the guard's advertised contract is "reintroduction in active dispatch," and both shapes qualify.

**4. Regex has narrow false-positive surface in comments/macros — LOW**

Same location. Any `"retired_name" =>` sequence anywhere in `registry.rs` — a doc comment example, an unrelated `matches!`-style macro arm, or a diagnostic template — would fire the guard even though nothing was reintroduced. Because the check is scoped to a single file and the retired list is specific, this is unlikely to trip in practice. Not blocking; worth an inline comment that the guard assumes `registry.rs` never mentions retired names outside live dispatch.

**5. Stale-phrase list is exact-substring only — INFORMATIONAL**

`scripts/check_stdlib_migration_closure.py:151–156` (STALE_ARCH_PHRASES)

Phrase-match will catch verbatim reversion of the three deleted sentences but not paraphrases ("the TOML ownership file is the canonical source"). That is an inherent limitation of the approach and matches the stated intent (prevent regression to the specific wording that was removed), so I'd accept it — noting only that this is the guard's ceiling, not a bug.

---

Scope, taxonomy, and wiring look correct: the architecture row now cleanly separates completed stateless/data leaves from retained runtime/resource/callback leaves; resource/runtime/callback surfaces are explicitly not migrated by this wave; `profile_runner.py:297–299` and `guardrails.json` wire the guard + self-test into core guardrails alongside the existing allowlist and certification-gate checks; deleted-file check and self-test coverage for each failure branch are sound.

VERDICT: NEEDS_CHANGES

Blocking item is finding #1 (`encode_utf8`). Finding #2 is worth confirming against `registry.rs` before merging — if wave 12 helpers had dispatch names, add them alongside `encode_utf8`; if not, the wave is closed as-is.
