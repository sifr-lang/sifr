"""Validate stable Rust-interop claims against compatibility evidence."""

from __future__ import annotations

import copy
import io
import json
import re
import sys
import tempfile
from contextlib import redirect_stderr
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
SOURCE_PATH = "verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json"
PUBLIC_PATH = "docs/rust-interop.mdx"
START_MARKER = "<!-- rust-interop-stable-claims:start -->"
END_MARKER = "<!-- rust-interop-stable-claims:end -->"
CLAIM_FIELDS = {"id", "category", "execution_kind", "capability"}
RUNTIME_CLAIM_TERMS = (
    "runtime evidence",
    "runtime support",
    "runtime-observed",
)
DOCUMENT_FIELDS = {
    "schema_version",
    "role",
    "source_compatibility_matrix",
    "public_document",
    "runtime_deferrals",
    "claims",
}
ADVERTISEMENT_TERMS = (" supported", "runtime support", "stable support", "certified")
DEFERRAL_TERMS = (
    "contract-only",
    "does not",
    "future-owned",
    "not supported",
    "pending",
    "planned",
    "unadvertised",
)


def _collect_public_documents(repo_root: Path) -> dict[str, str]:
    docs_root = repo_root / "docs"
    return {
        path.relative_to(repo_root).as_posix(): path.read_text(encoding="utf-8")
        for path in docs_root.rglob("*")
        if path.is_file() and path.suffix in {".md", ".mdx"}
    }


def _parse_public_claims(text: str, failures: list[str]) -> list[dict[str, str]]:
    if text.count(START_MARKER) != 1 or text.count(END_MARKER) != 1:
        failures.append("public docs must contain exactly one stable-claims marker pair")
        return []
    if text.index(START_MARKER) >= text.index(END_MARKER):
        failures.append("public stable-claims markers must be ordered")
        return []
    block = text.split(START_MARKER, maxsplit=1)[1].split(END_MARKER, maxsplit=1)[0]
    block_lines = [line.strip() for line in block.splitlines() if line.strip()]
    if any(not line.startswith("|") for line in block_lines):
        failures.append("public stable-claims block must contain only table rows")
        return []
    rows = block_lines
    expected_headers = [
        "| Compatibility row | Category | Execution scope |",
        "| --- | --- | --- |",
    ]
    if rows[:2] != expected_headers:
        failures.append("public stable-claims table must use the canonical three-column header")
        return []

    claims: list[dict[str, str]] = []
    for line in rows[2:]:
        cells = [cell.strip().strip("`") for cell in line.strip("|").split("|")]
        if len(cells) != 3 or not all(cells):
            failures.append(f"malformed public stable-claims row: {line}")
            continue
        claims.append(
            {
                "id": cells[0],
                "category": cells[1],
                "execution_kind": cells[2],
            }
        )
    return claims


