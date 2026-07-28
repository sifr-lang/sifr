"""Self-tests for exact stable publication staging and sign-off."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from collections.abc import Callable
from pathlib import Path
from typing import Any

from .common import (
    GovernanceError,
    canonical_json_bytes,
    load_json_strict,
    sha256_file,
    write_canonical_json,
)
from .stable_prepare_selftest import StablePrepareFixture, prepare
from .stable_publish_fixture import (
    fixture_paths,
    run_command,
    stage_call,
    stage_fixture,
)
from .stable_publish import (
    PLAN_ASSET_NAME,
    SMOKE_FILES,
    materialize_stable_signoff,
    stage_stable_publication,
)
from scripts.distribution.verify_marketplace_vsix import verify_marketplace_vsix
from .stable_orchestrator_selftest import (
    test_orchestrator_rejects_unmerged_candidate,
)

REPO_ROOT = Path(__file__).resolve().parents[4]


def run_self_tests() -> int:
    tests = (
        test_stage_and_signoff,
        test_stage_rejects_input_drift,
        test_signoff_rejects_public_drift,
        test_marketplace_adapter,
        test_github_release_adapter,
        test_orchestrator_rejects_unprotected_ref,
        test_orchestrator_rejects_unmerged_candidate,
        test_cli_producer,
    )
    for test in tests:
        test()
        print(f"stable-publication pass: {test.__name__}")
    print(f"stable publication self-tests ok: tests={len(tests)}")
    return 0


def test_stage_and_signoff() -> None:
    with StablePrepareFixture() as context:
        paths = stage_fixture(context)
        summary = paths["summary"]
        staged = paths["staged"]
        assets = staged / "release-assets"
        qualification = load_json_strict(
            paths["qualification"],
            require_canonical=True,
        )
        assert {path.name for path in assets.iterdir()} == {
            *(artifact["name"] for artifact in qualification["artifacts"]),
            PLAN_ASSET_NAME,
        }
        assert (staged / "channels.json").read_bytes() == canonical_json_bytes(
            summary["mutation"]["proposed_index"]
        )
        smoke = _write_smoke(context, summary, staged)
        site_run = _write_site_run(context, summary)
        signoff = materialize_stable_signoff(
            prepare_summary_path=paths["prepare"],
            release_assets_root=assets,
            site_facts_path=staged / "stable-site-release-facts.json",
            site_run_path=site_run,
            smoke_root=smoke,
            run_id=99,
            approver="release-reviewer",
        )
        assert signoff["version"] == "0.1.0"
        assert signoff["attempts"][0]["run_id"] == 99
        assert signoff["attempts"][0]["status"] == "completed"
        assert len(signoff["published_assets"]) == 21
        assert len(signoff["post_publication_smoke"]) == len(SMOKE_FILES)


def test_stage_rejects_input_drift() -> None:
    with StablePrepareFixture() as context:
        paths = fixture_paths(context)
        summary = prepare(context)
        write_canonical_json(paths["prepare"], summary, refuse_existing=True)
        plan = paths["plan"]
        plan_bytes = plan.read_bytes()
        plan.write_bytes(plan_bytes + b"\n")
        _expect_rejected(lambda: stage_call(context, paths, "plan-drift"))
        plan.write_bytes(plan_bytes)

        dispatcher = context["dispatcher_root"] / "stable"
        dispatcher_bytes = dispatcher.read_bytes()
        dispatcher.write_bytes(dispatcher_bytes + b"drift")
        _expect_rejected(lambda: stage_call(context, paths, "dispatcher-drift"))
        dispatcher.write_bytes(dispatcher_bytes)

        artifact = next(
            path
            for path in context["artifact_root"].rglob("*")
            if path.is_file()
        )
        artifact_bytes = artifact.read_bytes()
        artifact.write_bytes(artifact_bytes + b"drift")
        _expect_rejected(lambda: stage_call(context, paths, "artifact-drift"))
        artifact.write_bytes(artifact_bytes)

        existing = context["root"] / "existing"
        existing.mkdir()
        _expect_rejected(
            lambda: stage_stable_publication(
                prepare_summary_path=paths["prepare"],
                qualification_index_path=paths["qualification"],
                artifact_root=context["artifact_root"],
                plan_path=paths["plan"],
                dispatcher_root=context["dispatcher_root"],
                output_root=existing,
            )
        )


def test_signoff_rejects_public_drift() -> None:
    with StablePrepareFixture() as context:
        paths = stage_fixture(context)
        summary = paths["summary"]
        staged = paths["staged"]
        smoke = _write_smoke(context, summary, staged)
        site_run = _write_site_run(context, summary)
        arguments = {
            "prepare_summary_path": paths["prepare"],
            "release_assets_root": staged / "release-assets",
            "site_facts_path": staged / "stable-site-release-facts.json",
            "site_run_path": site_run,
            "smoke_root": smoke,
            "run_id": 99,
            "approver": "release-reviewer",
        }
        asset = next((staged / "release-assets").iterdir())
        asset_bytes = asset.read_bytes()
        asset.write_bytes(asset_bytes + b"drift")
        _expect_rejected(lambda: materialize_stable_signoff(**arguments))
        asset.write_bytes(asset_bytes)

        marketplace = smoke / "marketplace.vsix"
        marketplace.write_bytes(marketplace.read_bytes() + b"drift")
        _expect_rejected(lambda: materialize_stable_signoff(**arguments))


def test_marketplace_adapter() -> None:
    with StablePrepareFixture() as context:
        summary = prepare(context)
        marketplace = summary["marketplace"]
        vsix = (
            context["artifact_root"]
            / summary["artifacts"]["vsix"]["workflow_artifact_name"]
            / summary["artifacts"]["vsix"]["name"]
        )
        verify_marketplace_vsix(
            vsix_path=vsix,
            expected_sha256=marketplace["vsix_sha256"],
            publisher=marketplace["publisher"],
            extension=marketplace["extension"],
            version=marketplace["version"],
            compiler_version="0.1.0",
        )
        _expect_rejected(
            lambda: verify_marketplace_vsix(
                vsix_path=vsix,
                expected_sha256=marketplace["vsix_sha256"],
                publisher=marketplace["publisher"],
                extension="other-extension",
                version=marketplace["version"],
            )
        )
        _expect_rejected(
            lambda: verify_marketplace_vsix(
                vsix_path=vsix,
                expected_sha256=marketplace["vsix_sha256"],
                publisher=marketplace["publisher"],
                extension=marketplace["extension"],
                version=marketplace["version"],
                compiler_version="0.2.0",
            )
        )

        fake_bin = context["root"] / "fake-bin"
        fake_bin.mkdir()
        server = context["root"] / "marketplace-server.vsix"
        marker = context["root"] / "npx-called"
        _write_marketplace_commands(fake_bin)
        environment = os.environ.copy()
        environment.update(
            {
                "PATH": f"{fake_bin}:{environment['PATH']}",
                "MARKETPLACE_SERVER": str(server),
                "NPX_MARKER": str(marker),
                "VSCE_BIN": str(fake_bin / "vsce"),
            }
        )
        shutil.copyfile(vsix, server)
        reused = context["root"] / "reused.vsix"
        run_command(_marketplace_command(vsix, marketplace, reused), env=environment)
        assert reused.read_bytes() == vsix.read_bytes()
        assert not marker.exists()

        server.write_bytes(b"foreign Marketplace bytes")
        drifted = context["root"] / "drifted.vsix"
        rejected = subprocess.run(
            _marketplace_command(vsix, marketplace, drifted),
            cwd=REPO_ROOT,
            env=environment,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        assert rejected.returncode != 0
        assert not drifted.exists()

        server.unlink()
        published = context["root"] / "published.vsix"
        environment["VSCE_PAT"] = "fixture-token"
        run_command(_marketplace_command(vsix, marketplace, published), env=environment)
        assert published.read_bytes() == vsix.read_bytes()
        assert marker.read_text(encoding="utf-8") == "called\n"


def test_github_release_adapter() -> None:
    with StablePrepareFixture() as context:
        paths = stage_fixture(context)
        fake_bin = context["root"] / "fake-gh-bin"
        fake_bin.mkdir()
        state_root = context["root"] / "fake-gh-state"
        state_root.mkdir()
        _write_fake_gh(fake_bin / "gh")
        environment = os.environ.copy()
        environment.update(
            {
                "PATH": f"{fake_bin}:{environment['PATH']}",
                "FAKE_GH_STATE": str(state_root),
            }
        )
        notes = (
            context["evidence_root"]
            / context["candidate_path"]
            / "release-notes.md"
        )
        assets = paths["staged"] / "release-assets"
        initial_output = context["root"] / "initial-assets.json"
        command = _github_release_command(
            context,
            assets=assets,
            notes=notes,
            mode="initial",
            output=initial_output,
        )
        run_command(command, env=environment)
        expected = {
            path.name: sha256_file(path)
            for path in sorted(assets.iterdir())
        }
        assert load_json_strict(
            initial_output,
            require_canonical=True,
        ) == expected

        duplicate_initial = subprocess.run(
            _github_release_command(
                context,
                assets=assets,
                notes=notes,
                mode="initial",
                output=context["root"] / "rejected-initial.json",
            ),
            cwd=REPO_ROOT,
            env=environment,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        assert duplicate_initial.returncode != 0

        state = _load_fake_gh_state(state_root)
        missing_name = sorted(state["assets"])[0]
        Path(state["assets"][missing_name]["path"]).unlink()
        state["assets"].pop(missing_name)
        _write_fake_gh_state(state_root, state)
        resume_output = context["root"] / "resume-assets.json"
        run_command(
            _github_release_command(
                context,
                assets=assets,
                notes=notes,
                mode="resume",
                output=resume_output,
            ),
            env=environment,
        )
        assert load_json_strict(
            resume_output,
            require_canonical=True,
        ) == expected

        state = _load_fake_gh_state(state_root)
        drift_name = sorted(state["assets"])[0]
        Path(state["assets"][drift_name]["path"]).write_bytes(b"remote drift")
        rejected = subprocess.run(
            _github_release_command(
                context,
                assets=assets,
                notes=notes,
                mode="resume",
                output=context["root"] / "rejected-drift.json",
            ),
            cwd=REPO_ROOT,
            env=environment,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        assert rejected.returncode != 0


def test_orchestrator_rejects_unprotected_ref() -> None:
    with StablePrepareFixture() as context:
        paths = fixture_paths(context)
        write_canonical_json(paths["prepare"], prepare(context), refuse_existing=True)
        environment = os.environ.copy()
        environment.update({"SITE_TOKEN": "fixture", "VSCE_BIN": "/usr/bin/true"})
        command = [
            str(REPO_ROOT / "scripts/distribution/run_stable_publication.sh"),
            "--operation",
            "ga-activation",
            "--mode",
            "initial",
            "--repository",
            "sifr-lang/sifr",
            "--evidence-root",
            str(context["evidence_root"]),
            "--evidence-commit",
            context["evidence_commit"],
            "--candidate-path",
            context["candidate_path"],
            "--expected-plan-sha256",
            context["expected_plan_sha256"],
            "--source-root",
            str(context["source_root"]),
            "--prepare-summary",
            str(paths["prepare"]),
            "--expected-summary-sha256",
            sha256_file(paths["prepare"]),
            "--workflow-ref",
            "refs/heads/unprotected",
            "--workflow-commit",
            "a" * 40,
            "--run-id",
            "99",
            "--run-attempt",
            "1",
            "--initiator",
            "initiator",
            "--site-repository",
            "sifr-lang/sifr-website",
            "--site-workflow",
            "release-site.yml",
            "--site-workflow-ref",
            "stable-site",
            "--site-ruleset-id",
            "1",
            "--site-ruleset-updated-at",
            "2099-01-01T00:00:00Z",
            "--site-workflow-sha256",
            "b" * 64,
        ]
        completed = subprocess.run(
            command,
            cwd=REPO_ROOT,
            env=environment,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        assert completed.returncode != 0
        assert not (REPO_ROOT / "stable-publication-work").exists()


def test_cli_producer() -> None:
    with StablePrepareFixture() as context:
        paths = fixture_paths(context)
        write_canonical_json(paths["prepare"], prepare(context), refuse_existing=True)
        staged = context["root"] / "cli-staged"
        command = [
            "python3",
            str(REPO_ROOT / "scripts/distribution/materialize_stable_publication.py"),
            "stage",
            "--prepare-summary",
            str(paths["prepare"]),
            "--qualification-index",
            str(paths["qualification"]),
            "--artifact-root",
            str(context["artifact_root"]),
            "--plan",
            str(paths["plan"]),
            "--dispatchers",
            str(context["dispatcher_root"]),
            "--out",
            str(staged),
        ]
        run_command(command)
        smoke = _write_smoke(
            context,
            load_json_strict(paths["prepare"], require_canonical=True),
            staged,
        )
        summary = load_json_strict(paths["prepare"], require_canonical=True)
        site_run = _write_site_run(context, summary)
        signoff = context["root"] / "stable-release-signoff.json"
        run_command(
            [
                "python3",
                str(
                    REPO_ROOT
                    / "scripts/distribution/materialize_stable_publication.py"
                ),
                "signoff",
                "--prepare-summary",
                str(paths["prepare"]),
                "--release-assets",
                str(staged / "release-assets"),
                "--site-facts",
                str(staged / "stable-site-release-facts.json"),
                "--site-run",
                str(site_run),
                "--smoke",
                str(smoke),
                "--run-id",
                "99",
                "--approver",
                "release-reviewer",
                "--out",
                str(signoff),
            ]
        )
        assert json.loads(signoff.read_text(encoding="utf-8"))["version"] == "0.1.0"
        failed = subprocess.run(
            command,
            cwd=REPO_ROOT,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        assert failed.returncode != 0


def _write_smoke(
    context: dict[str, Any],
    summary: dict[str, Any],
    staged: Path,
) -> Path:
    smoke = context["root"] / f"smoke-{len(list(context['root'].glob('smoke-*')))}"
    smoke.mkdir()
    shutil.copyfile(staged / "channels.json", smoke / "governed-index.json")
    (smoke / "install-dispatcher").write_bytes(
        (context["dispatcher_root"] / "index").read_bytes()
    )
    (smoke / "stable-dispatcher").write_bytes(
        (context["dispatcher_root"] / "stable").read_bytes()
    )
    (smoke / "stable-release-docs.html").write_text(
        f"Active stable version: {summary['version']}\n"
        "Withdrawn stable versions: none.\n",
        encoding="utf-8",
    )
    published_assets = {
        path.name: sha256_file(path)
        for path in (staged / "release-assets").iterdir()
    }
    write_canonical_json(
        smoke / "version-assets.json",
        dict(sorted(published_assets.items())),
        refuse_existing=True,
    )
    write_canonical_json(
        smoke / "installed-self-update.json",
        {
            "schema_version": 2,
            "current_version": summary["version"],
            "target_version": summary["version"],
            "status": "up-to-date",
        },
        refuse_existing=True,
    )
    vsix = next(
        artifact
        for artifact in summary["artifacts"].values()
        if artifact["sha256"] == summary["marketplace"]["vsix_sha256"]
    )
    source = (
        context["artifact_root"]
        / vsix["workflow_artifact_name"]
        / vsix["name"]
    )
    shutil.copyfile(source, smoke / "marketplace.vsix")
    return smoke


def _write_site_run(
    context: dict[str, Any],
    summary: dict[str, Any],
) -> Path:
    path = context["root"] / (
        f"site-run-{len(list(context['root'].glob('site-run-*')))}.json"
    )
    write_canonical_json(
        path,
        {
            "repository": "sifr-lang/sifr-website",
            "workflow": "release-site.yml",
            "run_id": 123,
            "deployed_commit": summary["site"]["base_commit"],
        },
        refuse_existing=True,
    )
    return path


def _expect_rejected(operation: Callable[[], object]) -> None:
    try:
        operation()
    except (GovernanceError, OSError, ValueError):
        return
    raise AssertionError("drifted stable publication unexpectedly passed")


def _marketplace_command(
    vsix: Path,
    marketplace: dict[str, str],
    output: Path,
) -> list[str]:
    return [
        str(REPO_ROOT / "scripts/distribution/publish_marketplace_extension.sh"),
        "--package",
        str(vsix),
        "--publisher",
        marketplace["publisher"],
        "--extension",
        marketplace["extension"],
        "--version",
        marketplace["version"],
        "--expected-sha256",
        marketplace["vsix_sha256"],
        "--verified-out",
        str(output),
    ]


def _github_release_command(
    context: dict[str, Any],
    *,
    assets: Path,
    notes: Path,
    mode: str,
    output: Path,
) -> list[str]:
    return [
        "python3",
        str(REPO_ROOT / "scripts/distribution/publish_stable_release.py"),
        "--repository",
        "sifr-lang/sifr",
        "--version",
        "0.1.0",
        "--source-commit",
        context["source_commit"],
        "--mode",
        mode,
        "--assets",
        str(assets),
        "--notes",
        str(notes),
        "--out",
        str(output),
    ]


def _write_fake_gh(path: Path) -> None:
    path.write_text(
        """#!/usr/bin/env python3
