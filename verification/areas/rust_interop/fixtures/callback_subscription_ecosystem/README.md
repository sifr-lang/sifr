# callback_subscription_ecosystem

This fixture family tracks runtime-observed subscription callbacks for
`tokio-tungstenite`, Redis pub/sub, and filesystem notification workflows.

- Positive evidence: `subscription_cancel_shutdown` executes a locked package
  against real loopback WebSocket and Redis Pub/Sub transports plus a real
  `notify` watcher. Its package queue consumes the callback's carried policy
  and observes bounded overflow, handler errors, stable panic redaction, a
  foreign-thread callback, close-time drain shutdown, cancellation of a
  scheduled delivery before invocation, and zero leaked
  tasks/watchers/resources.
- Negative evidence: `invalid_thread_capture_rejected` proves a nested handler
  retaining `NonSend` state is rejected with `SIFR-RUST-CB-0001` before Cargo
  probing.
- Compatibility category: `supported-through-bridge`. Ecosystem callbacks use
  an owned typed bridge carrying the exact declared queue and shutdown policy;
  package code remains responsible for its protocol-specific queue and cleanup
  handles.