def _validate(
    compatibility: dict[str, Any],
    claims_document: dict[str, Any],
    public_text: str,
    public_documents: dict[str, str] | None = None,
) -> list[str]:
    failures: list[str] = []
    if set(claims_document) != DOCUMENT_FIELDS:
        failures.append("stable claims document fields must be exact")
    if claims_document.get("schema_version") != 1:
        failures.append("stable claims schema_version must be 1")
    if claims_document.get("role") != "compatibility-derived-release-plan-input":
        failures.append("stable claims role must be compatibility-derived release-plan input")
    if claims_document.get("source_compatibility_matrix") != SOURCE_PATH:
        failures.append("stable claims source_compatibility_matrix is not canonical")
    if claims_document.get("public_document") != PUBLIC_PATH:
        failures.append("stable claims public_document is not canonical")

    rows = compatibility.get("rows")
    if not isinstance(rows, list):
        failures.append("compatibility rows must be a list")
        rows = []
    compatibility_by_id = {
        str(row.get("id")): row for row in rows if isinstance(row, dict) and row.get("id")
    }
    runtime_deferral_ids = {
        row_id
        for row_id, row in compatibility_by_id.items()
        if row.get("category") == "tracked-unsupported"
        and row.get("execution_kind") == "runtime-observed"
    }
    compile_scope_ids = {
        row_id
        for row_id, row in compatibility_by_id.items()
        if row.get("execution_kind") == "contract-only"
    }
    raw_runtime_deferrals = claims_document.get("runtime_deferrals")
    if not isinstance(raw_runtime_deferrals, list) or not all(
        isinstance(row_id, str) and row_id for row_id in raw_runtime_deferrals
    ):
        failures.append("stable claims runtime_deferrals must be a list of row ids")
    elif (
        len(raw_runtime_deferrals) != len(set(raw_runtime_deferrals))
        or set(raw_runtime_deferrals) != runtime_deferral_ids
    ):
        failures.append(
            "stable claims runtime_deferrals must exactly match future-owned "
            "runtime-observed compatibility rows"
        )

    raw_claims = claims_document.get("claims")
    if not isinstance(raw_claims, list):
        failures.append("stable claims must be a list")
        raw_claims = []
    claims: list[dict[str, Any]] = []
    seen: set[str] = set()
    for claim in raw_claims:
        if not isinstance(claim, dict):
            failures.append("stable claim entries must be objects")
            continue
        claim_id = str(claim.get("id", ""))
        if set(claim) != CLAIM_FIELDS:
            failures.append(f"{claim_id or '<missing>'}: stable claim fields must be exact")
        if not claim_id:
            failures.append("stable claim id is required")
            continue
        if claim_id in seen:
            failures.append(f"{claim_id}: stable claim ids must be unique")
        seen.add(claim_id)
        claims.append(claim)

        row = compatibility_by_id.get(claim_id)
        if row is None:
            failures.append(f"{claim_id}: stable claim has no compatibility row")
            continue
        if row.get("category") == "tracked-unsupported":
            failures.append(f"{claim_id}: future-owned rows cannot be stable claims")
        for field in ("category", "execution_kind", "capability"):
            if claim.get(field) != row.get(field):
                failures.append(f"{claim_id}: stable claim {field} must match compatibility row")

    public_claims = _parse_public_claims(public_text, failures)
    expected_public = [
        {
            "id": str(claim.get("id", "")),
            "category": str(claim.get("category", "")),
            "execution_kind": str(claim.get("execution_kind", "")),
        }
        for claim in claims
    ]
    public_ids = [claim["id"] for claim in public_claims]
    if len(public_ids) != len(set(public_ids)):
        failures.append("public stable claim ids must be unique")
    missing_claims = set(public_ids) - seen
    for claim_id in sorted(missing_claims):
        failures.append(f"{claim_id}: public stable claim is absent from stable_support_claims.json")
    if public_claims != expected_public:
        failures.append("public stable-claims table must exactly match stable_support_claims.json")

    for public_claim in public_claims:
        row = compatibility_by_id.get(public_claim["id"])
        if (
            row is not None
            and row.get("execution_kind") == "contract-only"
            and public_claim["execution_kind"] == "runtime-observed"
        ):
            failures.append(
                f"{public_claim['id']}: public docs cannot advertise runtime support "
                "through a contract-only row"
            )
    documents = {PUBLIC_PATH: public_text}
    if public_documents is not None:
        documents.update(public_documents)
    for path, text in sorted(documents.items()):
        if path != PUBLIC_PATH and (
            START_MARKER in text or END_MARKER in text
        ):
            failures.append(
                f"{path}: stable-claims markers may appear only in {PUBLIC_PATH}"
            )
        _validate_unstructured_advertisements(
            failures,
            text,
            compatibility_by_id,
            path,
        )
    _validate_public_document_scope(
        failures,
        documents,
        runtime_deferral_ids,
        compile_scope_ids,
        seen,
    )
    return failures


def _outside_claim_table(text: str) -> str:
    if START_MARKER not in text or END_MARKER not in text:
        return text
    if text.index(START_MARKER) >= text.index(END_MARKER):
        return text
    prefix, remainder = text.split(START_MARKER, maxsplit=1)
    _block, suffix = remainder.split(END_MARKER, maxsplit=1)
    return f"{prefix}\n\n{suffix}"


