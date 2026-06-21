# callback_subscription_matrix

This fixture family tracks runtime-observed subscription callbacks for
`tokio-tungstenite`, Redis pub/sub, and filesystem notification workflows.

- Positive evidence: `subscription_cancel_shutdown` remains planned for a
  runtime fixture that observes cancellation handles and deterministic shutdown.
- Negative evidence: `invalid_thread_capture_rejected` remains planned for a
  fixture proving non-send captures and invalid thread-affinity captures cannot
  cross the declared callback boundary.
- Compatibility category: `future-owned-by-separate-phase`. Contract-level
  callback policy validation is verified by `callbacks_threadsafe`; runtime
  subscription certification is not listed as verified support.
