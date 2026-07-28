"""Executed self-tests for the post-publication incident recovery adapter."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
RECOVERY = REPO_ROOT / "scripts/distribution/run_incident_public_recovery.sh"


def run_self_tests() -> int:
    tests = (
        test_rollback_and_roll_forward_recovery,
        test_recovery_rejects_binary_and_receipt_drift,
    )
    for test in tests:
        test()
        print(f"incident-public-recovery pass: {test.__name__}")
    print(f"incident public recovery self-tests ok: tests={len(tests)}")
    return 0


def test_rollback_and_roll_forward_recovery() -> None:
    for operation in ("rollback", "incident-roll-forward"):
        with tempfile.TemporaryDirectory(
            prefix=f"sifr-public-recovery-{operation}-"
        ) as temporary:
            root = Path(temporary)
            environment, working, broken, dispatcher, affected, successor = _fixture(
                root,
                operation=operation,
            )
            output = root / "recovery.json"
            _run_recovery(
                operation=operation,
                working=working,
                broken=broken,
                dispatcher=dispatcher,
                output=output,
                environment=environment,
                affected_version=affected,
                successor_version=successor,
            )
            evidence = json.loads(output.read_text(encoding="utf-8"))
            assert evidence == {
                "affected_version": affected,
                "operation": operation,
                "out_of_band": "pass",
                "schema_version": 2,
                "successor_version": successor,
                "working_client": "pass",
            }
            assert (working / "install.json").is_file()
            assert (broken / "install.json").is_file()
            assert not (working / "install-receipt.json").exists()
            if operation == "rollback":
                assert "--force" in (
                    working / "without-force.txt"
                ).read_text(encoding="utf-8")
                assert (working / "with-force.txt").is_file()
            else:
                assert (working / "roll-forward.txt").is_file()
            assert (broken / "out-of-band.txt").is_file()


def test_recovery_rejects_binary_and_receipt_drift() -> None:
    for drift, environment_key in (
        ("binary", "BINARY_VERSION"),
        ("receipt", "RECEIPT_VERSION"),
    ):
        with tempfile.TemporaryDirectory(
            prefix=f"sifr-public-recovery-{drift}-drift-"
        ) as temporary:
            root = Path(temporary)
            (
                environment,
                working,
                broken,
                dispatcher,
                affected,
                successor,
            ) = _fixture(
                root,
                operation="incident-roll-forward",
            )
            environment[environment_key] = "0.1.2"
            completed = _run_recovery(
                operation="incident-roll-forward",
                working=working,
                broken=broken,
                dispatcher=dispatcher,
                output=root / "recovery.json",
                environment=environment,
                affected_version=affected,
                successor_version=successor,
                check=False,
            )
            assert completed.returncode != 0
            assert not (root / "recovery.json").exists()


def _fixture(
    root: Path,
    *,
    operation: str,
) -> tuple[dict[str, str], Path, Path, Path, str, str]:
    if operation == "rollback":
        affected_version, successor_version = "0.1.1", "0.1.0"
    else:
        affected_version, successor_version = "0.1.0", "0.1.1"
    template = root / "fake-sifr"
    template.write_text(
        """#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${GH_TOKEN:-}" || -n "${SITE_TOKEN:-}" || -n "${VSCE_PAT:-}" ]]; then
  echo "production token reached recovery client" >&2
  exit 2
fi
root="${SIFR_SYSROOT_INSTALL_DIR:-$(cd "$(dirname "$0")/.." && pwd)}"
if [[ "${1:-}" == "--version" ]]; then
  printf 'sifr %s\\n' "$(cat "${root}/version.txt")"
  exit 0
