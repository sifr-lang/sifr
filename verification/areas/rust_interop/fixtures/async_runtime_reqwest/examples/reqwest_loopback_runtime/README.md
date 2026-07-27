# Reqwest loopback runtime

Locked, offline package scenario for generated asynchronous Rust bridge glue.

The safe package-local bridge binds an ephemeral `127.0.0.1` listener before
spawning its single-request HTTP task. Client, accept, read, write, and join
work is bounded, and the reqwest client disables ambient proxies so loopback
traffic cannot leave the process. Two requests prove that a borrowed Sifr input
crosses reqwest on the generated current-thread Tokio runtime and that the same
runtime ID and thread are reused. A delayed third request is cancelled by a
Sifr timeout; request and server-task drop guards report zero active work only
after the aborted task future has actually unwound.
The manifest records the exact pinned `ring` build-script and native-link
evidence required by reqwest's Rustls feature graph.

The paired negative bridge constructs a nested Tokio runtime and calls
`block_on`; package checking rejects it with `SIFR-RUST-ASYNC-0001` before the
bridge can run.
