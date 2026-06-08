The pass-1 polish items are all visibly applied:

1. **Pass fixture host-independence** — `signal_constants_strsignal.sifr:13-17` no longer asserts `terminate_signal.supported`, only checks number/name/strsignal mapping which are host-independent constants.
2. **Fail-fixture markers** — All 5 fail fixtures now declare `# expect-error: SIFR-NAME-0004` + `# expect-error: SIFR-NAME-0002`, making the unsupported-API contract machine-enforced rather than narrative-only.
3. **Non-Unix terminate wording** — `signal.rs:44` now reads `"SIGTERM is unsupported on this host"` (no longer the misleading "shutdown stream" phrase for a single-shot await).
4. **Codegen test alignment** — `registry_core_tests.rs:294` asserts on the new substring `"SIGTERM is unsupported"`.
5. **Execution ledger** — line 569 records both the pass-1 review outcome and the post-polish revalidation runs.

The substantive review conclusions from pass-1 still hold:
- Scope honesty: deferrals still enumerated; no new claims introduced by the polish.
- Async surface shape: unchanged.
- Host honesty: improved (pass fixture no longer implicitly Unix-only; non-Unix message no longer claims "stream" semantics for a one-shot).
- Tokio feature gating, panic discipline, coverage: unchanged; codegen test still pinned and now matches the corrected substring.

No new blocker is introduced; one of the three non-blocking items from pass-1 (Windows-implicit pass fixture; fail-fixture markers; terminate wording) was addressed and the others were not regressions.

RESULT: PASS
