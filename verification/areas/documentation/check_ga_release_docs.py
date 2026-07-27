#!/usr/bin/env python3
"""Validate the complete public stable-release documentation contract."""

from __future__ import annotations

import argparse
import copy
import importlib.util
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
FACTS_PATH = AREA_ROOT / "fixtures" / "stable_site_release_facts.json"
DOCS_CONFIG_PATH = REPO_ROOT / "docs" / "docs.json"
INTERNAL_DISTRIBUTION_PATH = REPO_ROOT / "internal_docs" / "distribution_pipeline.md"
RUST_CLAIMS_CHECK = (
    REPO_ROOT
    / "verification"
    / "areas"
    / "rust_interop"
    / "checks"
    / "check_stable_support_claims.py"
)
START_MARKER = "<!-- stable-release-facts:start -->"
END_MARKER = "<!-- stable-release-facts:end -->"
RENDERER_PATH = (
    REPO_ROOT / "scripts" / "distribution" / "render_stable_release_docs.py"
)

CANONICAL_DOCUMENTS = {
    "installation": REPO_ROOT / "docs" / "installation.mdx",
    "self-update": REPO_ROOT / "docs" / "self_update.md",
    "cli": REPO_ROOT / "docs" / "cli" / "overview.mdx",
    "stable-release": REPO_ROOT / "docs" / "releases" / "stable.mdx",
    "release-notes": REPO_ROOT / "docs" / "releases" / "0.1.0.mdx",
    "compatibility": REPO_ROOT / "docs" / "releases" / "compatibility.mdx",
    "support": REPO_ROOT / "docs" / "support.mdx",
    "troubleshooting": REPO_ROOT / "docs" / "troubleshooting.mdx",
    "rust-interop": REPO_ROOT / "docs" / "rust-interop.mdx",
}
PUBLIC_DOC_SUFFIXES = {".md", ".mdx"}
TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
)
NAVIGATION_PAGES = (
    "releases/stable",
    "releases/0.1.0",
    "releases/compatibility",
    "support",
    "troubleshooting",
)
REQUIRED_BY_DOCUMENT = {
    "installation": (
        "https://sifr.sh/install",
        "https://sifr.sh/install/stable",
        "sifr self update --channel stable",
        "--version 0.1.0",
        "macOS 15.0",
        "glibc 2.39",
        *TARGETS,
    ),
    "self-update": (
        "alpha|beta|stable",
        "schema-v2 governed release index",
        "stable `0.1.0`",
        "Withdrawn versions",
        "`rc` is not a public channel",
    ),
    "cli": (
        "sifr self version [--format text\\|json]",
        "sifr self update --channel stable",
        "sifr self update --version 0.1.0",
        "schema-v2 receipt",
        "withdrawn",
    ),
    "stable-release": (
        "schema_version: 2",
        "stable-site-release-facts.json",
        "https://sifr.sh/install/stable",
        "--version 0.1.0",
        "incident roll-forward",
        "`rc` is not a public channel",
    ),
    "release-notes": (
        "Sifr `0.1.0` is the first stable compiler release",
        "sifr.sifr-vscode` `0.2.0",
        ">=0.1.0,<0.2.0",
        "schema_version: 2",
        "https://sifr.sh/install/stable",
        "incident roll-forward",
    ),
    "compatibility": (
        "Sifr `0.1.0`",
        "macOS 15.0",
        "glibc 2.39",
        "sifr.sifr-vscode",
        "version `0.2.0`",
        ">=0.1.0,<0.2.0",
        "stable_support_claims.json",
        *TARGETS,
    ),
    "support": (
        "https://sifr.sh/install",
        "sifr self version --format json",
        "VS Code extension version",
        "withdraw a bad stable version",
        "incident-roll-forward",
    ),
    "troubleshooting": (
        "sifr self update --dry-run --format json",
        "https://sifr.sh/install/stable",
        ">=0.1.0,<0.2.0",
        "macOS 15.0",
        "glibc 2.39",
        "withdrawn",
    ),
    "rust-interop": (
        "stable_support_claims.json",
        "<!-- rust-interop-stable-claims:start -->",
        "<!-- rust-interop-stable-claims:end -->",
    ),
}
FORBIDDEN_CLAIMS = (
    "cryptographically signed",
    "notarized",
    "all Rust crates are supported",
    "Stable-channel self-update remains gated",
    "Self-update currently accepts only `alpha` and `beta`",
    "Stable channels and release-candidate channels are not yet available",
    "The beta channel is the recommended starting point",
    "Sifr is currently in preview.",
    "sifr self update --channel nightly",
    "-preview.2",
    "one immutable preview version",
    "aarch64-pc-windows-msvc",
    "x86_64-pc-windows-msvc",
)
MUTATION_CASES = (
    "missing-stable-entrypoint",
    "preview-only-docs",
    "unsupported-target-claim",
    "platform-floor-drift",
    "stable-version-drift",
    "schema-reference-drift",
    "release-facts-drift",
    "extension-range-drift",
    "missing-support-section",
    "unsupported-rust-claim",
    "signing-claim",
    "notarization-claim",
    "global-preview-claim",
)


