"""Repository guard for the atomic release-governance schema-v2 cutover."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "distribution_release"
GOVERNED_FILES = (
    REPO_ROOT / ".github" / "workflows" / "preview-release.yml",
    REPO_ROOT / "docs" / "self_update.md",
    REPO_ROOT / "internal_docs" / "distribution_pipeline.md",
)
GOVERNED_GLOBS = (
    ("crates/sifr/src", "self_update*.rs"),
    ("scripts/distribution", "*.py"),
    ("scripts/distribution", "*.sh"),
    ("verification/areas/distribution_release/cases", "*.sh"),
    ("verification/areas/distribution_release/tools", "*"),
    ("verification/areas/distribution_release/governance", "*.py"),
)
SCAN_EXCLUSIONS = {"schema_epoch.py", "selftest.py", "schema_contracts.py"}
# The archive/receipt contract starts its own version history, outside the v2
# release-index cutover. Keep this exemption limited to its two literal owners.
ARCHIVE_SOURCE_EXCLUSIONS = {
    AREA_ROOT / "governance" / "archive_store.py",
    AREA_ROOT / "governance" / "archive_offline_selftest.py",
}
# These exact lines own independent v1 formats: the stdlib metadata descriptor,
# the private DX trace, and the native package qualification report. Continue
# scanning every other line in these sources for release-governance v1 paths.
NON_RELEASE_V1_LINES = {
    REPO_ROOT / "scripts" / "distribution" / "metadata_artifact.py": {
        '    if not isinstance(descriptor,dict) or set(descriptor) != FIELDS or type(descriptor.get("schema_version")) is not int or descriptor["schema_version"] != 1:',
        '    descriptor = {"schema_version": 1, "compiler_identity": identity, "semantic_target": target,',
    },
    REPO_ROOT / "scripts" / "distribution" / "native_dxf_contracts.py": {
        '    require(len(data) <= 32768 and trace["schema_version"] == 1',
    },
    REPO_ROOT / "scripts" / "distribution" / "qualify_native_package.py": {
        '        self.report = {"schema_version": 1, "source_commit": source, "target": target,',
    },
    AREA_ROOT / "governance" / "qualification_fixture_support.py": {
        '    descriptor = {"schema_version": 1, "compiler_identity": compiler, "semantic_target": target,',
    },
}
V1_PATTERNS = (
    re.compile(r'"schema_version"\s*:\s*1(?:\D|$)'),
    re.compile(r"(?<![A-Za-z0-9_])schema_version\s+must\s+be\s+1\b"),
    re.compile(r"(?<![A-Za-z0-9_])schema_version[^\n]{0,40}==\s*1\b"),
)


def check_schema_epoch() -> None:
    for path in sorted((AREA_ROOT / "schemas").glob("*.schema.json")):
        schema = json.loads(path.read_text(encoding="utf-8"))
        check_schema_declaration(path, schema)
    for path in governed_sources():
        check_source_text(path, path.read_text(encoding="utf-8"))
    if (REPO_ROOT / "scripts" / "distribution" / "bootstrap_channel_metadata.py").exists():
        raise ValueError("schema-v1 bootstrap producer must not exist")


def check_schema_declaration(path: Path, schema: dict) -> None:
    expected = {"const": 2}
    if path == AREA_ROOT / "schemas" / "release_evidence_archive.schema.json":
        expected = {"const": 1, "type": "integer"}
    if "schema_version" not in schema.get("required", []):
        raise ValueError(f"{path}: schema_version is not required")
    if schema.get("properties", {}).get("schema_version") != expected:
        raise ValueError(f"{path}: schema_version must be exactly {expected}")
    if "default" in json.dumps(schema.get("properties", {}).get("schema_version")):
        raise ValueError(f"{path}: schema_version must not have a default")


def governed_sources() -> list[Path]:
    paths = set(GOVERNED_FILES)
    for root_text, pattern in GOVERNED_GLOBS:
        root = REPO_ROOT / root_text
        paths.update(
            path
            for path in root.glob(pattern)
            if path.is_file() and path.name not in SCAN_EXCLUSIONS
            and path not in ARCHIVE_SOURCE_EXCLUSIONS
        )
    required = {
        REPO_ROOT / "crates" / "sifr" / "src" / "self_update_receipt.rs",
        REPO_ROOT / "scripts" / "distribution" / "generate_version_installer.sh",
        REPO_ROOT
        / "verification"
        / "areas"
        / "distribution_release"
        / "tools"
        / "validate_self_update_metadata.sh",
    }
    missing = required.difference(paths)
    if missing:
        raise ValueError(f"governed schema scan omitted required surface(s): {sorted(missing)}")
    return sorted(paths)


def check_source_text(path: Path, text: str) -> None:
    independent_lines = NON_RELEASE_V1_LINES.get(path, set())
    scanned_text = "\n".join(
        "" if line in independent_lines else line for line in text.splitlines()
    )
    for pattern in V1_PATTERNS:
        if pattern.search(scanned_text):
            raise ValueError(f"{path}: retained a release-governance schema-v1 code path")


def run_self_test() -> None:
    invalid = (
        '{"schema_version": 1}',
        "schema_version must be 1",
        "if schema_version == 1:",
        '"schema_version"\n: 1',
    )
    for index, text in enumerate(invalid):
        try:
            check_source_text(Path(f"mutation-{index}"), text)
        except ValueError:
            continue
        raise ValueError(f"schema epoch mutation {index} unexpectedly passed")
    for path, independent_lines in NON_RELEASE_V1_LINES.items():
        if path not in governed_sources():
            raise ValueError(f"{path}: independent schema exception escaped the governed scan")
        for line in independent_lines:
            check_source_text(path, line)
            try:
                check_source_text(path, line + '\n{"schema_version": 1}')
            except ValueError:
                continue
            raise ValueError(f"{path}: release schema-v1 mutation escaped the governed scan")


def main() -> int:
    try:
        check_schema_epoch()
        run_self_test()
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        print(f"release-governance-schema-epoch: {exc}", file=sys.stderr)
        return 2
    print("Release-governance schema epoch ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