import json
import os
import shutil
import sys
from pathlib import Path

root = Path(os.environ["FAKE_GH_STATE"])
state_path = root / "state.json"
state = json.loads(state_path.read_text()) if state_path.exists() else {
    "release": None,
    "tag": None,
    "assets": {},
    "next_id": 10,
}

def save():
    state_path.write_text(json.dumps(state, sort_keys=True))

args = sys.argv[1:]
if args[0] == "api":
    endpoint = args[-1]
    if "/releases/tags/" in endpoint:
        value = state["release"]
    elif "/git/ref/tags/" in endpoint:
        value = state["tag"]
    elif "/releases/assets/" in endpoint:
        asset_id = int(endpoint.rsplit("/", 1)[1])
        for asset in state["assets"].values():
            if asset["id"] == asset_id:
                sys.stdout.buffer.write(Path(asset["path"]).read_bytes())
                raise SystemExit(0)
        value = None
    elif "/assets?" in endpoint:
        page = int(endpoint.rsplit("page=", 1)[1])
        value = (
            [
                {"id": asset["id"], "name": name}
                for name, asset in sorted(state["assets"].items())
            ]
            if page == 1
            else []
        )
    else:
        value = None
    if value is None:
        print("HTTP 404: Not Found", file=sys.stderr)
        raise SystemExit(1)
    print(json.dumps(value))
