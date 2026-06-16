## Pass 1 Review — Wave 9.5 runtime/platform executable evidence

Code-review stance against the Wave 9.5 task list in `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md` (lines 1569–1585) and the platform-policy rule on line 188. I read the new and modified files end-to-end and the upstream profile-runner integration enough to judge offline policy and result accounting. Validation output you posted reconciles with the variant counts I see in the code (4 support-matrix + 12 evidence = 16; runtime_platform area total 38 with 10 skips composed across platform-golden + platform-support-matrix + platform-evidence + sanitizer-smoke).

### Blockers

**None.** No correctness, network-policy, schema-fail-closed, overclaim, accounting, or guardrail blocker. Implementation satisfies the W9.5 task list and the platform-policy rule. Another Opus review round is not required before PR.

### Verifications passed during review

**Host/target support declaration (W9.5 task line 1569)**
- `supported_platforms.json` declares 6 host triples (2 macOS + 2 Linux + 2 Windows) and 6 target triples; each row has the required `status`, `merge_requirement`, `nightly_requirement`, `toolchain`, and `allowed_skips`. Stable Linux/macOS rows are `supported` + `execute`/`execute` with empty `allowed_skips`; Windows rows are `host-limited` + `structured-skip`/`structured-skip` with three concrete skip reasons (symlink privileges, POSIX signal, Unix subprocess signal status). No Windows row claims `supported` or `execute`, so the overclaim guard holds. `supported_platforms.json:14–18, 71–87, 89–105`.
- `validate_host_row` (check_platform_evidence.py:144–186) enforces fail-closed semantics: unknown field set, duplicate triple, unknown OS/status/requirement, allowed_skips on `supported`, and merge_requirement/status mismatch all `SystemExit`. The Windows row's `allowed_skips` is permitted because its status is `host-limited`, not `supported`.
- Targets (`validate_target_row` lines 189–204) limit the requirement set to `{execute-on-matching-host, structured-skip, not-required}` so a target row cannot accidentally claim host-style `execute`. Windows targets are `structured-skip`, consistent with the host policy.

**Executable host/target evidence coverage (W9.5 task lines 1570–1583)**
- `platform_evidence_manifest.json` lists all 12 W9.5-required cases (filesystem-paths, unicode-paths, symlink-roundtrip, file-permissions, tempdir-cleanup, line-endings, subprocess-exit-code, subprocess-stdio, loopback-networking, signals-process-control, locale-unicode-assumptions, install-distribution-smoke). `validate_evidence_manifest` (check_platform_evidence.py:207–241) enforces this exact set via `required.difference(seen)` — fail-closed if any is removed.
- Each builtin in `BUILTINS` (lines 557–570) maps to a real probe and the probes exercise the right behavior:
  - filesystem-paths checks `is_absolute()`, `os.sep` presence, and a round-trip readback through a relative path inside a tempdir.
  - unicode-paths writes a name with å, λ, and an emoji and verifies byte preservation.
  - symlink-roundtrip uses `Path.symlink_to`, then `is_symlink` + readback + `resolve()` parity. POSIX-only, with `host-limited` skip wired to the matching Windows reason.
  - file-permissions chmods `0o600` and checks `st_mode & 0o777`. POSIX-only, with `host-limited` skip.
  - tempdir-cleanup writes inside a `TemporaryDirectory()`, asserts existence inside the scope and non-existence after.
  - line-endings round-trips a binary payload mixing LF and CRLF without translation.
  - subprocess-exit-code uses `sys.executable` to launch a child that exits 7 and prints a marker; both exit code and stdout are verified.
  - subprocess-stdio pipes bytes in, reverses them in a child, and verifies both stdout bytes and stderr capture.
  - loopback-networking binds 127.0.0.1, asserts the peer address is loopback, and exchanges `ping`/`pong`. Honors the manifest's `network: loopback-only` field.
  - signals-process-control launches a sleeping child, sends `SIGTERM`, and accepts `-SIGTERM`, `128 + SIGTERM`, or `143`. POSIX-only with an explicit `EvidenceFailure` if invoked on Windows. The `try/finally` reaps even on the unhappy path (line 519–521).
  - locale-unicode-assumptions checks a non-empty preferred encoding, NFC normalization stability, and an explicit UTF-8 round trip.
  - install-distribution-smoke runs `cargo run --locked -q -p sifr -- --help` and asserts `Usage`/`Commands` is rendered. I verified the live binary emits both tokens.

**Structured skip reasons (W9.5 task line 1585)**
- `run_evidence_case` (check_platform_evidence.py:347–358) only emits a `skip` variant when `host_status ∈ case.allowed_skip_statuses`; otherwise the unsupported-OS path returns `fail` with a specific reason. Skips carry `host_triple` and the case's `skip_reason` (or a derived default), and the manifest's `validate_evidence_case` requires `skip_reason` whenever `allowed_skip_statuses` is non-empty (line 287–288). So a host-limited case cannot be silently skipped on an in-support OS, and an unsupported OS cannot quietly skip without a declared reason.

