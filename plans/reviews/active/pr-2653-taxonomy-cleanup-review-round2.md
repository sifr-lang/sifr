I have what I need. Reproducing the scanner once more confirms it passes; my deeper sweep surfaced a few stragglers the scanner regexes don't catch.

## Code review — PR #2653 (round 2)

### Findings, ordered by severity

**1. BLOCKER — Active compiler source still references milestone 6 by tag.**
`crates/sifr_frontend/src/source_provider.rs:218`:
```rust
// TODO(m6): surface the dirty directory when watcher invalidation
// starts consuming overlay-backed directory dependencies.
```
The `(m6)` tag is "milestone 6" — independently confirmed by historical review material naming M6 as the "consume tracked records for dirty-scope" owner. The scanner does not catch it because `\bM\d+\b` is case-sensitive and `m6)` has no `[_-]` separator. This is the exact category of leak the PR is meant to remove, sitting in the active `crates/` surface. Recommended fix: rephrase to either drop the tag (`// TODO: surface the dirty directory …`) or replace with a compiler-behavior reference (e.g., "TODO: when the tracked-dependency consumer lands"). No allowlist needed.

**2. BLOCKER — `demos/codegen_preamble` left the "m14" milestone tag inside string literals.**
The PR cleaned the comment headers (`milestone_codegen_preamble_migration` → `codegen_preamble_migration`) and the path (`sifr_m14_preamble_demo.txt` → `sifr_codegen_preamble_demo.txt`) but left these data strings:
- `demos/codegen_preamble/main.sifr:14` — `write_text(path, "m14 preamble")`
- `demos/codegen_preamble/main.sifr:19` — `assert str(...) == "file = m14 preamble"`
- `demos/codegen_preamble/main.sifr:24` — `log: Logger = getLogger("m14")`
- Mirrored at `demos/codegen_preamble/emitted.rs:969,1019,1027`
- Mirrored at `demos/codegen_preamble/idiomatic.rs:38,43,48`

The original demo header explicitly tied "m14" to "milestone_codegen_preamble_migration", so these are leftover milestone-14 tags. The values are arbitrary roundtrip test data — changing "m14" to e.g. "codegen" (and regenerating `emitted.rs`/`idiomatic.rs`) is purely cosmetic.

**3. NIT (not blocking) — Lingering single-letter "m{n}" tags in HTTP fixtures.**
Likely milestone-4 remnants the rename pass didn't touch; values are arbitrary test sentinels, so cleanup is a one-line search/replace whenever convenient:
- `crates/sifr/tests/e2e/pass/network_http_http1_loopback.sifr` — request path `/m4/http1`
- `crates/sifr/tests/e2e/pass/network_http_http2_loopback.sifr` — `/m4/h2c`
- `crates/sifr/tests/e2e/pass/network_http_https_h2_loopback.sifr` — `/m4/https-h2`
- `crates/sifr/tests/e2e/pass/network_http_header_cookie.sifr:27` — header `("x-sifr", "m3")`

### Remaining keyword leaks outside .cursor/**, AGENTS.md, and archived plans/

The two blockers above are the only confirmed leaks outside excluded surfaces. README.md, `internal_docs/**`, `docs/**`, `.github/workflows/**`, `scripts/**`, `lib/**`, the verification runner code, and the renamed reports are all clean. The bare `backlog` tokens that show up in `lib/sifr/net.sifr`, `crates/sifr_runtime/src/net.rs`, etc. are TCP listener-backlog parameters — legitimate compiler/runtime vocabulary, correctly excluded by the scanner's contextual patterns. Submodule SHA bumps (editor_integrations 7e972f2 → d03a2f4) are clean pointer updates.

### Validation concerns

- The scanner reproduces clean on this branch (`verification taxonomy ok …`) and the self-test passes. The two blockers above demonstrate that the regex set has two real blind spots worth tightening once they're cleaned:
  1. Lowercase `m\d+` inside parens / punctuation (e.g., `TODO(m6):`).
  2. Bare `"m\d+"` and `"m\d+ <word>"` quoted string tokens.
   These are out of scope for *this* PR's verdict — but if the goal is durable enforcement, the scanner should grow patterns to catch them, otherwise the same leak class will reappear.
- `scripts/run_all_tests.sh --profile create-pr` is the right gate; the full merge profile's perf-p95 variance is a known flake unrelated to this work.

### What needs to change before merge

1. Remove or rephrase the `TODO(m6)` tag in `crates/sifr_frontend/src/source_provider.rs:218`.
2. Replace the "m14" string literals / logger name in `demos/codegen_preamble/{main.sifr,emitted.rs,idiomatic.rs}` with a neutral token (e.g., "codegen") so the demo no longer carries the milestone-14 tag.
3. Re-run the scanner and the create-pr profile; both should remain green.

Verdict: BLOCKED
