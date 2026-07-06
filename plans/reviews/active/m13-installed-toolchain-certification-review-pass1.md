## Findings, severity-ordered

**Non-blocking correctness / hardening:**

1. **`compiler_surface_matrix.json:365`** — new `nightly_release_suite` uses `" + "` where every other row uses `,`. Consumer-drift risk.
2. **`check_no_path_leakage.py:207`** — `Path.home()` as a raw byte substring will false-positive on CI where TMPDIR lives under HOME (installed sysroot JSON legitimately embeds the extract-root under TMPDIR). Passed locally because macOS TMPDIR is under `/var/folders`, not `/Users/...`.
3. **`build_preview_artifacts.sh:154`** — `release_rustflags` doesn't remap `CARGO_TARGET_DIR` when it's set to a path outside repo/sysroot/CARGO_HOME/RUSTUP_HOME. This wave's own runner uses a target dir inside `REPO_ROOT` (covered by the repo remap), but a release engineer using an external target dir could still leak host paths into the shipped binary — and the installed certification wouldn't catch it because it only forbids `REPO_ROOT` and `Path.home()`.
4. **`build_preview_artifacts.sh:148`** — `repo_root="$(pwd)"` silently misses the real checkout root when the script is run from a subdirectory. Today all callers use `cwd=REPO_ROOT`, so this is latent. `git rev-parse --show-toplevel` (or an explicit `--repo-root` arg mirroring `--sysroot-root`) would eliminate the invariant.
5. **`rust_interop_probe.rs:178`** — release fallback `crates/sifr_runtime` is a relative path against `env::temp_dir()`, so if the installed sysroot ever fails to supply `runtime_crate_manifest`, users see an opaque Cargo "manifest not found" error rather than a Sifr diagnostic. Prior behavior was worse (leaked source-tree path), but a hard `probe_io_failure` here would be strictly better than a broken relative fallback.
6. **`runner.py:418`** — `archive.extractall(install_root)` has no `filter=` argument. Python 3.12/3.13 emit a DeprecationWarning; 3.14 will error. Pre-validation already rejects unsafe members, so adding `filter='data'` is mechanical.

## Blocking findings

**None.** No correctness bug that would ship broken code or mis-certify a release.

## Non-blocking residual risks

- The heavy suite is ~15 min real; runner explicitly guards `parallel_safe: false`. That's correct, but the runtime is entirely in the nightly/release lane cost budget — worth watching if more heavy checks land here later.
- `BUILT_ARCHIVES` cache only works within a single Python process. The profile runner invokes the area once with both `--suite` args, so caching works today; a future refactor that splits the profile invocation would silently double the build cost.
- The lsp smoke's success check is only `'"id":1' in stdout` (stripped of spaces). Weak but sufficient as a smoke; not worth strengthening in this wave.

## PR readiness

**Yes, satisfied for PR.** The wave meets its stated goal — the merge lane gets a fast, self-contained installed-toolchain smoke; the two real path-leak causes (`explain_cli` and `rust_interop_probe`) are fixed cleanly with `#[cfg(debug_assertions)]` guards; the release verifier's one-pass digest change is correct and materially faster; documentation, coverage matrices, and profile assignments are all internally consistent. The findings above are worth follow-ups but none change the shipped behavior for the current release path.
