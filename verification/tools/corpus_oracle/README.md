# Corpus oracle

This verification-only library preserves the complete source-versus-metadata
emission oracle formerly implemented inside the driver ignored test. The same
named test calls the shared function.

Package preparation reuses the normal driver libtest and builds one small oracle
executable alongside the versioned release CLI. Both are Cargo-selected artifacts
from already required compiler graphs. The oracle is never shipped.

The packaged consumer executes the exact normal ignored test, then the release
oracle, over every maintained fixture. It compares exact generated Rust, source
maps, module sets and fixture bytes. Module array ordering is ignored only because
the Rust authority is a HashSet; duplicate modules are rejected. Every other row
field retains exact comparison. Versioned portable metadata remains checked within
each source/metadata oracle; installed qualification checks the release payload.

When a complete profile selects sysroot_release:metadata-corpus, that suite owns
the normal ignored assertion. The native group reports the case as delegated,
not passed, and does not execute it again. Without that selected owner, native
selection executes the case normally. Emitted plans declare assertion ownership
and plan comparison checks it. A failed or blocked owner still fails the profile.

Preparation binds the libtest, release oracle and compiler, source snapshot,
content inputs, tool versions and build environment. Consumers reject missing or
stale preparation, execute fresh assertions without Cargo, and retain each attempt.
The normal libtest must report exactly the named test and one pass; zero or filtered
execution fails. The release fixture directory is explicit to respect path remapping.
