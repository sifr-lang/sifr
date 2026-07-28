"""Self-test for protected stable public install/update/docs smoke."""

from __future__ import annotations

import os
import tempfile
from pathlib import Path

from scripts.distribution.verify_public_stable_docs import (
    verify_public_stable_docs,
)

from .common import (
    GovernanceError,
    canonical_json_bytes,
    sha256_file,
    write_canonical_json,
)
from .schema_contracts import site_facts
from .stable_prepare_selftest import StablePrepareFixture
from .stable_publish import SMOKE_FILES
from .stable_publish_fixture import run_command, stage_fixture

REPO_ROOT = Path(__file__).resolve().parents[4]


def run_self_tests() -> int:
    tests = (test_public_smoke_adapter, test_withdrawal_documentation_contract)
    for test in tests:
        test()
        print(f"stable-public-smoke pass: {test.__name__}")
    print(f"stable public smoke self-tests ok: tests={len(tests)}")
    return 0


def test_public_smoke_adapter() -> None:
    with StablePrepareFixture() as context:
        paths = stage_fixture(context)
        summary = paths["summary"]
        staged = paths["staged"]
        dispatchers = context["root"] / "smoke-dispatchers"
        dispatchers.mkdir()
        (dispatchers / "index").write_text("#!/usr/bin/env sh\nexit 0\n")
        self_update = {
            "schema_version": 2,
            "current_version": summary["version"],
            "target_version": summary["version"],
            "receipt_channel": "stable",
            "requested_channel": "stable",
            "resolved_channel": "stable",
            "install_dir": "/tmp/sifr/bin",
            "binary_path": "/tmp/sifr/bin/sifr",
            "sysroot_path": "/tmp/sifr",
            "installer_url": "https://example.invalid/sifr-installer-0.1.0",
            "action": "no_op",
            "force": False,
            "would_run_installer": False,
            "warnings": [],
        }
        (dispatchers / "stable").write_text(
            """#!/usr/bin/env sh
set -eu
mkdir -p "${SIFR_INSTALL_DIR}"
cat >"${SIFR_INSTALL_DIR}/sifr" <<'SIFR'
#!/usr/bin/env sh
printf '%s\\n' '"""
            + canonical_json_bytes(self_update).decode().strip()
            + """'
SIFR
chmod +x "${SIFR_INSTALL_DIR}/sifr"
""",
            encoding="utf-8",
        )
        asset_digests = context["root"] / "smoke-assets.json"
        write_canonical_json(
            asset_digests,
            {
                path.name: sha256_file(path)
                for path in sorted((staged / "release-assets").iterdir())
            },
            refuse_existing=True,
        )
        fake_bin = context["root"] / "smoke-bin"
        fake_bin.mkdir()
        _write_smoke_curl(fake_bin / "curl")
        smoke_docs = context["root"] / "stable-release-docs.html"
        smoke_docs.write_text(
            "Active stable version: 0.1.0\nWithdrawn stable versions: none.\n",
            encoding="utf-8",
        )
        environment = os.environ.copy()
        environment.update(
            {
                "PATH": f"{fake_bin}:{environment['PATH']}",
                "SMOKE_INDEX": str(staged / "channels.json"),
                "SMOKE_DISPATCHERS": str(dispatchers),
                "SMOKE_ASSETS": str(staged / "release-assets"),
                "SMOKE_DOCS": str(smoke_docs),
            }
        )
        output = context["root"] / "public-smoke"
        run_command(
            [
                str(REPO_ROOT / "scripts/distribution/run_stable_public_smoke.sh"),
                "--repository",
                "sifr-lang/sifr",
                "--version",
                summary["version"],
                "--index",
                str(staged / "channels.json"),
                "--dispatchers",
                str(dispatchers),
                "--site-facts",
                str(staged / "stable-site-release-facts.json"),
                "--asset-digests",
                str(asset_digests),
                "--marketplace-vsix",
                str(
                    context["artifact_root"]
                    / summary["artifacts"]["vsix"]["workflow_artifact_name"]
                    / summary["artifacts"]["vsix"]["name"]
                ),
                "--out",
                str(output),
            ],
            env=environment,
        )
        assert {path.name for path in output.iterdir()} == set(
            SMOKE_FILES.values()
        )


def test_withdrawal_documentation_contract() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-stable-docs-") as temporary:
        root = Path(temporary)
        facts = site_facts()
        facts["withdrawals"] = [
            {"version": "0.0.9", "incident_id": "inc-rollback-001"}
        ]
        facts_path = root / "stable-site-release-facts.json"
        write_canonical_json(facts_path, facts, refuse_existing=True)
        document = root / "stable.html"
        document.write_text(
            "Active stable version: 0.1.0\n"
            "Withdrawn stable versions: 0.0.9 (inc-rollback-001)\n",
            encoding="utf-8",
        )
        verify_public_stable_docs(
            facts_path=facts_path,
            document_path=document,
        )
        document.write_text(
            "Active stable version: 0.1.0\nWithdrawn stable versions: 0.0.9\n",
            encoding="utf-8",
        )
        try:
            verify_public_stable_docs(
                facts_path=facts_path,
                document_path=document,
            )
        except GovernanceError:
            pass
        else:
            raise AssertionError("public docs accepted a missing incident id")


def _write_smoke_curl(path: Path) -> None:
    path.write_text(
        """#!/usr/bin/env bash
set -euo pipefail
output=""
url=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    -o|--output) output="$2"; shift 2 ;;
    -H|--connect-timeout|--max-time) shift 2 ;;
    -*) shift ;;
    *) url="$1"; shift ;;
  esac
done
url="${url%%\\?*}"
case "${url}" in
  */releases/download/channels/channels.json)
    source="${SMOKE_INDEX}"
    ;;
  https://sifr.sh/install)
    source="${SMOKE_DISPATCHERS}/index"
    ;;
  https://sifr.sh/install/stable)
    source="${SMOKE_DISPATCHERS}/stable"
    ;;
  https://sifr.sh/releases/stable)
    source="${SMOKE_DOCS}"
    ;;
  */releases/download/*/*)
    source="${SMOKE_ASSETS}/${url##*/}"
    ;;
  *)
    echo "unexpected smoke URL: ${url}" >&2
    exit 2
    ;;
esac
cp "${source}" "${output}"
""",
        encoding="utf-8",
    )
    path.chmod(0o755)


if __name__ == "__main__":
    raise SystemExit(run_self_tests())
