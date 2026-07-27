"""Validate stable editor, compiler-range, and Marketplace publication plan."""

from __future__ import annotations

import re
from typing import Any

from .common import (
    TARGETS,
    fail,
    require_nonempty_string,
    require_object,
    require_sha256,
)

COMPILER_RANGE_RE = re.compile(
    r"^>=([0-9]+)\.([0-9]+)\.([0-9]+),<([0-9]+)\.([0-9]+)\.([0-9]+)$"
)


def validate_editor_report(
    payload: Any,
    *,
    source_commit: str,
    submodule_commit: str | None,
    candidate_version: str,
    rollback_version: str,
) -> dict[str, Any]:
    report = require_object(payload, "editor qualification report")
    required = {
        "schema_version",
        "kind",
        "source_commit",
        "submodule_commit",
        "package_path",
        "package_version",
        "compiler_compatibility",
        "candidate_version",
        "rollback_version",
        "candidate_target",
        "candidate_binary_sha256",
        "target_report_sha256",
        "vsix_sha256",
        "vsix_package_smoke",
        "lsp_smoke",
        "marketplace_publish_plan",
        "status",
    }
    if set(report) != required:
        fail("$.vscode.validation_report_sha256", "editor report fields are not exact")
    if (
        report["schema_version"] != 2
        or report["kind"] != "stable-editor-qualification"
        or report["status"] != "pass"
        or report["source_commit"] != source_commit
        or report["submodule_commit"] != submodule_commit
        or report["candidate_version"] != candidate_version
        or report["rollback_version"] != rollback_version
        or report["candidate_target"] not in TARGETS
        or report["vsix_package_smoke"] != "pass"
        or report["lsp_smoke"] != "pass"
    ):
        fail("$.vscode.validation_report_sha256", "editor report identity did not pass")
    for field in ("package_path", "package_version", "compiler_compatibility"):
        require_nonempty_string(report[field], f"editor qualification report.{field}")
    if not compiler_range_contains(
        report["compiler_compatibility"],
        candidate_version,
    ):
        fail(
            "$.vscode.compiler_compatibility",
            "does not contain the stable candidate",
        )
    if rollback_version != "none" and not compiler_range_contains(
        report["compiler_compatibility"],
        rollback_version,
    ):
        fail(
            "$.vscode.compiler_compatibility",
            "does not contain the rollback target",
        )
    require_sha256(
        report["candidate_binary_sha256"],
        "editor qualification report.candidate_binary_sha256",
    )
    require_sha256(
        report["target_report_sha256"],
        "editor qualification report.target_report_sha256",
    )
    require_sha256(report["vsix_sha256"], "editor qualification report.vsix_sha256")
    validate_marketplace_publish_plan(report)
    return report


def validate_marketplace_publish_plan(report: dict[str, Any]) -> None:
    marketplace = require_object(
        report["marketplace_publish_plan"],
        "editor qualification report.marketplace_publish_plan",
    )
    if set(marketplace) != {
        "publisher",
        "extension",
        "version",
        "package_path",
        "vsix_sha256",
        "command",
        "rebuild",
        "execution_owner",
        "status",
    }:
        fail(
            "$.vscode.validation_report_sha256",
            "Marketplace publication-plan fields are not exact",
        )
    expected_vsix = f"sifr-vscode-{report['package_version']}.vsix"
    if (
        marketplace["publisher"] != "sifr"
        or marketplace["extension"] != "sifr-vscode"
        or marketplace["version"] != report["package_version"]
        or marketplace["package_path"] != expected_vsix
        or marketplace["vsix_sha256"] != report["vsix_sha256"]
        or marketplace["command"]
        != [
            "npx",
            "--no-install",
            "vsce",
            "publish",
            "--packagePath",
            expected_vsix,
        ]
        or marketplace["rebuild"] is not False
        or marketplace["execution_owner"] != "stable-publication-workflow"
        or marketplace["status"] != "planned"
    ):
        fail(
            "$.vscode.validation_report_sha256",
            "Marketplace plan must bind protected publication to the exact VSIX without rebuilding",
        )


def compiler_range_contains(expression: str, version: str) -> bool:
    range_match = COMPILER_RANGE_RE.fullmatch(expression)
    version_match = re.fullmatch(
        r"([0-9]+)\.([0-9]+)\.([0-9]+)",
        version,
    )
    if range_match is None or version_match is None:
        return False
    bounds = tuple(int(part) for part in range_match.groups())
    candidate = tuple(int(part) for part in version_match.groups())
    return bounds[:3] <= candidate < bounds[3:]
