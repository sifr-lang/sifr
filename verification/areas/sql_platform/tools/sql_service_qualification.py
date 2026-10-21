"""Follow the SQL service owner and preserve the driver's public delegation."""

from __future__ import annotations

from collections.abc import Callable
from pathlib import Path
from unittest.mock import patch

SERVICE_PATH = "crates/sifr_semantic_services/src/sql_editor.rs"
NATIVE_PATH = "crates/sifr_compiler_services/src/lib.rs"
NATIVE_MARKERS = ("pub use sifr_semantic_services::sql_editor;",)
DRIVER_PATH = "crates/sifr_driver/src/build/sql_profiles.rs"
DRIVER_MARKERS = ("pub use sifr_compiler_services::sql_editor::{", "prepare_sql_profiles")


def service_source(root: Path) -> str:
    driver = (root / DRIVER_PATH).read_text(encoding="utf-8")
    if any(marker not in driver for marker in DRIVER_MARKERS):
        raise ValueError("driver does not delegate SQL profile preparation to compiler_services")
    native = (root / NATIVE_PATH).read_text(encoding="utf-8")
    if any(marker not in native for marker in NATIVE_MARKERS):
        raise ValueError("native services do not delegate SQL profiles to semantic_services")
    return (root / SERVICE_PATH).read_text(encoding="utf-8")


def assert_source_mutations(validate: Callable[[], None], root: Path, markers: tuple[str, ...]) -> int:
    """Require a passing baseline, then reject each removed authority independently."""
    validate()
    cases = [(root / DRIVER_PATH, marker) for marker in DRIVER_MARKERS]
    cases.extend((root / NATIVE_PATH, marker) for marker in NATIVE_MARKERS)
    cases.extend((root / SERVICE_PATH, marker) for marker in markers)
    original_read = Path.read_text
    for target, marker in cases:
        if marker not in original_read(target, encoding="utf-8"):
            raise ValueError(f"source mutation did not select a real authority: {marker}")

        def read(path: Path, *args: object, **kwargs: object) -> str:
            source = original_read(path, *args, **kwargs)
            return source.replace(marker, "__removed_sql_authority__") if path == target else source

        with patch.object(Path, "read_text", read):
            try:
                validate()
            except ValueError:
                continue
        raise ValueError(f"SQL source authority mutation was accepted: {target}: {marker}")
    return len(cases)
