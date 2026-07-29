# fixture: native_build_script
# scenario-example: native_trust_package

This locked/offline scenario certifies package-level trust for four direct
wrapper crates. Their build scripts compile exact root-lock `cc`, `bindgen`,
`cxx`, and `zstd` dependencies and emit versioned evidence files into
`OUT_DIR`. The generated Sifr package observes those artifacts and proves a
zstd encode/decode roundtrip.

The `cc` and zstd wrappers declare `sifr_cc_probe` and
`sifr_zstd_probe` native identities before Cargo executes. The actual zstd
dependency emits `zstd`, while the cxx graph emits `cxxbridge1` and
`link-cplusplus` and selects `c++` on Apple targets or `stdc++` on GNU targets.
All seven portable-envelope names are exact manifest allowlist entries. Any
other build-script native link remains a post-build hard error.

The certified host envelope is Apple/GNU arm64 and x86_64. Running this
scenario requires a working C/C++ compiler and a `libclang` installation
discoverable by bindgen; these host tools are prerequisites rather than Cargo
dependencies. MSVC is not covered by this scenario.
