# Crate-Backed View Runtime

This locked/offline package is the runtime-observed scenario for
`zero_copy_runtime_matrix`. It moves the owned `Vec<u8>` received by the bridge
into `bytes::Bytes`, retains an alias after the original owner binding is
released, seals an anonymous `memmap2` mapping without changing its
allocation, and reads the mutated mapped values through pointer-identical
`bytemuck` and `zerocopy` views. The generated opaque view
boundary also enforces the declared `Send + Sync` type obligations and records
deterministic release.

The package uses only safe Rust. Mutation happens before immutable aliases are
published, so the scenario's executable state transition mirrors the compiler
contract's exclusivity requirement. Its manifest explicitly trusts the build
script shipped by the direct `zerocopy` dependency.
