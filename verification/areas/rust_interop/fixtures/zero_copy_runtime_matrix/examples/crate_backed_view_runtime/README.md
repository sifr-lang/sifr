# Crate-Backed View Runtime

This locked/offline package is the runtime-observed scenario for
`zero_copy_runtime_matrix`. It moves a Sifr `bytes` owner into `bytes::Bytes`,
retains an alias after the original owner binding is released, seals an
anonymous `memmap2` mapping without changing its allocation, and observes
pointer-identical `bytemuck` and `zerocopy` views. The generated opaque view
boundary also enforces the declared `Send + Sync` type obligations and records
deterministic release.

The package uses only safe Rust. Mutation happens before immutable aliases are
published, so the scenario's executable state transition mirrors the compiler
contract's exclusivity requirement.
