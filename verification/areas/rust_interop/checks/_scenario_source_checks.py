"""Source-level policy for Rust interop scenario packages."""

from __future__ import annotations

from pathlib import Path

from _binding_helpers import contains_empty_pass_body
from _binding_helpers import rust_bound_declarations
from _binding_helpers import verifier_binds_call


def read_scenario_text(
    readme_path: Path,
    sifr_config_path: Path,
    sifr_sources: list[Path],
    cargo_manifests: list[Path],
    rust_sources: list[Path],
) -> str:
    paths = [
        readme_path,
        sifr_config_path,
        *sifr_sources,
        *cargo_manifests,
        *rust_sources,
    ]
    return "\n".join(
        path.read_text(encoding="utf-8") for path in paths if path.is_file()
    )


def validate_scenario_sifr_source(
    failures: list[str],
    fixture_id: str,
    example: str,
    raw_path: str,
    text: str,
) -> None:
    if len(text.strip().splitlines()) < 10:
        failures.append(
            f"{fixture_id}: {raw_path} must contain a full scenario source"
        )
    for header in ("# execution-kind:", "# expected-result:"):
        if header not in text:
            failures.append(f"{fixture_id}: {raw_path} missing {header} header")
    if contains_empty_pass_body(text):
        failures.append(
            f"{fixture_id}: {raw_path} must not use empty placeholder class bodies"
        )
    if not any(line.lstrip().startswith("@rust") for line in text.splitlines()):
        failures.append(
            f"{fixture_id}: {raw_path} must exercise a Rust interop declaration"
        )
    bound_declarations = rust_bound_declarations(text)
    if not bound_declarations:
        failures.append(
            f"{fixture_id}: {raw_path} must include Rust-decorated binding declarations"
        )

    verifier_markers = (
        f"def verify_{example}(",
        f"async def verify_{example}(",
    )
    verifier_start = min(
        (text.find(marker) for marker in verifier_markers if marker in text),
        default=-1,
    )
    if verifier_start < 0:
        failures.append(f"{fixture_id}: {raw_path} must include verify_{example}")
        return

    verifier_body = text[verifier_start:]
    for name, return_type in bound_declarations:
        if f"{name}(" not in verifier_body and f".{name}(" not in verifier_body:
            failures.append(
                f"{fixture_id}: {raw_path} verifier must call {name}"
            )
        if return_type != "None" and not verifier_binds_call(verifier_body, name):
            failures.append(
                f"{fixture_id}: {raw_path} verifier must bind {name} result "
                "before returning"
            )


def validate_auxiliary_sifr_source(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    text: str,
) -> None:
    """Reject placeholders without imposing entrypoint-only evidence rules."""
    if contains_empty_pass_body(text):
        failures.append(
            f"{fixture_id}: {raw_path} must not use empty placeholder class bodies"
        )


def reject_generated_bridge_imports(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    rust_sources: list[Path],
) -> None:
    for source in rust_sources:
        lines = source.read_text(encoding="utf-8").splitlines()
        for line_number, line in enumerate(lines, start=1):
            stripped = line.strip()
            if (
                "crate::__sifr_bridge" in stripped
                and not stripped.startswith("//")
            ):
                relative = source.as_posix().split(
                    f"{raw_path}/", maxsplit=1
                )[-1]
                failures.append(
                    f"{fixture_id}: {relative}:{line_number} must not reference "
                    "crate::__sifr_bridge"
                )
