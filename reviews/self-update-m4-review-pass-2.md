I ran the full distribution validation suite and inspected each polish fix. Here's the verdict.

# READY (one polish note recurs)

All M4 obligations remain met after the pass-2 fixes:

- **Drift coverage intact** — `channel_metadata_dispatcher_drift_rejected.sh`, `channel_metadata_installer_drift_rejected.sh`, `channel_metadata_stable_rejected.sh`, and `channel_metadata_installer_agreement.sh` all pass via `scripts/run_distribution_validation.sh`.
- **Create-new-version fixtures still prove mutation** — `create_new_version_alpha_dry_run.sh` asserts `channel_metadata=…` and `channel_metadata_update=…`; `create_new_version_real_run_plan_reuse.sh` asserts the metadata write and re-invokes the validator on the populated install root.
- **Wiring** — `scripts/run_distribution_validation.sh` globs `verification/distribution/*.sh` (excluding `common.sh`), so the four new drift fixtures are discovered without changes.
- **Validator branches** — `validate_self_update_metadata.sh` still rejects malformed JSON, unsupported fields, stable channel, unknown channels, ordering drift, mismatched prerelease labels, dispatcher↔dispatcher drift, metadata↔dispatcher drift, missing installer, and APP_VERSION drift.

## Polish fix audit

| Pass-1 note | Pass-2 claim | Actual state |
|---|---|---|
| #1 capture python stderr into wrapper | "captures Python stderr into the wrapper failure message" | **Not landed as described** — see below |
| #2 `schema_version != 1` accepts `True` | exact int type + value | ✅ `type(schema_version) is not int or schema_version != 1` at `scripts/distribution/validate_self_update_metadata.sh:69` — verified by mutating fixture metadata to `true` and observing rejection |
| #3 redundant prerelease shell recheck | removed | ✅ `validate_installer` (`scripts/distribution/validate_self_update_metadata.sh:127-136`) no longer re-derives the channel label; Python regex at line 89 is the sole check |
| #4 comment on artifact_dir deletion | added | ✅ `verification/distribution/common.sh:157` |

## Recurring issue — non-blocking

Fix #1 doesn't actually work. The `2>&1` placement at `scripts/distribution/validate_self_update_metadata.sh:93` (on its own line *after* the heredoc terminator `PY`) is parsed as a standalone redirection, not as a redirection on `python3`. I confirmed with a minimal reproduction:

```bash
out="$( python3 - <<'PY'
...
raise SystemExit("msg")
PY
  2>&1
)"
# → substitution exit status: 0   (python's exit 1 is lost)
# → stderr leaks to parent; out captures only stdout
```

Consequence: when Python rejects metadata, the substitution exits 0 with empty `metadata_values`, so `|| fail "${metadata_values}"` never fires. Execution falls through to `[[ -n "${metadata_alpha}" && -n "${metadata_beta}" ]] || fail "metadata versions could not be extracted"` at line 109. Combined with the leaked-but-uncaptured Python stderr, the user sees:

```
channel metadata schema_version must be 1
self-update metadata validation: metadata versions could not be extracted
```

Tests still pass because `require_failure_contains` captures combined stderr+stdout via `2>&1`, so the leaked Python message is matched at the test layer. But the wrapper has not been improved over the pass-1 state — and it's now arguably *more* misleading because line 109 prints a misattribution ("could not be extracted" when the real reason is a Python-level rejection).

The minimal correct placement is `python3 - "${metadata_path}" 2>&1 <<'PY'` (redirection before the heredoc on the command line), which I verified yields substitution status 1 and captures stderr into the substitution. Suggested for a follow-up — not blocking M4.

## Recommendation

**READY.** M4 definition-of-done items are all met; all seeded drift, agreement, mutation-line, and write-assertion fixtures pass. The Python-stderr capture polish is still imperfect but the validator behaves correctly for every contract obligation, so this should not block merge. Worth a one-line follow-up to actually land fix #1 by moving `2>&1` to the same line as `python3 -`.
