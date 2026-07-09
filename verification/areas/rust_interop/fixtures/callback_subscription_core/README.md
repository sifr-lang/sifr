# callback_subscription_core

This fixture family tracks stdlib-owned callback subscription mechanics needed
by signal-style subscriptions.

- Positive evidence: `signal_subscription_cancel_shutdown` exercises a
  thread-safe callback subscription that returns an async-close opaque handle
  and observes deterministic cancellation/shutdown through the declared close
  contract.
- Negative evidence: `invalid_subscription_callback_policy_rejected` proves
  subscription callbacks must declare bounded backpressure, overflow, and
  shutdown policy before they can cross the callback boundary.
- Compatibility category: `supported`. Ecosystem subscription crates remain in
  `callback_subscription_ecosystem` and are not claimed by this core row.