def _prose_units(text: str) -> list[str]:
    """Return normalized sentence/semicolon units outside the canonical table."""
    segments: list[str] = []
    for block in _outside_claim_table(text).split("\n\n"):
        current_lines: list[str] = []
        for line in block.splitlines():
            is_heading = re.match(r"^\s*#{1,6}\s+", line)
            is_markdown_item = re.match(
                r"^\s*(?:[-*+]\s+|\d+[.)]\s+|\||>|#{1,6}\s+|<li\b)",
                line,
            )
            if is_markdown_item and current_lines:
                segments.append(" ".join(current_lines))
                current_lines = []
            if is_heading:
                segments.append(line.strip())
                continue
            if line.strip():
                current_lines.append(line.strip())
        if current_lines:
            segments.append(" ".join(current_lines))

    normalized_segments = [
        " ".join(segment.split()).lower()
        for segment in segments
        if segment.strip()
    ]
    return [
        unit.strip()
        for segment in normalized_segments
        for unit in re.split(
            r"(?<=[.;])\s+|(?:,\s+|\s+)(?=(?:but|except|however|although|though|"
            r"whereas|besides|yet|while|whilst|instead of|rather than|unlike|"
            r"notwithstanding|save for|excluding|barring|aside from|apart from|"
            r"other than|with the exception of)\b)",
            segment,
        )
        if unit.strip()
    ]


def _validate_public_document_scope(
    failures: list[str],
    documents: dict[str, str],
    runtime_deferral_ids: set[str],
    compile_scope_ids: set[str],
    stable_claim_ids: set[str],
) -> None:
    canonical_text = documents.get(PUBLIC_PATH, "")
    for row_id in sorted(runtime_deferral_ids):
        if f"`{row_id}`" not in canonical_text:
            failures.append(
                f"{row_id}: canonical public docs must name the planned runtime deferral"
            )
    for path, text in sorted(documents.items()):
        units = _prose_units(text)
        for row_id in sorted(runtime_deferral_ids):
            row_token = f"`{row_id}`"
            for unit in units:
                if row_token not in unit:
                    continue
                if "future-owned" not in unit or not any(
                    term in unit for term in ("planned", "unadvertised")
                ):
                    failures.append(
                        f"{path}: {row_id} must be described as future-owned and planned"
                    )
        for row_id in sorted(compile_scope_ids):
            row_token = f"`{row_id}`"
            for unit in units:
                if row_token not in unit or not any(
                    term in unit for term in RUNTIME_CLAIM_TERMS
                ):
                    continue
                has_negation = any(
                    term in unit
                    for term in (
                        " cannot ",
                        " does not ",
                        " never ",
                        " not ",
                        "unadvertised",
                    )
                )
                if "contract-only" not in unit or not has_negation:
                    failures.append(
                        f"{path}: {row_id} cannot be presented as runtime evidence"
                    )
        for row_id in sorted(stable_claim_ids):
            row_token = f"`{row_id}`"
            for unit in units:
                if row_token not in unit:
                    continue
                if any(
                    term in unit
                    for term in ("future-owned", "planned", "pending", "unadvertised")
                ):
                    failures.append(
                        f"{path}: {row_id} is a stable claim and cannot remain "
                        "described as a runtime deferral"
                    )


def _validate_unstructured_advertisements(
    failures: list[str],
    public_text: str,
    compatibility_by_id: dict[str, dict[str, Any]],
    path: str,
) -> None:
    """Reject row-specific support prose outside the canonical claim table."""
    for unit_number, unit in enumerate(_prose_units(public_text), start=1):
        if not any(term in unit for term in ADVERTISEMENT_TERMS):
            continue
        if any(term in unit for term in DEFERRAL_TERMS):
            continue
        for row_id in compatibility_by_id:
            if f"`{row_id}`" in unit:
                failures.append(
                    f"{path}:prose-unit-{unit_number}: {row_id} public stable support "
                    "advertisement must be in the canonical table"
                )


def _public_table(claims: list[dict[str, Any]]) -> str:
    lines = [
        START_MARKER,
        "| Compatibility row | Category | Execution scope |",
        "| --- | --- | --- |",
    ]
    lines.extend(
        f"| `{claim['id']}` | `{claim['category']}` | `{claim['execution_kind']}` |"
        for claim in claims
    )
    lines.append(END_MARKER)
    return "\n".join(lines)


