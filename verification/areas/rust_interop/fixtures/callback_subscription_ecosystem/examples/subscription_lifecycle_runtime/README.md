# subscription_lifecycle_runtime

Locked, offline runtime evidence for retained Rust callbacks across
`tokio-tungstenite`, Redis pub/sub, and `notify`.

The package binds only ephemeral loopback ports, watches a unique temporary
directory, contains callback panics, bounds all network operations and joins,
and removes every harness-owned resource during consuming async close.
