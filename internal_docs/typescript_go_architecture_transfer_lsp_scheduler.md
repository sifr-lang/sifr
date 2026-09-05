# TypeScript-Go Architecture Transfer: LSP Scheduler Queues

LSP scheduler makes LSP request scheduling concrete while deliberately keeping execution
serialized until LSP scheduler adds cancellation tokens, progress, and worker execution.
`sifr_lsp::RequestQueue` now stores FIFO queues per lane:

- latency-sensitive
- formatting
- workspace
- background

The scheduler prefers latency-sensitive work but runs a bounded fairness pass
after a fixed interval so workspace and background work cannot starve forever.
Formatting has its own lane, so large workspace requests cannot sit ahead of
formatting or hover/completion-style work.

Diagnostics publication now flows through debounced jobs keyed by document URI.
Each scheduled diagnostic job captures the current document version; publication
checks that version both before and after analysis, then skips stale jobs instead
of publishing superseded diagnostics. Re-scheduling a document refreshes the
captured version while preserving that document's original queue slot, and
pending diagnostic jobs are cleared when diagnostics are disabled.

Current limitations:

- request execution remains serialized in the stdio loop
- LSP scheduler still owns cancellation tokens, progress, delayed progress, worker loops,
  and parent-process watchdog behavior
- request bodies are retained in the stdio server until dispatch; if LSP scheduler makes
  queued cancellation reachable across async worker turns, it must remove the
  matching retained body when a queued request is cancelled
- background index work is represented by a scheduler lane and fairness tests,
  but no background worker is started in LSP scheduler
