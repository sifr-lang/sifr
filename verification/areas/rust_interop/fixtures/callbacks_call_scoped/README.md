# callbacks_call_scoped

This fixture family tracks call-scoped callback storage and invocation behavior.

- Positive evidence: `callback_valid_during_call` remains planned for a
  runtime fixture proving a callback can be invoked only during the Rust call.
- Negative evidence: `callback_storage_rejected` remains planned for a fixture
  proving storage or use-after-return is rejected before a Sifr package lists
  call-scoped callback behavior as verified support.
- Compatibility category: `future-owned-by-separate-phase`. Thread-safe
  callback policy declarations are verified, but call-scoped runtime behavior is
  not listed as verified support.
