# callbacks_call_scoped

This fixture family certifies generated call-scoped callback storage and
invocation behavior.

- Positive evidence: `callback_valid_during_call` executes generated glue in
  the locked `call_scoped_callback_runtime` package, observes invocation and
  ordinary callback errors, and maps a callback panic through the enclosing
  redacted Rust panic boundary.
- Negative evidence: `callback_storage_rejected` runs storage, returned
  deferred-call, and unmanaged-thread bridge variants. Each Cargo probe is
  pinned to the concrete rustc lifetime or thread-trait failure and reports
  `SIFR-RUST-CB-0001`.
- A paired ordinary signature mismatch remains `SIFR-RUST-TYPE-0001`, proving
  callback diagnostics are reserved for concrete lifetime/thread escape.
- The runtime bridge owns no callback. It borrows the generated adapter for the
  duration of the Rust call and is deliberately neither `Send` nor `Sync`, so
  storage, use-after-return, and unmanaged-thread movement remain rustc errors.
- `@rust.callback(...)` continues to describe the separate thread-safe
  subscription contract and does not widen this call-scoped certification.
- A call-scoped declaration must stay synchronous and expose a distinct
  ordinary error plus `RustPanicError`; `trusted_no_panic` and `panic=abort`
  do not cover panics originating in Sifr callback code.