**Network policy / hermetic create-pr (CLAUDE.md merge-policy guard, task line 1584)**
- `platform_evidence_manifest.json` declares `network_policy.create_pr_merge = "loopback-only"`, `external_network = "forbidden"`. `validate_evidence_manifest` (lines 215–218) enforces both fail-closed — the self-test mutation `mutated_external_network` (line 707–710) flips `create_pr_merge` to `"external"` and confirms `SystemExit`.
- The only case touching the network is `loopback-networking`, which is restricted to `127.0.0.1` and rejects non-loopback peers explicitly (line 502).
- The `cargo` invocation in `install-distribution-smoke` runs under `CARGO_NET_OFFLINE=true` when launched through `profile_runner.py` (profile_runner.py:128–130 sets the env once `cargo_policy.offline` is true, and both `verification/profiles/create-pr.json:21` and `verification/profiles/merge.json:21` set `offline: true`). The env propagates to the runner subprocess and to its child cargo. So create-pr and merge cannot reach the registry through this case.
- I checked that no probe imports `urllib`, `requests`, `http`, or initiates a non-loopback socket.

**Schema validation fails closed**
- `validate_supported_platforms` and `validate_evidence_manifest` both raise `SystemExit` on every shape violation. `run_self_test` (lines 680–694) confirms three live mutation classes (supported host carrying skip, external network in policy, missing required evidence case) all `SystemExit`. The user reports `--self-test` passing.
- `validate_evidence_case` (lines 244–290) rejects unknown fields, unknown commands, unknown OSes, non-positive timeouts, allowed-skip statuses without `skip_reason`, and non-loopback `network` values — every required guard is present.
- The runner's behavior when `validate_*` raises is: SystemExit → no result JSON → `run_platform_evidence_suite` falls into its JSON-read exception path (runner.py:187–203) and emits a `fail` variant with the subprocess exit code. So a manifest mutation cannot pass silently.

**Result accounting**
- `support_matrix_variants` returns 4 pass variants; `evidence_variants` returns one variant per case (12 on this host). Totals match the harness summary you posted (4 / 12).
- `main()` summary (check_platform_evidence.py:75–82) computes `total_variants`, `total_failures`, `blocking_failures = failures`, `non_blocking_failures = 0`, and `skipped`. Exit code is `1` iff `failures > 0` (line 88); skips do not flip exit code, matching the structured-skip policy.
- `run_platform_evidence_suite` (runner.py:168–216) reads back the variants verbatim, and only appends a synthetic fail when the subprocess exited non-zero AND none of the variants is already `fail` — so we don't double-count. `run_suite` then tallies `failed_cases`, `total_variants`, `total_failures`, `total_skips`. The runtime_platform area `main` rolls these up correctly into `summary`.
- Skip variants set `actual_exit_code: None` (lines 634–645), which keeps them distinguishable from `pass` (0) and `fail` (1).

**Profile assignments (task lines 1462–1469 / W9.5)**
- create-pr selects `platform-golden + platform-support-matrix + platform-evidence` under `platform-specific` (verification/profiles/create-pr.json:82–89).
- merge selects the same three plus `sanitizer-smoke` (verification/profiles/merge.json:106–114).
- nightly selects the same three plus `sanitizer-full` (verification/profiles/nightly.json:100–108).
- release matches nightly (verification/profiles/release.json:100–108).
- No profile selects `sanitizer-full` on create-pr/merge or `sanitizer-smoke` on nightly/release, preserving the W7.3 split.

**No overclaiming**
- Windows host rows are `host-limited` with `structured-skip` for both merge and nightly; their `allowed_skips` enumerate the three concrete host-policy reasons.
- Linux/macOS rows are `supported` with `execute`; no `allowed_skips`, and `validate_host_row` rejects any non-empty `allowed_skips` for `supported` (line 181–182).
- The evidence manifest does **not** claim Windows runs symlink-roundtrip / file-permissions / signals-process-control — `supported_os` is POSIX-only on those three; the other nine cases legitimately work on Windows (loopback networking, line endings, tempdir cleanup, unicode paths via `wchar_t`-backed Path, etc.).
- No sanitizer/toolchain overclaim was introduced — the W9.5 changes only add support-matrix + evidence suites and leave sanitizer manifest, sanitizer assignment, and `evidence_suites` on Windows excluding `sanitizer-smoke` unchanged.

**File-size and maintainability**
- `check_platform_evidence.py` is 721 lines; `runner.py` is 562; `supported_platforms.json` is 146; `platform_evidence_manifest.json` is 217. All under the 900-line first-party cap. You report the file-size guardrail passes.
- The check_platform_evidence tool is single-purpose, ~12 builtin probes plus validators — decomposed by concern (load → validate → run → report → self-test).

### Non-blocking suggestions

These are robustness improvements, not gate failures.

