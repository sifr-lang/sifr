"""Regression coverage for runtime-computed dependency audit project paths."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from unittest.mock import patch

import dependency_versions as audit_runner


class DependencyProjectPathsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.audit = json.loads(audit_runner.AUDIT_PATH.read_text(encoding="utf-8"))

    def test_computed_project_paths_resolve_to_maintained_files(self) -> None:
        projects = audit_runner.project_map(self.audit)
        self.assertEqual(
            projects["dlpack-demo"][:2],
            ("demos/python_dlpack/pyproject.toml", "demos/python_dlpack/uv.lock"),
        )
        for name, (pyproject, lock, _) in projects.items():
            for relative_path in (pyproject, lock):
                with self.subTest(project=name, path=relative_path):
                    self.assertTrue((audit_runner.REPO_ROOT / relative_path).is_file())

    def test_repository_validation_reads_authoritative_demo_inputs(self) -> None:
        with patch.object(
            audit_runner, "load_toml", wraps=audit_runner.load_toml
        ) as loader:
            self.assertEqual(audit_runner.validate_repository(self.audit), [])
        demo_root = audit_runner.REPO_ROOT / "demos/python_dlpack"
        for filename in ("pyproject.toml", "uv.lock"):
            self.assertEqual(
                loader.call_args_list.count(unittest.mock.call(demo_root / filename)),
                1,
            )

    def test_missing_demo_input_fails_without_fallback(self) -> None:
        load_toml = audit_runner.load_toml
        for filename in ("pyproject.toml", "uv.lock"):
            missing = audit_runner.REPO_ROOT / "demos/python_dlpack" / filename

            def load_with_missing(path: Path) -> dict[str, object]:
                if path == missing:
                    raise FileNotFoundError(str(path))
                return load_toml(path)

            with self.subTest(filename=filename), patch.object(
                audit_runner, "load_toml", side_effect=load_with_missing
            ):
                with self.assertRaises(FileNotFoundError) as failure:
                    audit_runner.validate_repository(self.audit)
                self.assertEqual(str(failure.exception), str(missing))

    def test_obsolete_computed_demo_reference_is_rejected(self) -> None:
        obsolete_root = Path("demos") / ("m" + "12_dlpack_demo")
        obsolete_paths = tuple(
            str(obsolete_root / filename) for filename in ("pyproject.toml", "uv.lock")
        )
        with patch.dict(audit_runner.PROJECT_PATHS, {"dlpack-demo": obsolete_paths}):
            with self.assertRaises(FileNotFoundError) as failure:
                audit_runner.validate_repository(self.audit)
        self.assertEqual(
            failure.exception.filename,
            str(audit_runner.REPO_ROOT / obsolete_paths[0]),
        )


if __name__ == "__main__":
    unittest.main()
