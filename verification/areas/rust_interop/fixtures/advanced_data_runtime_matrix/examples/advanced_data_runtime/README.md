# Advanced Data Runtime

This locked/offline package is the runtime-observed scenario for
`advanced_data_runtime_matrix`. Two shared bridge crates use the exact root
lock versions of Arrow, DataFusion, Polars, ndarray, and CPU-only Candle.

The Arrow path moves the owned Sifr floating-point vector into an Arrow array without
changing its address, preserves schema identity in a record batch registered
with DataFusion, and derives the corresponding Polars dataframe through an
explicit copy. The tensor path moves two owned vectors into ndarray and Candle
without changing either allocation, then transfers the ndarray owner into a
one-shot, DLPack-style managed capsule without copying. Pre-close and
post-close observations prove deterministic owner cleanup.

The bridge implementation uses only safe Rust. The managed capsule models the
shared bridge's DLPack ownership and metadata contract without exposing an
unsafe C ABI to generated package code. The package manifest names the exact
seven-entry native-link envelope covering the locked graph on arm64 and
x86_64; Sifr's post-build audit rejects any emitted build-script link output
outside that envelope.
