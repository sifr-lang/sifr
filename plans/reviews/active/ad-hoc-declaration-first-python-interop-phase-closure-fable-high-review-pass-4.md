All focused test modules pass at HEAD (71/71 lowering python-interop, 17/17 codegen python-context). Status so far, while the last fresh-eyes diff audit finishes:

- **Pass-2 closure table**: NB-1, NM-1, NM-3, NM-4 and AM-1 through AM-10 all verified CLOSED at HEAD (with a handful of small residuals I'll enumerate); **NM-2 is PARTIALLY CLOSED** — `task_scope_offload_calls.rs:286-298` (scope/TaskGroup `spawn_cpu`, call site line 93) and `task_join_set_calls.rs:466-477` still synthesize `WorkerRuntimeError`/`WorkerError` with `identity: None`.
- **NF-1 confirmed live on five surfaces**: wrapped python-rooted decorators (`@python(math.sqrt)()`, `@python(...).extra`, `@python.opaque(...)()`) are silently discarded on module functions, opaque methods, classes, nested defs — `sifr check` prints "no errors found" and the Sifr body compiles instead of the Python binding.
- **New live-reproduced minor**: a user class named `RuntimeFault` is misclassified as a runtime fault at Python context exits (`outcome.rs:13-17`, `sync.rs:743-746`) — the same shadow bug just fixed for `CancellationError`/`TimeoutError`.

I'll deliver the full itemized report once the final audit agent returns.
