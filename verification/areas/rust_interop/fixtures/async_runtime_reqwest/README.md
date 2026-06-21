# async_runtime_reqwest

This fixture family tracks runtime-observed async Rust bridge behavior on the
Sifr Tokio runtime with `tokio` and `reqwest`.

- Positive evidence: `async_reqwest_loopback` remains planned for a loopback
  HTTP fixture that observes request execution, cancellation, and shutdown.
- Negative evidence: `hidden_block_on_rejected` remains planned for a fixture
  proving hidden runtime creation and hidden `block_on` are rejected in the
  generated runtime path.
- Compatibility category: `future-owned-by-separate-phase`. Async contract
  validation and signature probing are verified; loopback reqwest runtime
  certification is not listed as verified support.
