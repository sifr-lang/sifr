# subscription_lifecycle_runtime

Locked, offline runtime evidence for retained Rust callbacks across
`tokio-tungstenite`, Redis pub/sub, and `notify`.

The package binds only ephemeral loopback ports, watches a unique temporary
directory, contains callback panics, bounds all network operations and joins,
derives queue capacity, overflow, and shutdown behavior from the callback's
carried policy, drains pending delivery during consuming async close, cancels
a scheduled callback before invocation, and removes every harness-owned
resource.
