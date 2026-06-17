# Hyper-Util Necessity

HTTP-transport capability enables `hyper-util` for the HTTP transport substrate because Hyper 1.x keeps Tokio
integration outside the core `hyper` crate. Sifr's async-network capability TCP streams and TLS capability TLS streams are
Tokio I/O types, while Hyper's client and server connection APIs expect Hyper runtime I/O
traits. `hyper_util::rt::TokioIo` is the maintained adapter between those ruless.

The HTTP/2 connection builder also needs a Hyper executor. `hyper_util::rt::TokioExecutor`
keeps that executor wiring on the upstream-supported path instead of introducing a local
adapter implementation inside `sifr_runtime`.

The Hyper-only alternative would require Sifr to implement the `hyper::rt::Read` and
`hyper::rt::Write` traits for each accepted transport shape, including async-network capability TCP streams and
TLS capability TLS streams, and to supply an executor compatible with Hyper's HTTP/2 connection
driver. That local adapter would duplicate the maintained Tokio bridge that Hyper keeps
outside the core crate. HTTP-transport capability therefore uses `TokioIo` and `TokioExecutor` directly and does
not enable Hyper-Util client pooling, proxy, auto-server, or graceful-shutdown features.

The dependency is constrained to:

```toml
hyper-util = { version = "0.1.20", default-features = false, features = ["tokio"] }
```

No public Sifr type, request lifecycle rules, or shutdown policy exposes `hyper-util`.
It is an implementation detail of the internal HTTP transport harness.
