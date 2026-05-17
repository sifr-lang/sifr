

**Review: Post-Fix Delta — m36.6 Follow-Up**

**Delta analyzed:**

`crates/sifr_lsp/src/request_queue.rs:23` — `pub(crate) fn cancel(...)` renamed to `pub(crate) fn remove_pending(...)`

`crates/sifr_lsp/src/session.rs:54` — `self.queue.cancel(...)` updated to `self.queue.remove_pending(...)`

**Assessment:**

The rename is purely mechanical. `Session::cancel_request` still calls the queue's removal operation, still logs `"cancelled request"`, and the method's signature and callers are unchanged. No behavior change — the rename only removes the `.cancel(...)` call site from `sifr_lsp`, satisfying the guardrail that reserves `.cancel(...)` for the diagnostics model (`crates/sifr_diagnostics/src/model/mod.rs`).

- m36.5 LSP request-queue behavior is preserved: `start`, `finish`, `remove_pending` (was `cancel`), `begin_shutdown` all remain.
- m36.6 contract is unaffected: editor assets (Neovim/Zed/Helix/Emacs configs, TextMate grammars, syntax drift checks) were approved in pass-1 and are untouched.
- Validation evidence in `phase36-developer-tooling-execution.md` correctly reflects the fix and the passing rerun: `wall_time=2305.22s`, `e2e cache_hits=0/12`, `report_signature=f808284595f17a99`.

No blocking issues. The change is a focused guardrail compliance fix with no functional impact.

**SATISFIED**
