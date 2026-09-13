# Editor candidate qualification demo

Initialize the actual editor repositories with
`git submodule update --init --recursive -- editor_integrations`.
Select the exact Node release in `editor_integrations/vscode/.node-version`.
The npm release is selected independently by the extension's `packageManager`;
the npm bundled with Node is only used to bootstrap this private installation:

```bash
item_npm_root="$(mktemp -d "${TMPDIR:-/tmp}/sifr-demo-npm.XXXXXX")"
item_npm_bin="$(bash editor_integrations/vscode/scripts/setup-npm.sh "${item_npm_root}")"
export PATH="${item_npm_bin}:${PATH}"
python3 scripts/check_node_toolchain.py
bash demos/editor_candidate_qualification/run.sh
```

The demo checks the owner checkout, metadata, and active Node/npm before
building native candidate artifacts. It requires a clean checkout and the
normal release build prerequisites. A toolchain preflight pass alone does not
qualify a release candidate.
