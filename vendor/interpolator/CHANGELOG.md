# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## [Unreleased] -->
## [0.5.0] - 2023-04-17
### Changed
- **Breaking Changed**: `format` and `write` take `Context` trait, this breaks type inference
- `i` allows access to outer variables

### Added
- `set_{trait}` functions on `Formattable`
- Macros for format, write and print

## [0.4.0] - 2023-03-01
### Changed
- **Breaking Change**: Made error enums `non_exhaustive`
- Added `Copy` implementation to `Trait` and `Formattable`

## [0.3.0] - 2023-03-01
### Added
- `i` gained implicit format `({})` when none is specified `{:i}`
- `i` gained support for indexing single values without range `{:i1}`

### Changed
- **Breaking Change**: `i` uses `{}` instead of `{it}`

## [0.2.0] - 2023-02-28
### Added
- `i` iter format

### Changed
- **Breaking Change**: Improved naming of error enums and variants

## [0.1.2] - 2023-02-27
Fixed `include` in Cargo.toml

## [0.1.1] - 2023-02-27
Fixed URLs in Cargo.toml

## [0.1.0] - 2023-02-27
Initial release

[unreleased]: https://github.com/ModProg/interpolator/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/ModProg/interpolator/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/ModProg/interpolator/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ModProg/interpolator/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ModProg/interpolator/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/ModProg/interpolator/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ModProg/interpolator/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ModProg/interpolator/v0.1.0