def _run_self_test() -> int:
    compatibility = {
        "rows": [
            {
                "id": "zero_copy_bytes",
                "category": "supported",
                "execution_kind": "contract-only",
                "capability": "compile-time contract",
            },
            {
                "id": "runtime",
                "category": "supported-through-bridge",
                "execution_kind": "runtime-observed",
                "capability": "runtime behavior",
            },
            {
                "id": "future",
                "category": "tracked-unsupported",
                "execution_kind": "runtime-observed",
                "capability": "future behavior",
            },
        ]
    }
    claims = {
        "schema_version": 1,
        "role": "compatibility-derived-release-plan-input",
        "source_compatibility_matrix": SOURCE_PATH,
        "public_document": PUBLIC_PATH,
        "runtime_deferrals": ["future"],
        "claims": [
            {
                "id": "zero_copy_bytes",
                "category": "supported",
                "execution_kind": "contract-only",
                "capability": "compile-time contract",
            },
            {
                "id": "runtime",
                "category": "supported-through-bridge",
                "execution_kind": "runtime-observed",
                "capability": "runtime behavior",
            },
        ],
    }
    def scoped_text(claim_rows: list[dict[str, Any]]) -> str:
        return (
            _public_table(claim_rows)
            + "\n\n"
            + "`future` is a future-owned planned runtime deferral "
            + "and remains unadvertised."
        )

    control_text = scoped_text(claims["claims"])
    control_failures = _validate(compatibility, claims, control_text)
    if control_failures:
        print(
            f"stable support claims self-test error: valid control failed: {control_failures}",
            file=sys.stderr,
        )
        return 1

    cases: tuple[tuple[str, dict[str, Any], str, str], ...] = (
        (
            "missing compatibility row",
            {**copy.deepcopy(claims), "claims": [{**claims["claims"][0], "id": "missing"}]},
            scoped_text([{**claims["claims"][0], "id": "missing"}]),
            "has no compatibility row",
        ),
        (
            "future-owned claim",
            {
                **copy.deepcopy(claims),
                "claims": [
                    {
                        "id": "future",
                        "category": "tracked-unsupported",
                        "execution_kind": "runtime-observed",
                        "capability": "future behavior",
                    }
                ],
            },
            scoped_text(
                [
                    {
                        "id": "future",
                        "category": "tracked-unsupported",
                        "execution_kind": "runtime-observed",
                    }
                ]
            ),
            "future-owned rows cannot be stable claims",
        ),
        (
            "claim scope drift",
            {
                **copy.deepcopy(claims),
                "claims": [{**claims["claims"][0], "execution_kind": "runtime-observed"}],
            },
            scoped_text([{**claims["claims"][0], "execution_kind": "runtime-observed"}]),
            "execution_kind must match compatibility row",
        ),
        (
            "public claim absent from data",
            claims,
            scoped_text([*claims["claims"], {"id": "extra", "category": "supported", "execution_kind": "cargo-probe"}]),
            "public stable claim is absent",
        ),
        (
            "public claim omission",
            claims,
            _public_table(claims["claims"][:-1]),
            "must exactly match stable_support_claims.json",
        ),
        (
            "public runtime overclaim",
            claims,
            scoped_text([{**claims["claims"][0], "execution_kind": "runtime-observed"}, claims["claims"][1]]),
            "cannot advertise runtime support through a contract-only row",
        ),
        (
            "prose inside canonical stable-claims block",
            claims,
            control_text.replace(
                END_MARKER,
                "`zero_copy_bytes` now provides certified runtime support and "
                "`future` is fully supported.\n"
                + END_MARKER,
            ),
            "stable-claims block must contain only table rows",
        ),
        (
            "reversed canonical stable-claims markers",
            claims,
            control_text.replace(START_MARKER, "<start-placeholder>")
            .replace(END_MARKER, START_MARKER)
            .replace("<start-placeholder>", END_MARKER),
            "stable-claims markers must be ordered",
        ),
        (
            "duplicated canonical stable-claims start marker",
            claims,
            control_text.replace(START_MARKER, f"{START_MARKER}\n{START_MARKER}"),
            "exactly one stable-claims marker pair",
        ),
        (
            "qualifier borrowing across canonical claim table",
            claims,
            "These rows are contract-only and do not certify runtime behavior "
            + _public_table(claims["claims"])
            + "`zero_copy_bytes` now provides runtime support and is certified.",
            "cannot be presented as runtime evidence",
        ),
        (
            "public doc omission",
            claims,
            scoped_text([claims["claims"][0]]),
            "must exactly match stable_support_claims.json",
        ),
        (
            "capability drift",
            {
                **copy.deepcopy(claims),
                "claims": [
                    {**claims["claims"][0], "capability": "drifted capability"},
                    claims["claims"][1],
                ],
            },
            scoped_text(claims["claims"]),
            "capability must match compatibility row",
        ),
        (
            "duplicate claim",
            {
                **copy.deepcopy(claims),
                "claims": [claims["claims"][0], claims["claims"][0]],
            },
            scoped_text([claims["claims"][0], claims["claims"][0]]),
            "stable claim ids must be unique",
        ),
        (
            "schema drift",
            {**copy.deepcopy(claims), "schema_version": 2},
            control_text,
            "schema_version must be 1",
        ),
        (
            "source path drift",
            {
                **copy.deepcopy(claims),
                "source_compatibility_matrix": "wrong.json",
            },
            control_text,
            "source_compatibility_matrix is not canonical",
        ),
        (
            "future deferral promotion",
            claims,
            _public_table(claims["claims"])
            + "\n\n`future` has runtime support.",
            "must be described as future-owned and planned",
        ),
        (
            "contract runtime prose overclaim",
            claims,
            scoped_text(claims["claims"])
            + "\n\n`zero_copy_bytes` provides runtime evidence.",
            "cannot be presented as runtime evidence",
        ),
        (
            "contract qualifier borrowing",
            claims,
            scoped_text(claims["claims"])
            + "\n\nThe remaining rows are contract-only and unadvertised, "
            "except `zero_copy_bytes`, which now provides runtime support.",
            "cannot be presented as runtime evidence",
        ),
        (
            "contract markdown-item borrowing",
            claims,
            scoped_text(claims["claims"])
            + "\n\n- these rows are contract-only and do not certify runtime behavior\n"
            "- `zero_copy_bytes` now provides runtime support",
            "cannot be presented as runtime evidence",
        ),
        (
            "contract blockquote borrowing",
            claims,
            scoped_text(claims["claims"])
            + "\n\n> these rows are contract-only and do not certify runtime behavior\n"
            "> `zero_copy_bytes` now provides runtime support",
            "cannot be presented as runtime evidence",
        ),
        (
            "contract heading borrowing",
            claims,
            scoped_text(claims["claims"])
            + "\n\n## contract-only rows do not certify runtime behavior\n"
            "`zero_copy_bytes` now provides runtime support",
            "cannot be presented as runtime evidence",
        ),
        (
            "contract comma-free exception borrowing",
            claims,
            scoped_text(claims["claims"])
            + "\n\nThese rows are contract-only and do not certify runtime behavior "
            "except `zero_copy_bytes` which now provides runtime support.",
            "cannot be presented as runtime evidence",
        ),
        (
            "contract although borrowing",
            claims,
            scoped_text(claims["claims"])
            + "\n\nThese rows are contract-only and cannot certify execution, although "
            "`zero_copy_bytes` now provides runtime support.",
            "cannot be presented as runtime evidence",
        ),
        (
            "contract yet borrowing",
            claims,
            scoped_text(claims["claims"])
            + "\n\nThese rows are contract-only and do not certify runtime behavior, yet "
            "`zero_copy_bytes` now provides runtime support.",
            "cannot be presented as runtime evidence",
        ),
        (
            "deferral markdown-item borrowing",
            claims,
            scoped_text(claims["claims"])
            + "\n\n- the rows below are future-owned and planned\n"
            "- `future` now provides runtime support",
            "must be described as future-owned and planned",
        ),
        (
            "deferral comma-free exception borrowing",
            claims,
            scoped_text(claims["claims"])
            + "\n\nAll these rows are future-owned and planned except "
            "`future` which now provides runtime support.",
            "must be described as future-owned and planned",
        ),
        (
            "authority drift",
            {**copy.deepcopy(claims), "role": "release-plan-authority"},
            control_text,
            "role must be compatibility-derived release-plan input",
        ),
        (
            "runtime deferral inventory drift",
            {**copy.deepcopy(claims), "runtime_deferrals": []},
            control_text,
            "runtime_deferrals must exactly match",
        ),
        (
            "unstructured future advertisement",
            claims,
            f"{control_text}\n`future` has stable runtime support.",
            "advertisement must be in the canonical table",
        ),
    )
    for name, case_claims, public_text, expected in cases:
        failures = _validate(compatibility, case_claims, public_text)
        if not any(expected in failure for failure in failures):
            print(
                f"stable support claims self-test error: {name} did not report {expected!r}",
                file=sys.stderr,
            )
            return 1

    promoted_compatibility = copy.deepcopy(compatibility)
    promoted_row = promoted_compatibility["rows"][2]
    promoted_row["category"] = "supported"
    promoted_claim = {
        "id": "future",
        "category": "supported",
        "execution_kind": "runtime-observed",
        "capability": "future behavior",
    }
    promoted_claims = {
        **copy.deepcopy(claims),
        "runtime_deferrals": [],
        "claims": [*claims["claims"], promoted_claim],
    }
    stale_promotion_text = (
        _public_table(promoted_claims["claims"])
        + "\n\n`future` is future-owned, planned, and unadvertised."
    )
    promotion_failures = _validate(
        promoted_compatibility,
        promoted_claims,
        stale_promotion_text,
    )
    if not any(
        "future is a stable claim and cannot remain described as a runtime deferral"
        in failure
        for failure in promotion_failures
    ):
        print(
            "stable support claims self-test error: stale promotion prose passed",
            file=sys.stderr,
        )
        return 1

    with tempfile.TemporaryDirectory(prefix="sifr-stable-claims-docs-") as raw_root:
        temp_root = Path(raw_root)
        docs_root = Path(raw_root) / "docs"
        docs_root.mkdir()
        data_root = (
            temp_root / "verification" / "areas" / "rust_interop" / "data"
        )
        data_root.mkdir(parents=True)
        (data_root / "rust_interop_compatibility_matrix.json").write_text(
            json.dumps(compatibility),
            encoding="utf-8",
        )
        (data_root / "stable_support_claims.json").write_text(
            json.dumps(claims),
            encoding="utf-8",
        )
        (docs_root / "rust-interop.mdx").write_text(
            control_text,
            encoding="utf-8",
        )
        (docs_root / "secondary.md").write_text(
            "`zero_copy_bytes` provides certified runtime support.",
            encoding="utf-8",
        )
        (docs_root / "release-notes.md").write_text(
            START_MARKER
            + "\n| Compatibility row | Category | Execution scope |"
            + "\n| --- | --- | --- |"
            + "\n| `zero_copy_bytes` | `supported` | `runtime-observed` |"
            + "\n`future` has certified runtime support."
            + "\n"
            + END_MARKER,
            encoding="utf-8",
        )
        discovered_documents = _collect_public_documents(temp_root)
        main_stderr = io.StringIO()
        with redirect_stderr(main_stderr):
            main_result = main([], repo_root=temp_root)
        if main_result != 1 or not all(
            term in main_stderr.getvalue()
            for term in (
                "docs/secondary.md",
                "docs/release-notes.md: stable-claims markers may appear only",
            )
        ):
            print(
                "stable support claims self-test error: main() did not enforce "
                "the docs-wide sweep",
                file=sys.stderr,
            )
            return 1
    docs_wide_failures = _validate(
        compatibility,
        claims,
        control_text,
        discovered_documents,
    )
    if not any(
        "docs/secondary.md" in failure
        and "cannot be presented as runtime evidence" in failure
        for failure in docs_wide_failures
    ):
        print(
            "stable support claims self-test error: docs-wide sweep did not "
            "reject a secondary-document runtime overclaim",
            file=sys.stderr,
        )
        return 1
    if not any(
        "docs/release-notes.md: stable-claims markers may appear only" in failure
        for failure in docs_wide_failures
    ):
        print(
            "stable support claims self-test error: secondary stable-claims "
            "marker copy passed",
            file=sys.stderr,
        )
        return 1
    print(f"stable support claims self-test ok: cases={len(cases) + 4}")
    return 0


def main(argv: list[str] | None = None, repo_root: Path = REPO_ROOT) -> int:
    args = sys.argv[1:] if argv is None else argv
    if args == ["--self-test"]:
        return _run_self_test()
    if args:
        print(f"usage: {Path(__file__).name} [--self-test]", file=sys.stderr)
        return 2

    compatibility_path = repo_root / SOURCE_PATH
    claims_path = (
        repo_root
        / "verification"
        / "areas"
        / "rust_interop"
        / "data"
        / "stable_support_claims.json"
    )
    public_doc_path = repo_root / PUBLIC_PATH
    compatibility = json.loads(compatibility_path.read_text(encoding="utf-8"))
    claims = json.loads(claims_path.read_text(encoding="utf-8"))
    public_text = public_doc_path.read_text(encoding="utf-8")
    public_documents = _collect_public_documents(repo_root)
    failures = _validate(
        compatibility,
        claims,
        public_text,
        public_documents,
    )
    if failures:
        for failure in failures:
            print(f"stable support claims error: {failure}", file=sys.stderr)
        return 1
    print(f"stable support claims ok: claims={len(claims['claims'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