class DocumentationError(ValueError):
    """A public stable-release documentation contract violation."""


def load_documents() -> dict[str, str]:
    documents: dict[str, str] = {}
    for name, path in CANONICAL_DOCUMENTS.items():
        try:
            documents[name] = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            raise DocumentationError(f"{name} document cannot be read: {exc}") from exc
    return documents


def load_public_documents() -> dict[str, str]:
    documents: dict[str, str] = {}
    for path in sorted((REPO_ROOT / "docs").rglob("*")):
        if not path.is_file() or path.suffix not in PUBLIC_DOC_SUFFIXES:
            continue
        name = str(path.relative_to(REPO_ROOT / "docs"))
        try:
            documents[name] = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            raise DocumentationError(
                f"public document {name} cannot be read: {exc}"
            ) from exc
    if not documents:
        raise DocumentationError("public documentation sweep found no documents")
    return documents


def load_facts() -> dict[str, Any]:
    if str(REPO_ROOT) not in sys.path:
        sys.path.insert(0, str(REPO_ROOT))
    from verification.areas.distribution_release.governance.common import (
        GovernanceError,
        load_json_strict,
    )
    from verification.areas.distribution_release.governance.release_plan import (
        validate_site_release_facts,
    )

    try:
        payload = load_json_strict(FACTS_PATH, require_canonical=True)
        validate_site_release_facts(payload)
    except (OSError, UnicodeError, json.JSONDecodeError, GovernanceError) as exc:
        raise DocumentationError(f"stable site facts cannot be read: {exc}") from exc
    if not isinstance(payload, dict):
        raise DocumentationError("stable site facts must be an object")
    expected_fields = {
        "schema_version",
        "generation",
        "stable_version",
        "stable_status",
        "source_plan_sha256",
        "release_index_sha256",
        "dispatchers",
        "withdrawals",
    }
    if set(payload) != expected_fields:
        raise DocumentationError("stable site facts fields drifted")
    if payload["schema_version"] != 2 or payload["stable_status"] != "active":
        raise DocumentationError("stable site facts must represent active schema-v2 GA")
    if payload["stable_version"] != "0.1.0":
        raise DocumentationError("stable site facts must identify release 0.1.0")
    withdrawals = payload["withdrawals"]
    if not isinstance(withdrawals, list):
        raise DocumentationError("stable site facts withdrawals must be an array")
    return payload


def render_facts_block(facts: dict[str, Any]) -> str:
    spec = importlib.util.spec_from_file_location(
        "sifr_stable_release_docs_renderer",
        RENDERER_PATH,
    )
    if spec is None or spec.loader is None:
        raise DocumentationError("stable documentation renderer cannot be loaded")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return str(module.render_facts_block(facts))


def validate_documents(
    documents: dict[str, str],
    *,
    public_documents: dict[str, str],
    facts: dict[str, Any],
    docs_config: str,
    internal_distribution: str,
) -> None:
    if set(documents) != set(CANONICAL_DOCUMENTS):
        raise DocumentationError("canonical GA document set drifted")
    for name, required_facts in REQUIRED_BY_DOCUMENT.items():
        text = documents[name]
        for fact in required_facts:
            if fact not in text:
                raise DocumentationError(f"{name} is missing governed GA fact: {fact}")

    stable_text = documents["stable-release"]
    if stable_text.count(START_MARKER) != 1 or stable_text.count(END_MARKER) != 1:
        raise DocumentationError("stable release facts must use exactly one marker pair")
    if render_facts_block(facts) not in stable_text:
        raise DocumentationError("stable release facts do not match the governed payload")

    combined = "\n".join(public_documents.values())
    for claim in FORBIDDEN_CLAIMS:
        if claim in combined:
            raise DocumentationError(f"forbidden or stale GA claim: {claim}")
    for page in NAVIGATION_PAGES:
        if f'"{page}"' not in docs_config:
            raise DocumentationError(f"docs navigation is missing {page}")

    internal_facts = (
        "schema_version: 2",
        "/install/stable",
        "alpha|beta|stable",
        "macOS 15.0",
        "glibc 2.39",
        *TARGETS,
    )
    for fact in internal_facts:
        if fact not in internal_distribution:
            raise DocumentationError(f"internal/public release contract drift: {fact}")


def canonical_inputs() -> tuple[
    dict[str, str],
    dict[str, str],
    dict[str, Any],
    str,
    str,
]:
    try:
        docs_config = DOCS_CONFIG_PATH.read_text(encoding="utf-8")
        internal_distribution = INTERNAL_DISTRIBUTION_PATH.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise DocumentationError(f"documentation contract input cannot be read: {exc}") from exc
    return (
        load_documents(),
        load_public_documents(),
        load_facts(),
        docs_config,
        internal_distribution,
    )


