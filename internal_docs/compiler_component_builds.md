# Compiler component builds

The SQL builders and the non-SQL words fixture record the actual inputs and
tools that produced every checked-in component. Rebuilds use the repository's
exact Rust toolchain and locked dependencies. Install its `wasm32-wasip2` and
`wasm32-unknown-unknown` target libraries before building.

PostgreSQL and SQLite require the official WASI SDK 34.0 archive for the build
host. Download that archive from the [SDK 34 release](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-34),
verify its published SHA-256 before extraction, and supply both paths:

```bash
export WASI_SDK_ARCHIVE=/absolute/path/to/wasi-sdk-34.0-arm64-macos.tar.gz
export WASI_SDK_PATH=/absolute/path/to/wasi-sdk-34.0-arm64-macos
```

`wasi_sdk_inputs.py` owns the official host asset names, sizes and checksums.
Before invoking a compiler, it authenticates the archive and compares every
installed file and symbolic link to that archive. A replaced Clang, altered
sysroot, additional header, unsupported host or missing archive fails the build.
The builder selects that SDK's compiler, archiver and sysroot explicitly.
Ambient C/C++ flags and SQLite compile-flag overrides are rejected. MySQL uses
a pure Rust parser and does not claim to consume an SDK.

Initialize the six pinned `third_party/libpg_query` sources and
`third_party/wasi-virt`. The PostgreSQL builder verifies every parser's exact
commit and tracked-source checksum. Each SQL builder verifies and builds the
pinned WASI-Virt source with its own locked upstream dependencies and disabled
default features. Its historical upstream adapter binaries remain authenticated
source inputs; they are not relabeled as newly built adapters.

With the locked dependency sources available in Cargo's cache:

```bash
python3 verification/areas/sql_platform/tools/build_postgresql_components.py
python3 verification/areas/sql_platform/tools/build_mysql_components.py
python3 verification/areas/sql_platform/tools/build_sqlite_component.py
python3 verification/areas/sql_platform/tools/build_words_component.py
```

The builders use Cargo's actual target directory, including an explicitly
selected private `CARGO_TARGET_DIR`. PostgreSQL builds all majors 13–18; MySQL
builds series 8.4, 9.7 and 26.7; SQLite retains the selected 3.53.2 grammar.
SQLite's native-source upgrade is a separate qualification item.

Each SQL `component-artifacts.json` records the source-file digest map, exact
Cargo lock, shared guest sources, grammar and WIT inputs, build scripts, Rust
compiler/Cargo/target-library identities, virtualizer source and executable
digests, SDK compiler/sysroot identities where used, and core/final component
digests. All artifacts in a family must build before its receipt is replaced.
The words fixture's receipt records the same source and Rust identities plus
the actual componentizer binary and exact wit-bindgen 0.61.1/wit-component
0.258.0 versions. The encoder validates the produced component.

`check_component_qualification.py` rejects stale sources, changed producer
selections, missing majors and mismatched artifact bytes. The SDK input tests
exercise compiler, header and symbolic-link substitution. Runtime provider
tests execute every SQL artifact in the capability-free host; the component
crate tests exercise the words fixture. Historical server/driver qualification
receipts remain evidence for their original runs and are not rewritten by
component regeneration.
