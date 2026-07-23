# M2 Former E2E Dependency-Inference Inventory

This is the checked deletion record for the generated-Rust inference rules
removed by M2. The executable evidence is
`crates/sifr/tests/e2e_support/dependency_plan_authority_tests.rs`. Its bounded
tables name every former rule below and verify the corresponding typed module
or `StdlibFeature` identity. The cache-disabled pass suite then proves those
production plans build the complete live corpus without inference.

## Former module rules

| Deleted scanner rule | Typed production authority | Checked production resolution |
| --- | --- | --- |
| `_bigint` | BigInt use is detected structurally by codegen and emits `NumBigint` plus `NumTraits`. | The plan records both typed requirements and renders retained `num-bigint`/`num-traits` Cargo inputs. |
| `_sifr.fs` | Resolved private stdlib imports are retained in `used_stdlib_modules`; `features_for_stdlib_module` maps the module to `Fs`. | `planned_sifr_stdlib_features` selects `sifr_stdlib/fs`. |
| `_sifr.net` | Resolved private stdlib imports plus the module feature registry supply `SifrRuntime`, `Tokio`, and `Tracing`. | The plan selects `sifr_stdlib/net` and the runtime `net` feature. |
| `_sifr.tls` | Resolved private stdlib imports plus the module feature registry supply the runtime/TLS feature set. | The plan selects `sifr_stdlib/tls` and the runtime `net,tls` features. |
| `_sifr.http` | Resolved private stdlib imports plus `StdlibFeature::Http`. | The plan selects `sifr_stdlib/http`; its implementation dependencies stay transitive. |
| `_sifr.signal` | Resolved private stdlib imports are the typed input; no generated-Rust inspection is needed. | `planned_sifr_stdlib_features` selects `sifr_stdlib/signals`. |

## Former runtime-crate rule

| Deleted scanner rule | Typed production authority | Checked production resolution |
| --- | --- | --- |
| `sifr_runtime` | `StdlibFeature::SifrRuntime`, or runtime requirements derived from resolved stdlib modules. | The plan renders the resolved sysroot `sifr_runtime` path dependency and selected runtime features. |

## Former direct-crate rules

The “resolution” column distinguishes retained generated-project dependencies
from sysroot implementation dependencies. A transitive entry must never be
reintroduced as a direct generated-project dependency.

| Deleted scanner rule | Typed production authority | Checked production resolution |
| --- | --- | --- |
| `regex` | `StdlibFeature::Regex` / resolved `sifr.re` modules | `sifr_stdlib/regex` (transitive implementation crate) |
| `rand` | `StdlibFeature::Rand` / resolved random modules | `sifr_stdlib/random` (transitive) |
| `rand_distr` | `StdlibFeature::RandDistr` / resolved random modules | `sifr_stdlib/random` (transitive) |
| `chrono` | `StdlibFeature::Chrono` / resolved time modules | `sifr_stdlib/time` (transitive) |
| `md5` | `StdlibFeature::Md5` / resolved hash modules | `sifr_stdlib/hash` (transitive) |
| `uuid` | `StdlibFeature::Uuid` / resolved UUID modules | `sifr_stdlib/uuid` (transitive) |
| `toml` | `StdlibFeature::Toml` / resolved TOML modules | `sifr_stdlib/toml` (transitive) |
| `flate2` | `StdlibFeature::Flate2` / resolved gzip modules | `sifr_stdlib/gzip` (transitive) |
| `zip` | `StdlibFeature::Zip` / resolved zip modules | `sifr_stdlib/zipfile` (transitive) |
| `base64` | `StdlibFeature::Base64` / resolved Base64 modules | `sifr_stdlib/base64` (transitive) |
| `sha1` | `StdlibFeature::Sha1` / resolved hash modules | `sifr_stdlib/hash` (transitive) |
| `sha2` | `StdlibFeature::Sha2` / resolved hash modules | `sifr_stdlib/hash` (transitive) |
| `blake2` | `StdlibFeature::Blake2` / resolved hash modules | `sifr_stdlib/hash` (transitive) |
| `rust_decimal` | Structural decimal codegen emits `StdlibFeature::RustDecimal`. | Retained direct `rust_decimal` Cargo input. |
| `bigdecimal` | Structural decimal codegen emits `StdlibFeature::BigDecimal`. | Retained direct `bigdecimal` Cargo input. |
| `tracing` | `StdlibFeature::Tracing`; runtime diagnostics now resolve through `_sifr.runtime`. | `sifr_stdlib/runtime-observability` or runtime features; never a direct diagnostics dependency. |
| `metrics` | `StdlibFeature::Metrics`; runtime diagnostics now resolve through `_sifr.runtime`. | `sifr_stdlib/runtime-observability`; never a direct diagnostics dependency. |
| `postcard` | `StdlibFeature::Ipc` remains the typed requirement identity. | The old scan is obsolete: current `sifr.ipc` is checked source and emits no direct `postcard` reference. |
| `url` | `StdlibFeature::Url` / resolved URL modules | `sifr_stdlib/url` (transitive) |
| `percent-encoding` | `StdlibFeature::PercentEncoding` / resolved URL modules | `sifr_stdlib/url` (transitive) |
| `http` | `StdlibFeature::Http` / resolved HTTP modules | `sifr_stdlib/http` (transitive) |
| `bytes` | `StdlibFeature::Bytes` is the typed requirement identity; active adapters are selected by resolved modules. | The old substring scan also matched `sifr_stdlib::bytes`; `sifr_stdlib/bytes` owns the implementation. |
| `h2` | `StdlibFeature::H2` | `sifr_stdlib/http` plus runtime HTTP features (transitive) |
| `http-body` | `StdlibFeature::HttpBody` | `sifr_stdlib/http` plus runtime HTTP features (transitive) |
| `http-body-util` | `StdlibFeature::HttpBodyUtil` | `sifr_stdlib/http` plus runtime HTTP features (transitive) |
| `hyper` | `StdlibFeature::Hyper` | `sifr_stdlib/http` plus runtime HTTP features (transitive) |
| `hyper-util` | `StdlibFeature::HyperUtil` | `sifr_stdlib/http` plus runtime HTTP features (transitive) |
| `tower-service` | `StdlibFeature::TowerService` | `sifr_stdlib/http` plus runtime HTTP features (transitive) |
| `cookie` | `StdlibFeature::Cookie` is the typed requirement identity. | The old scan is obsolete; cookie parsing/building is implemented inside `sifr_stdlib/http`. |

## Validation evidence

- Checked inventory: 6 module rules, 1 runtime rule, and 29 direct-crate rules.
- Missing-metadata regression: direct `num_bigint` Rust with empty typed metadata
  fails its Cargo build; no source scan repairs it.
- Cross-fixture regression: different plans cannot form one batch, and normal
  planning places them in separate groups.
- Cache regression: changing the production plan changes the group identity.
- Production parity regression: the harness plan is compared with a real
  `build_single_file_report`, including resolved sysroot interop crates and the
  complete dependency fingerprint.
- Repeated pure-Sifr compilation reuses the resolved stdlib interop metadata by
  sysroot identity; this preserves exact plans without resolving the same
  private bridge graph once per fixture.
- Cache-disabled E2E: 648/648 pass fixtures, 143 groups, 0 cache hits.
