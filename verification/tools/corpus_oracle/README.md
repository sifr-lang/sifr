# Corpus oracle

This verification-only crate owns the maintained source-versus-metadata
comparison. The driver’s ignored `full_corpus_exact_emission` test calls its
library under the normal test profile. The installed sysroot verification area
calls its executable, linked against the canonical packaged-version release
libraries. Neither path uses the installed CLI to manufacture its own reference.

The executable requires an explicit fixture directory. It compares all maintained
fixtures unless the existing diagnostic selection environment variable is set.
The canonical metadata-corpus runner removes that selection before execution.
Native runtime assertions remain in the E2E and installed qualification runners.

Prepare through `verification/areas/sysroot_release/package_build.py`. Its
`--corpus-only` mode follows the canonical release-compiler prerequisite;
`--run-corpus` rejects missing or stale preparation and never invokes Cargo.
The preparation receipt binds the source inventory, compiler/oracle bytes,
versioned source snapshot, tool versions and build-affecting environment.
Changing those inputs requires preparation again. It is not a stored test result.

The release builder's `--prepare-corpus` flag requires `--prepare-only`.
The oracle is not shipped in release artifacts. Concurrent assertion consumers
may read the prepared inputs; preparation and mutation retain exclusive ownership.
