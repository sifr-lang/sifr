# Call-scoped callback runtime

Locked, offline package scenario for generated call-scoped callback glue.

The bridge invokes a borrowed callback during the Rust call. The generated
outer panic boundary contains both target and callback panics and exposes only
the redacted `Rust bridge panicked` message; assertion payloads and locations
inside callback code are intentionally not forwarded. The positive path also
compiles exact-integer, list, dictionary, optional, and multi-argument adapter
conversions in one generated package. The paired negative generated-build test
replaces the bridge source with storage, returned-deferred-call, and
unmanaged-thread escapes that rustc rejects before a package can run.
