

## Review: `preview-release.yml` submodule fix

### Patch correctness

The single-line change — adding `submodules: recursive` to the `actions/checkout@v4` step — is correct and necessary. Ruff is a git submodule at `third_party/ruff/`, and without this flag `actions/checkout` only retrieves the top-level repo, leaving Cargo unable to resolve `crates/ruff_python_parser` dependencies that live inside the submodule.

### Publication risk assessment

| Concern | Status | Notes |
|---|---|---|
| `--latest=false` on release | **Safe** | Releases won't override GitHub's "Latest" tag. |
| Version validation | **Safe** | Stable-looking versions rejected until Phase 39. Channel/version consistency enforced. |
| Checksum verification | **Safe** | SHA-256 verified before upload; mismatches abort the run. |
| Idempotent release creation | **Safe** | `gh release edit` if exists, `create` if not — prevents duplicate release errors. |
| `--clobber` on asset upload | **Intended** | Enables rebuild/re-dispatch with the same version. Deliberate design. |
| `workflow_dispatch` only | **Safe** | No accidental triggers from pushes or PRs. |
| `submodules: recursive` | **Correct** | Must fetch the full ruff submodule tree. |

### Hidden risks

None identified. The `--latest=false` flag and prerelease designation together ensure these Phase 33 previews don't pollute stable upgrade paths.

### Verdict

**SATISFIED** — The patch is correct and the workflow is safe to re-dispatch for both `alpha` and `beta` channels. No blockers.

The root cause (missing `submodules: recursive`) is fully addressed. No additional changes needed before dispatching Phase 33 alpha and beta releases.
