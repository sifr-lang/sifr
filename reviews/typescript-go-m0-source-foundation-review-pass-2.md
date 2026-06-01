# Follow-up Review: M0 Pass-1 Blocker Area

**Verdict: M0 is approved for PR.** The blocker is resolved cleanly with no fallout.

## Blocker resolution

`crates/sifr_frontend/src/source_maps.rs:87` — signature is now `position: &TextPosition`. All three call sites updated:
- `source_maps.rs:144` (`&position`)
- `source_maps.rs:166-173, 177-184` (inline `&TextPosition {...}` for the unregistered-file and invalid-boundary tests)
- `frontend_query_bench.rs:271` (`&target`) — also folds in the prior review's optional "extract `let target`" readability note

## Validation I re-ran (independent confirmation)

| Check | Result |
|---|---|
| `cargo clippy --workspace -- -D warnings` | PASS — clean `Finished` |
| `cargo test -p sifr_frontend` | PASS — 5/5 (both `source_maps` tests included) |
| `scripts/run_all_tests.sh --profile quick` | PASS — 67 e2e, validation lane report clean |
| `git diff --check` | PASS |
| `cargo fmt --check` | PASS |

## Fallout check

- **No unintended callers broke.** `text_position_to_span` is referenced only in `source_maps.rs` (definition + 3 tests) and `frontend_query_bench.rs:271`. Both updated. The remaining grep hits are in `reviews/`, `issues/`, `internal_docs/phases/35_*` (historical / forward-looking prose).
- **Public API change is intentional and consistent.** The prior review explicitly called this out as a consequence and acceptable ("Pick the one the project prefers for SourceMapView ergonomics"). The project chose `&TextPosition`; everywhere that builds one now does.
- **Bench upgrade is a positive, not a regression.** The old `let _ = source_map.text_position_to_span(...)` discarded the result. The new version asserts a real UTF-8 round trip (`.span_to_text_range` → `round_trip.start == target` etc.). This is the M0 contract that pass-1 said the bench should be exercising; it now does.
- **Pre-existing inconsistency, not fallout.** The bench uses `PositionEncoding::UTF8` (uppercase) while the tests use `PositionEncoding::Utf8`. Both compile (Rust accepts the all-caps form as a valid identifier) and this discrepancy pre-dates the M0 fix — flagging only so the team is aware; it is not in the blocker area and not required to be fixed here.

## Recommendation

Open the PR. The local validation gate that mirrors CI is green.
