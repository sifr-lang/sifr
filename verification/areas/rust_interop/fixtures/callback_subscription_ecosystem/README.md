# callback_subscription_ecosystem

This fixture family tracks runtime-observed subscription callbacks for
`tokio-tungstenite`, Redis pub/sub, and filesystem notification workflows.

- Positive evidence: `subscription_cancel_shutdown` executes a locked package
  against real loopback WebSocket and Redis Pub/Sub transports plus a real
  `notify` watcher. Its package queue consumes the callback's carried policy
  and observes bounded overflow, handler errors, stable panic redaction, a
  foreign-thread callback, close-time drain shutdown, cancellation of a
  scheduled delivery before invocation, and zero leaked
  tasks/watchers/resources. Its retained handlers include attribute- and
  method-derived `str` captures, a declaration-time non-`Copy` snapshot whose
  enclosing binding is rebound and used after attachment, and a loop-local
  attachment. The generated package therefore proves capture-type fidelity,
  the snapshot contract, isolated capture cloning, and owning `move`-closure
  emission through rustc.
- Negative evidence: `invalid_thread_capture_rejected` proves both a nested
  handler retaining `NonSend` state and one retaining a callable with unknown
  captures are rejected with `SIFR-RUST-CB-0001`, while second attachment of a
  consumed nested handler is rejected with `SIFR-OWN-0001`. Direct and
  sibling-transitive capture mutation are also rejected with
  `SIFR-RUST-CB-0001` because the retained bridge requires `Fn`, all before
  Cargo probing.
- Compatibility category: `supported-through-bridge`. Ecosystem callbacks use
  an owned typed bridge carrying the exact declared queue and shutdown policy;
  package code remains responsible for its protocol-specific queue and cleanup
  handles.
