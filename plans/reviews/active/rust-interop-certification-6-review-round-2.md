# Rust interop certification 6 review — round 2

Date: 2026-07-28

Reviewer: Claude Opus 5 (`--effort medium`)

Reviewed head: `1afcea31c`

Verdict: **NOT SATISFIED**

The reviewer verified that all eight round-1 findings were resolved and that
the focused validation, mandatory generated-build evidence, inventories, and
runtime claims were accurate.

## New findings

1. **High:** a directly declared nested retained-callback handler with an
   otherwise valid capture received `Send + Sync + 'static` bounds but still
   emitted as a borrowing closure, so `check` passed and `build` failed with a
   raw Rust `E0373` lifetime error.
2. **Medium:** capture validation inspected only direct captures. A retained
   handler could capture a sibling nested function that itself captured
   `NonSend` state, bypassing `SIFR-RUST-CB-0001`.
3. **Medium:** imported retained-callback declarations did not transport
   callable-parameter metadata through external module definitions, so
   cross-module attachment checks failed open.
4. **Low:** retained-callback abort-strategy enforcement ran only after a bridge
   signature contract was found, even though the unwind requirement is
   independent of signature resolution.

## Required outcome

Emit valid nested retained handlers as move closures, traverse nested callable
capture dependencies, transport retained-callback parameter metadata across
modules, move abort enforcement before signature lookup, add regression
coverage, validate, and run another exact Opus review.
