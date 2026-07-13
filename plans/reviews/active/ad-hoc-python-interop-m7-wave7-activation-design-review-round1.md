## Review Summary

I reviewed the M7 Wave 7 async declaration activation design against the locked contracts and current repository code.

**Design strengths**:
- Correctly scopes gate lifts to `@python.coroutine(path)` and `cleanup=async_close`; all other M8–M12 syntax stays reserved via existing `PYRES_UNIMPLEMENTED_DECLARATION` paths in `python_interop.rs`.
- Independent-CancelledError semantics (bridge-requested=false → PythonError) match the protocol architecture contract.
- The 60-second create-PR budget referenced in the design is real (`verification/profiles/create-pr.json:14`).

**Blocking gaps** (see ReportFindings above):
1. **Test harness incompatibility**: The current `async_declaration_tests` uses `std::thread::park`-based `block_on` (async_declaration_tests.rs:495), which cannot honor the "await pending → Tokio abort" mechanism. `semantic_async_close_uses_python_terminal_outcome_after_cancellation` would deadlock, yet the design keeps it in the Validation list without stating how it migrates.
2. **Undocumented new carrier API**: The "resume the bound Tokio abort fallback" operation is not expressible with the current `CancellationCarrier` surface (`bind_fallback`/`claim`/`request_cancel`); the design implies a new method but names no return type or idempotency contract.
3. **Conditional executable-evidence enforcement**: The compiled fixture is added to merge/release "if runtime cost can't fit" the create-PR budget, and pre-PR execution is a manual step — not automated by the create-PR lane that AGENTS.md treats as the merge gate mirror.
4. **Terminal.complete ↔ request_cancel race**: The design doesn't state which outcome wins when Sifr requests cancel between outcome storage and lease drop.
5. **Cancel-before-registration** still returns catchable `AsyncSubmissionCancelled` PythonError (async_runtime.rs:231-260) — contradicts "without exposing a catchable declaration error."
6. Missing coverage for independent-CancelledError classification and fallback-not-bound runtime-error case.
7. `async-close` is not a separate capability-ledger row; design language should say activation goes through the cleanup-evidence sub-kind of `coroutine-declaration`.
8. File-size guardrail headroom is thin (python_interop.rs at 858/900; async_runtime.rs at 812) — design should name submodule targets.

VERDICT: CHANGES_REQUIRED
