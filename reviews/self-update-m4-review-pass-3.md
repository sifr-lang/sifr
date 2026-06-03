Distribution validation: exit 0. Fix verified end-to-end.

# READY

The pass-2 polish note is resolved. Verified at `scripts/distribution/validate_self_update_metadata.sh:52`:

```bash
python3 - "${metadata_path}" 2>&1 <<'PY'
```

The `2>&1` is now on the python3 command line *before* the heredoc, not on its own line after the `PY` terminator. Behavioral verification:

- **Isolated repro** — `out="$( python3 - 2>&1 <<'PY' ... raise SystemExit("...") PY )" || ...` captures the SystemExit message into `out` *and* the substitution exits non-zero, so `|| fail` fires.
- **End-to-end probe** — mutating `schema_version` to `true` now produces a single, correctly-attributed failure: `self-update metadata validation: channel metadata schema_version must be 1` (exit 2). The misleading "metadata versions could not be extracted" fall-through that pass-2 flagged is gone.
- **Full suite** — `scripts/run_distribution_validation.sh` exits 0, including all four M4 drift/agreement fixtures.

No blocking issues. M4 remains **READY**.
