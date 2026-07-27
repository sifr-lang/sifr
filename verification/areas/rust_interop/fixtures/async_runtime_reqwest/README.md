# async_runtime_reqwest

This fixture family tracks runtime-observed async Rust bridge behavior on the
Sifr Tokio runtime with `tokio` and `reqwest`.

- Positive evidence: `async_reqwest_loopback` is executed by
  `test_build_async_reqwest_loopback_runtime`. The locked package binds an
  ephemeral loopback server before spawning it, executes two borrowed-input
  requests on one generated current-thread Tokio runtime, cancels a delayed
  request through a Sifr timeout, and observes zero active request/server work
  after bounded cleanup.
- Negative evidence: `hidden_block_on_rejected` is checked by
  `test_check_async_reqwest_hidden_blocking_rejected`. The package-local async
  bridge package is rejected with `SIFR-RUST-ASYNC-0001` when ordinary source
  under `src/` constructs a Tokio runtime, calls `block_on`, or invokes another
  recognized blocking runtime operation. Same-file aliases and re-exports are
  resolved independent of declaration order; cross-file re-exports reached
  only through an unresolved glob remain governed by the package trust
  contract. Both evidence tests are blocking bindings in the
  `sifr_driver_generated_builds` suite at the `merge` profile.
- Compatibility category: `supported-through-bridge`. The claim is limited to
  the runtime-observed package-local bridge behavior above.
