Review complete. Output written to `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-16-review-pass-2.md`.

**Summary:**

| File | Verdict |
|------|---------|
| `demos/logging_and_timers/idiomatic.rs` | **APPROVE** — compiles, runs clean, no bugs |
| `demos/config_json_csv/idiomatic.rs` | **APPROVE** — uses `serde_json` (by design, workspace context), all assertions sound |
| `demos/collections_and_argparse/idiomatic.rs` | **APPROVE** — compiles, runs, all assertions pass |

**One latent bug noted** (pre-existing, not demo-breaking): In `collections_and_argparse/idiomatic.rs:183`, `args[index + 1]` is accessed without bounds checking. If a `store` action option is the last argument, it panics. The current demo passes well-formed args only, so this bug is dormant.

**No pass-1 regressions.** All three files are clean.