elif args[:2] == ["release", "create"]:
    version = args[2]
    source = args[args.index("--target") + 1]
    state["release"] = {
        "id": 1,
        "tag_name": version,
        "target_commitish": source,
        "draft": False,
        "prerelease": False,
    }
    state["tag"] = {"object": {"sha": source, "type": "commit"}}
    save()
elif args[:2] == ["release", "upload"]:
    for value in args[3:args.index("--repo")]:
        source = Path(value)
        destination = root / f"asset-{state['next_id']}"
        shutil.copyfile(source, destination)
        state["assets"][source.name] = {
            "id": state["next_id"],
            "path": str(destination),
        }
        state["next_id"] += 1
    save()
else:
    print(f"unsupported fake gh command: {args}", file=sys.stderr)
    raise SystemExit(2)
""",
        encoding="utf-8",
    )
    path.chmod(0o755)


def _load_fake_gh_state(root: Path) -> dict[str, Any]:
    return json.loads((root / "state.json").read_text(encoding="utf-8"))


def _write_fake_gh_state(root: Path, state: dict[str, Any]) -> None:
    (root / "state.json").write_text(
        json.dumps(state, sort_keys=True),
        encoding="utf-8",
    )


def _write_marketplace_commands(fake_bin: Path) -> None:
    curl = fake_bin / "curl"
    curl.write_text(
        """#!/usr/bin/env bash
set -euo pipefail
output=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output) output="$2"; shift 2 ;;
    --write-out) shift 2 ;;
    *) shift ;;
  esac
done
if [[ -f "${MARKETPLACE_SERVER}" ]]; then
  cp "${MARKETPLACE_SERVER}" "${output}"
  printf '200'
else
  : >"${output}"
  printf '404'
fi
""",
        encoding="utf-8",
    )
    vsce = fake_bin / "vsce"
    vsce.write_text(
        """#!/usr/bin/env bash
set -euo pipefail
package=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --packagePath) package="$2"; shift 2 ;;
    *) shift ;;
  esac
done
cp "${package}" "${MARKETPLACE_SERVER}"
printf 'called\\n' >"${NPX_MARKER}"
""",
        encoding="utf-8",
    )
    curl.chmod(0o755)
    vsce.chmod(0o755)


if __name__ == "__main__":
    raise SystemExit(run_self_tests())
