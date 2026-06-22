# fixture: native_build_script
# scenario-example: native_trust_package

This scenario models package-level trust for Rust build scripts and native-link
metadata. `zstd` exposes native `links` metadata, and every backend crate with a
build script is listed in `[trust].rust-build-scripts`.
