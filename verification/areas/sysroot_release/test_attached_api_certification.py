from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

AREA_ROOT = Path(__file__).resolve().parent
if str(AREA_ROOT) not in sys.path:
    sys.path.insert(0, str(AREA_ROOT))

from attached_api_certification import bind_runtime_dependency  # noqa: E402


class BindRuntimeDependencyTests(unittest.TestCase):
    def test_binds_exact_runtime_crate_path(self) -> None:
        with tempfile.TemporaryDirectory() as raw_temp:
            fixture = Path(raw_temp)
            manifest = fixture / "Cargo.toml"
            manifest.write_text(
                '[dependencies]\n'
                'sifr_runtime = { path = "../../crates/sifr_runtime", '
                'features = ["structural"] }\n',
                encoding="utf-8",
            )

            error = bind_runtime_dependency(
                fixture=fixture,
                runtime_crate=Path('/tmp/runtime "exact"'),
            )

            self.assertIsNone(error)
            self.assertIn(
                'path = "/tmp/runtime \\"exact\\""',
                manifest.read_text(encoding="utf-8"),
            )

    def test_rejects_missing_or_duplicate_runtime_dependency(self) -> None:
        with tempfile.TemporaryDirectory() as raw_temp:
            fixture = Path(raw_temp)
            manifest = fixture / "Cargo.toml"
            manifest.write_text("[dependencies]\n", encoding="utf-8")
            missing = bind_runtime_dependency(
                fixture=fixture, runtime_crate=Path("/tmp/runtime")
            )
            self.assertIn("exactly one", missing)

            manifest.write_text(
                '[dependencies]\n'
                'sifr_runtime = { path = "one", features = ["structural"] }\n'
                'sifr_runtime = { path = "two", features = ["structural"] }\n',
                encoding="utf-8",
            )
            duplicate = bind_runtime_dependency(
                fixture=fixture, runtime_crate=Path("/tmp/runtime")
            )
            self.assertIn("exactly one", duplicate)


if __name__ == "__main__":
    unittest.main()
