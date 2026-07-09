# callback_subscription_ecosystem

This fixture family tracks runtime-observed subscription callbacks for
`tokio-tungstenite`, Redis pub/sub, and filesystem notification workflows.

- Positive evidence: `subscription_cancel_shutdown` remains planned for a
  runtime fixture that observes cancellation handles and deterministic shutdown.
- Negative evidence: `invalid_thread_capture_rejected` remains planned for a
  fixture proving non-send captures and invalid thread-affinity captures cannot
  cross the declared callback boundary.
- Compatibility category: `future-owned-by-separate-phase`. Stdlib-owned signal
  subscription mechanics are verified by `callback_subscription_core`;
  ecosystem callback subscription certification is not listed as verified
  support.
