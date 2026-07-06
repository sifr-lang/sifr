Reviewing this diff without touching the repo.

## Findings

### 1. Regex misses guarded arms whose guard expression contains `==`

`scripts/check_stdlib_migration_closure.py:22`
```python
EXACT_INTRINSIC_RE = re.compile(
    r'(?:"([A-Za-z0-9_]+)"|Named\("([A-Za-z0-9_]+)"\))\s*(?:if\b[^=]*)?(?=\||=>)'
)
```

The optional guard clause is `if\b[^=]*` — greedy up to the first `=`. Because `[^=]*` stops before any `=`, and the trailing lookahead is `(?=\||=>)`, a guard containing an equality operator terminates the match before `=>` is reached.

Failure scenario: someone reintroduces
```rust
"sqrt" if kind == Kind::Real => lower_sqrt(args),
```
The regex tries `[^=]*` up to `kind `, then the lookahead needs `\|` or `=>` at position `==` and fails. `\s*` and the optional group backtrack to zero-width, but the lookahead position after the string literal is at ` `, and after `\s*` is at `i` — none satisfy `(?=\||=>)`. Result: the arm is not classified as an active dispatcher, and a retired name slips through the guard.

The self-test only exercises a `cfg!(test)` guard, which contains no `=`, so this hole is uncovered. Consider making the guard clause tolerant of `==`, e.g. by consuming up to `=>` with a lazy match plus explicit exclusion, or by scanning line-by-line and stripping the pattern before `=>` differently. At minimum, extend `_self_test` with a `"foo" if x == 1 =>` case so any regression is loud.

### 2. Lookahead accepts `||`, producing false positives on boolean-or

`scripts/check_stdlib_migration_closure.py:22`

`(?=\||=>)` matches wherever a `|` character follows, including the first `|` of a `||` boolean-or operator. Any non-match code like
```rust
if name == "sqrt" || other() { … }
```
would flag `sqrt` as an "active retired intrinsic" even though the code is not a match arm. In practice `registry.rs` is a large `match` and unlikely to have such expressions, so the false-positive risk is low — but the lookahead should be tightened to `(?=\|[^|]|=>)` (or `(?=\s*\||=>)` after asserting the next `|` is not doubled) so the guard's meaning matches its intent.

### 3. Retired coverage vs. the wave log — unverifiable from the diff alone

The PR log narrates migrations of `_sifr.platform` (M9 wave 1), `_sifr.datetime` (M10 wave 10 — only `datetime_format`, `datetime_from_timestamp`, `datetime_now`, `datetime_now_struct` are listed), and `_sifr.compress`/`_sifr.gzip`/`_sifr.zipfile` (M10 wave 9), among others. The retired set includes no `platform_*` entries and no additional datetime helpers (e.g. any `datetime_parse`/`datetime_utcnow`-style names). The narrative for wave 9 mentions "gzip and zipfile adapters", but only `gzip_compress`/`_gzip_compress_bytes_impl`/`gzip_decompress`/`_gzip_decompress_bytes_impl` are in the retired list — no gzip level/options helpers.

I cannot verify from the diff whether these are real omissions or simply not intrinsic-dispatch names. Flagging as **PLAUSIBLE** — worth cross-checking the wave PRs' removed match arms against `RETIRED_INTRINSICS` before locking this in as the closure invariant.

### 4. Minor: `STALE_ARCH_PHRASES` is fragile against harmless rewording

`scripts/check_stdlib_migration_closure.py:216-220`

Each phrase must appear verbatim. If a future edit softens `"old handwritten intrinsic registry is removed or reduced"` to `"legacy handwritten intrinsic registry has been removed or reduced"` the guard silently stops catching regressions. Consider matching a shorter, more essential fragment (e.g. `"remains the TOML registry"`, `"compiler intrinsic registry"`) or using a regex per phrase. Not a correctness bug in the current diff — just a durability observation.

### 5. Guardrails wiring: correct

`verification/policy/guardrails.json:59` and `verification/runner/sifr_verify/profile_runner.py:297` both add the new script and self-test invocation in the same section as the retained-allowlist and resource-certification guards. Wiring, args, timeout, and self-test invocation are consistent with the neighboring entries.

### 6. Docs taxonomy: aligned with stated scope

The arch-doc rewrite correctly:
- keeps the resource/runtime/callback rows retained pending Rust interop certification,
- reframes the "surface-by-surface ownership decision" cell as the retained-glue TOML + closure guard pair (no more TOML ownership registry as source of truth),
- introduces "retained-glue allowlist" as a blocker for the private-declaration row, matching the sibling `check_stdlib_native_intrinsic_allowlist.py` guard.

No taxonomy drift into resource-migration territory, which matches the stated scope of "no resource migration in this wave."

---

**VERDICT: PASS**

Findings 1 and 2 are regex sharpness issues worth tightening opportunistically (and finding 1 in particular should get a self-test that would fail today), but neither invalidates the current guard: `registry.rs` isn't expected to contain either `==`-in-guard arms or `"name" || …` scalar comparisons, so the guard behaves correctly on the code it's protecting today. Finding 3 is a plausibility check, not a diff defect. Scope, wiring, and doc taxonomy are correct for M12 wave 4.