1. **Uncaught non-`EvidenceFailure` exceptions abort the suite mid-run with no JSON.** `run_evidence_case` (check_platform_evidence.py:360–381) only catches `EvidenceFailure`. The probes can legitimately raise other types:
   - `check_install_distribution_smoke` has `subprocess.run(..., timeout=20, ...)`; on timeout it raises `subprocess.TimeoutExpired`, not `EvidenceFailure`.
   - `check_signals_process_control` calls `proc.wait(timeout=5)` (line 517 and again in the `finally` line 521); either can raise `TimeoutExpired`.
   - `check_loopback_networking` could raise `OSError` if the ephemeral port can't bind (unlikely but possible under sandboxing).
   - `check_file_permissions` could raise `PermissionError` on hostile filesystems.
   
   When this happens, the script crashes before writing `args.json-out`, and runner.py's JSON-read branch turns it into a single generic `failed to read platform evidence result JSON: …` variant. The real reason is in the captured stdout (the runner prints `result.stdout`), but the structured payload loses it.
   
   Suggested fix: wrap the body of `run_evidence_case`'s `try` to also catch `subprocess.TimeoutExpired` and `OSError` and convert to a fail variant with `str(exc)` so the suite always finishes and the JSON always lands.

2. **`run_with_timeout` is a post-hoc check, not enforcement.** check_platform_evidence.py:384–389 measures `time.monotonic()` *after* the callback returns and only raises if elapsed exceeded the budget. A probe that hangs forever (e.g., a kernel deadlock on `accept()`) will never time out. The probe-internal `timeout=…` arguments cap most realistic hangs, but the wrapper name suggests enforcement that isn't there.
   
   Suggested fix: either rename `run_with_timeout` to `assert_within_timeout` to match the semantics, or move to `concurrent.futures` / `signal.alarm`-based cancellation. The post-hoc check is fine for current probes, so renaming would be the minimal fix.

3. **`install-distribution-smoke` has a 20-second budget that matches its subprocess timeout.** If the local cargo cache is warm (typical CI) this is fine. On a PR that touched `sifr_driver` source (forcing a rebuild on first invocation), `cargo run --locked -q -p sifr -- --help` can exceed 20s on slow hosts. Today the probe would hit `subprocess.TimeoutExpired` and trip finding #1 above.
   
   Suggested fix (low priority): either raise the budget to 60s for this case (it's `install-distribution-smoke`, not a hot path), or split into "binary exists / Cargo.toml is reachable" + "binary launches" so the rebuild-once cost doesn't dominate. The current 20s is also what's declared in the manifest; raising both stays consistent.

4. **`support_matrix_variants` returns four hard-coded pass variants.** The actual gate is `validate_supported_platforms` at load time. The four variants don't re-verify anything against the loaded JSON — they're labels for what was validated.
   
   This is fine as long as readers understand the model (validate-then-label). If you want the variants to carry real signal, you could move the relevant assertion into each variant's body (e.g., `current-host-declared` could re-look up the host row and fail if not found, instead of relying on `host_support_row()` having raised earlier). Not blocking; the current shape is consistent with how other manifest suites in this area treat schema validation.

5. **Self-test could exercise more mutation branches.** `run_self_test` covers three classes (supported-with-skip, external network, missing case). The validators also reject: duplicate triples, unknown fields, missing required fields, unknown OS, unknown command, non-positive timeout, allowed-skip without `skip_reason`, non-loopback `network`. Adding one mutation per family would tighten the fail-closed contract without much code.

6. **`print_case_timing` for `platform-support-matrix` fires four `pass` timings *before* the variants are constructed.** The timings are not contingent on the variant being built. Minor smell — if the build itself failed, the timing would have lied about it. In practice the build is unconditional, so it can't happen, but it makes the timing emission an artifact rather than an observation.

7. **`x86_64-pc-windows-msvc` arch is recorded as `x86_64`, not `aarch64`.** That's fine for the existing matrix, but Windows-on-ARM (`aarch64-pc-windows-msvc`) is becoming a real target. If/when it lands, it would have to be added to both the host triples and the target triples — currently the absence of an aarch64-windows row would `SystemExit` with "current host triple is not declared in supported_platforms.json" rather than structured-skip. Worth noting but not blocking for W9.5.

8. **`current_rust_host_triple()` requires `rustc` on PATH.** A host without rustc would `SystemExit` at line 596. That's the right behavior for this gate, but it means the runtime_platform area cannot self-execute on a stripped runner image. The W9.5 task doesn't ask for that, and the toolchain requirement is recorded in `supported_platforms.json:17` (`toolchain: stable`). Documenting this dependency in `platform_contract.md` would help future contributors who try to run the area in isolation.

### Recommendation

**No further review round is needed before PR.** The implementation satisfies every Wave 9.5 task line on `supported_platforms.json` shape, the 12 executable evidence concerns, structured skips, loopback-only networking, and profile assignments. Validation is fail-closed, accounting is sound, no Windows or sanitizer overclaim is introduced, and file-size/maintainability are within bounds.

If you choose to land any robustness cleanup before PR, suggestion #1 (catch `TimeoutExpired`/`OSError` in `run_evidence_case`) is the highest-value: it removes the only path where a probe failure can degrade into a generic JSON-read error in the area summary. Everything else is post-merge polish.
