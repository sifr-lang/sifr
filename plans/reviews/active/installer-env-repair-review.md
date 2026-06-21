I've reviewed the diff against the surrounding installer logic. The fix is correctly scoped to the root cause. No blocking findings.

## Verdict
No blocking correctness regressions. The change preserves the documented invariants (`NO_MODIFY_PATH=1`/`--no-modify-path` → no writes; install_dir-on-PATH → no profile edits or "configured" message), and the new test correctly exercises the "already installed → repair" path that self-update hits.

## Findings (ordered by severity)

### Medium — Test gaps versus the stated invariants

1. **No assertion that the repair path is silent.** The prompt says: "If install_dir is already on PATH, it should still avoid appending profile hooks or printing a fresh PATH configuration message." `artifact_configures_path.sh:68-75` captures `output` but never asserts the absence of `"configured Sifr PATH via"`. A future regression that re-emits the message (or, worse, re-touches profiles) would slip past CI.
   - Suggested guard: assert `output` does NOT contain `"configured Sifr PATH via"`; assert `.zshrc` line count / mtime is unchanged across the repair run (or that it still has exactly one `. "${HOME}/.sifr/env"` line).

2. **No regression coverage for `SIFR_NO_MODIFY_PATH=1` on the repair path.** The fix moves writes ahead of the early return for `install_dir_on_path`, but the `NO_MODIFY_PATH=1` short-circuit at `generate_version_installer.sh:445` remains the only thing preventing writes when the user opted out. There's no test combining `SIFR_NO_MODIFY_PATH=1` with an `install_dir`-on-PATH re-run that asserts `~/.sifr/env` is *not* created. Given how easy that invariant is to break inside `configure_path` if someone later reorders, an explicit negative test would be cheap insurance.

### Low — Parallel hole / behavior nuances (not regressions)

3. **Fish env script is not repaired symmetrically.** The new early-return at `generate_version_installer.sh:469-471` sits *before* the fish block (`491-495`). So if a fish user loses `~/.config/fish/conf.d/sifr.env.fish` (the analogue of the bug being fixed) and runs self-update with install_dir on PATH, the fish file is not recreated. Same root cause, just on a less common shell. Not a regression — the old code didn't handle it either — but worth deciding whether to extend the fix or document the asymmetry.

4. **Stray `~/.sifr/env` for custom install dirs on PATH.** When a user sets `SIFR_INSTALL_DIR` to something outside `$HOME/.sifr/bin` *and* that dir is already on PATH, the new code unconditionally creates `~/.sifr/` and writes `~/.sifr/env` pointing at the custom dir. Previously the home tree was left untouched in this scenario. The env content is harmless (it adds the custom dir to PATH if missing), but it's a new side effect on custom installs.

5. **Test now requires `zsh` on the verification host.** Line 85 actually invokes `zsh -c '…'` (the first test used `SHELL=/bin/zsh` but never executed zsh). If any verification environment runs without zsh, this case will newly fail. The macOS/Linux CI runners likely already have it, but worth confirming so this doesn't surprise an offline contributor.

### Nit
- `install_dir_on_path=0` + `case` + later `[ "${install_dir_on_path}" = "1" ]` reads fine but is one extra global. Equivalent without the flag: re-`case` against `:${PATH}:` at the second decision point. Style preference only.

## Confirmations
- POSIX shell portability: `[`, `case`, simple assignments only — fine. `set -eu` semantics preserved (no unset reads added).
- Idempotent env rewrite: `write_env_script_sh` truncates, so successive repair runs converge.
- Self-update path: the "already installed" branch at `generate_version_installer.sh:541-546` calls `configure_path` then exits — this is precisely what the new test exercises, so the targeted bug class is covered.
