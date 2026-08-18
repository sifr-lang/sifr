# Method slot table

This package-neutral fixture proves compiler-emitted, static-program-indexed
method dispatch selected by package-owned method descriptors, with a
caller-owned mutable context. The Rust core sees only the generic context
parameter and checked structural input/output channels.