fi
if [[ "${1:-}" == "self" && "${2:-}" == "update" ]]; then
  force=false
  for argument in "$@"; do
    [[ "${argument}" == "--force" ]] && force=true
  done
  current="$(cat "${root}/version.txt")"
  if [[ "$(printf '%s\\n%s\\n' "${INSTALLED_VERSION}" "${current}" | sort -V | head -n1)" == "${INSTALLED_VERSION}" &&
    "${INSTALLED_VERSION}" != "${current}" && "${force}" == "false" ]]; then
    echo "downgrade requires --force" >&2
    exit 2
  fi
  exec "${FAKE_INSTALLER}"
fi
echo "unsupported fake sifr command" >&2
exit 2
""",
        encoding="utf-8",
    )
    template.chmod(0o755)
    installer = root / "fake-installer"
    installer.write_text(
        """#!/usr/bin/env bash
set -euo pipefail
root="${SIFR_SYSROOT_INSTALL_DIR}"
mkdir -p "${SIFR_INSTALL_DIR}"
cp "${FAKE_SIFR_TEMPLATE}" "${SIFR_INSTALL_DIR}/sifr"
chmod +x "${SIFR_INSTALL_DIR}/sifr"
printf '%s\\n' "${BINARY_VERSION}" >"${root}/version.txt"
cat >"${root}/install.json" <<JSON
{
  "schema_version": 2,
  "name": "sifr",
  "version": "${RECEIPT_VERSION}",
  "channel": "stable",
  "target": "x86_64-unknown-linux-gnu",
  "install_dir": "${SIFR_INSTALL_DIR}",
  "binary_path": "${SIFR_INSTALL_DIR}/sifr",
  "sysroot_path": "${root}",
  "sysroot_schema_version": 1,
  "sysroot_sifr_version": "${RECEIPT_VERSION}",
  "sysroot_target_triple": "x86_64-unknown-linux-gnu",
  "sysroot_content_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
  "artifact": "sifr-${RECEIPT_VERSION}-x86_64-unknown-linux-gnu.tar.gz",
  "modify_path": false
}
JSON
""",
        encoding="utf-8",
    )
    installer.chmod(0o755)
    dispatcher = root / "stable-dispatcher"
    dispatcher.write_text(
        '#!/usr/bin/env bash\nset -euo pipefail\nexec "${FAKE_INSTALLER}" "$@"\n',
        encoding="utf-8",
    )
    dispatcher.chmod(0o755)
    working = root / "working"
    broken = root / "broken"
    for client in (working, broken):
        (client / "bin").mkdir(parents=True)
        shutil.copyfile(template, client / "bin/sifr")
        (client / "bin/sifr").chmod(0o755)
        (client / "version.txt").write_text(
            f"{affected_version}\n",
            encoding="utf-8",
        )
    environment = os.environ.copy()
    environment.update(
        {
            "FAKE_INSTALLER": str(installer),
            "FAKE_SIFR_TEMPLATE": str(template),
            "INSTALLED_VERSION": successor_version,
            "BINARY_VERSION": successor_version,
            "RECEIPT_VERSION": successor_version,
            "GH_TOKEN": "must-be-scrubbed",
            "SITE_TOKEN": "must-be-scrubbed",
            "VSCE_PAT": "must-be-scrubbed",
        }
    )
    return (
        environment,
        working,
        broken,
        dispatcher,
        affected_version,
        successor_version,
    )


def _run_recovery(
    *,
    operation: str,
    working: Path,
    broken: Path,
    dispatcher: Path,
    output: Path,
    environment: dict[str, str],
    affected_version: str,
    successor_version: str,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        [
            str(RECOVERY),
            "--operation",
            operation,
            "--affected-version",
            affected_version,
            "--successor-version",
            successor_version,
            "--working-root",
            str(working),
            "--broken-root",
            str(broken),
            "--stable-dispatcher",
            str(dispatcher),
            "--out",
            str(output),
        ],
        cwd=REPO_ROOT,
        env=environment,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if check and completed.returncode != 0:
        raise AssertionError(completed.stderr)
    return completed


if __name__ == "__main__":
    raise SystemExit(run_self_tests())