def run_self_test() -> None:
    (
        documents,
        public_documents,
        facts,
        docs_config,
        internal_distribution,
    ) = canonical_inputs()
    validate_documents(
        documents,
        public_documents=public_documents,
        facts=facts,
        docs_config=docs_config,
        internal_distribution=internal_distribution,
    )
    mutations = {
        "missing-stable-entrypoint": lambda docs: docs.__setitem__(
            "installation",
            docs["installation"].replace("https://sifr.sh/install/stable", ""),
        ),
        "preview-only-docs": lambda docs: docs.__setitem__(
            "installation",
            docs["installation"].replace(
                "Stable is the recommended channel",
                "The beta channel is the recommended starting point",
            ),
        ),
        "unsupported-target-claim": lambda docs: docs.__setitem__(
            "compatibility",
            docs["compatibility"] + "\n`aarch64-pc-windows-msvc`\n",
        ),
        "platform-floor-drift": lambda docs: docs.__setitem__(
            "compatibility",
            docs["compatibility"].replace("glibc 2.39", "glibc 2.38"),
        ),
        "stable-version-drift": lambda docs: docs.__setitem__(
            "stable-release",
            docs["stable-release"].replace(
                "Active stable version: `0.1.0`",
                "Active stable version: `0.1.1`",
            ),
        ),
        "schema-reference-drift": lambda docs: docs.__setitem__(
            "stable-release",
            docs["stable-release"].replace("schema_version: 2", "schema_version: 1"),
        ),
        "release-facts-drift": lambda docs: docs.__setitem__(
            "stable-release",
            docs["stable-release"].replace(
                "Withdrawn stable versions: none.",
                "Withdrawn stable versions: `0.1.0` (unattributed).",
            ),
        ),
        "extension-range-drift": lambda docs: docs.__setitem__(
            "compatibility",
            docs["compatibility"].replace(">=0.1.0,<0.2.0", ">=0.1.1,<0.2.0"),
        ),
        "missing-support-section": lambda docs: docs.__setitem__(
            "support",
            docs["support"].replace("incident-roll-forward", "unspecified recovery"),
        ),
        "unsupported-rust-claim": lambda docs: docs.__setitem__(
            "compatibility",
            docs["compatibility"] + "\nall Rust crates are supported\n",
        ),
        "signing-claim": lambda docs: docs.__setitem__(
            "stable-release",
            docs["stable-release"] + "\ncryptographically signed\n",
        ),
        "notarization-claim": lambda docs: docs.__setitem__(
            "stable-release",
            docs["stable-release"] + "\nnotarized\n",
        ),
    }
    if tuple(mutations) != MUTATION_CASES[:-1]:
        raise DocumentationError("GA documentation mutation registration drifted")
    for case_id, mutate in mutations.items():
        changed = copy.deepcopy(documents)
        mutate(changed)
        try:
            validate_documents(
                changed,
                public_documents={
                    **public_documents,
                    **{
                        str(CANONICAL_DOCUMENTS[name].relative_to(REPO_ROOT / "docs")): text
                        for name, text in changed.items()
                    },
                },
                facts=facts,
                docs_config=docs_config,
                internal_distribution=internal_distribution,
            )
        except DocumentationError:
            continue
        raise DocumentationError(f"GA documentation mutation unexpectedly passed: {case_id}")
    changed_public = copy.deepcopy(public_documents)
    changed_public["introduction.mdx"] += "\nSifr is currently in preview.\n"
    try:
        validate_documents(
            documents,
            public_documents=changed_public,
            facts=facts,
            docs_config=docs_config,
            internal_distribution=internal_distribution,
        )
    except DocumentationError:
        pass
    else:
        raise DocumentationError(
            "GA documentation mutation unexpectedly passed: global-preview-claim"
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        (
            documents,
            public_documents,
            facts,
            docs_config,
            internal_distribution,
        ) = canonical_inputs()
        validate_documents(
            documents,
            public_documents=public_documents,
            facts=facts,
            docs_config=docs_config,
            internal_distribution=internal_distribution,
        )
        run_self_test()
        if not args.self_test:
            subprocess.run(
                [sys.executable, str(RUST_CLAIMS_CHECK)],
                cwd=REPO_ROOT,
                check=True,
            )
    except (
        DocumentationError,
        OSError,
        UnicodeError,
        subprocess.CalledProcessError,
    ) as exc:
        print(f"documentation-ga-release: {exc}", file=sys.stderr)
        return 2
    if args.self_test:
        print("GA documentation mutation harness ok")
    else:
        print("GA documentation and stable Rust claims ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
