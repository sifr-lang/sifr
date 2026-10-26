# Corpus oracle

This verification-only library preserves the complete source-versus-metadata
emission oracle formerly implemented inside the driver ignored test. The same
test remains selected and calls the shared function.

Package preparation builds a small oracle executable alongside each already
required CLI: the normal development graph and the versioned release graph.
It uses Cargo-selected development artifacts and the canonical release builder.
The oracle is never shipped in a release archive.

The packaged consumer runs both profiles over every maintained fixture and
compares exact generated Rust, source maps, module sets and fixture bytes.
Module array ordering is ignored only because the Rust authority is a HashSet;
duplicate modules are rejected. Every other row field retains exact comparison.
Portable metadata identities contain versioned inputs and are checked within each
source/metadata oracle; installed qualification checks the release payload.

Preparation binds both executables and compilers, source snapshot, content inputs,
tool versions and build environment. Consumers reject missing/stale preparation,
execute new assertions without Cargo and retain each attempt's output directory.
The fixture directory is explicit so release path remapping cannot embed a
nonexistent checkout path into the oracle.
