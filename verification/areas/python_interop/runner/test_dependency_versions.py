"""Regression coverage for runtime-computed dependency audit project paths."""

from __future__ import annotations

import json
import copy
import unittest
from pathlib import Path
from unittest.mock import patch

import dependency_versions as audit_runner
from dependency_requirements import requirement_records, validate_requirements


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
            # Source-routing coverage is independent of official freshness findings.
            audit_runner.validate_repository(self.audit)
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

    def test_obsolete_demo_reference_is_rejected(self) -> None:
        obsolete_root = Path("demos") / ("m" + "12_dlpack_demo")
        obsolete_paths = tuple(
            str(obsolete_root / filename) for filename in ("pyproject.toml", "uv.lock")
        )
        owner = next(p for p in self.audit["projects"] if p["name"] == "dlpack-demo")
        owner["pyproject"], owner["lock"] = obsolete_paths
        with self.assertRaisesRegex(ValueError, "actual maintained discovery"):
            audit_runner.validate_repository(self.audit)


class ServiceImageAuthorityTests(unittest.TestCase):
    def setUp(self):
        self.audit = json.loads(audit_runner.AUDIT_PATH.read_text())

    def test_all_live_owners_match_immutable_release_tags(self):
        self.assertEqual(audit_runner.validate_service_images(self.audit), [])
        self.assertEqual(
            {image["name"] for image in self.audit["service_images"]},
            {"kafka", "localstack", "postgres", "redis"},
        )

    def test_each_owner_omission_and_duplicate_are_rejected(self):
        for image in self.audit["service_images"]:
            for duplicate in (False, True):
                audit = copy.deepcopy(self.audit)
                if duplicate:
                    audit["service_images"].append(image)
                else:
                    audit["service_images"].remove(image)
                with self.subTest(owner=image["name"], duplicate=duplicate):
                    self.assertTrue(audit_runner.validate_service_images(audit))

    def test_new_live_owner_cannot_escape_inventory(self):
        live = audit_runner.load_literal_assignment(audit_runner.LIVE_CASE_CONFIG_PATH, "LIVE_IMAGES")
        live["new-service"] = "example:1.0@sha256:" + "0" * 64
        with patch.object(audit_runner, "load_literal_assignment", return_value=live):
            self.assertTrue(audit_runner.validate_service_images(self.audit))

    def test_release_identity_cannot_include_alpine_variant(self):
        redis = next(image for image in self.audit["service_images"] if image["name"] == "redis")
        redis["latest_stable"] = redis["image_tag"]
        self.assertIn("redis: stable release must exclude image tag variants",
                      audit_runner.validate_service_images(self.audit))

    def test_each_tag_and_digest_is_checked(self):
        for index, image in enumerate(self.audit["service_images"]):
            for field, value in (("image_tag", "latest"),
                                 ("image_tag", "0.0-alpine"),
                                 ("manifest_digest", "sha256:" + "g" * 64),
                                 ("manifest_digest", "sha256:" + "0" * 64)):
                audit = copy.deepcopy(self.audit)
                audit["service_images"][index][field] = value
                with self.subTest(owner=image["name"], field=field, value=value):
                    self.assertTrue(audit_runner.validate_service_images(audit))


class RequirementAuthorityTests(unittest.TestCase):
    def setUp(self):
        self.audit = json.loads(audit_runner.AUDIT_PATH.read_text())
        self.releases = audit_runner.release_map(self.audit)
        self.owner = next(p for p in self.audit["projects"] if p["name"] == "python-interop")
        self.project = audit_runner.load_toml(audit_runner.REPO_ROOT / self.owner["pyproject"])
        self.lock = audit_runner.load_toml(audit_runner.REPO_ROOT / self.owner["lock"])

    def errors(self):
        return validate_requirements("test", self.project, self.lock, self.releases,
                                     self.owner["requirements"])

    def test_all_current_requirement_metadata_is_consistent(self):
        self.assertEqual(self.errors(), [])

    def test_every_direct_requirement_omission_is_rejected(self):
        original = copy.deepcopy(self.project)
        for index in range(len(original["project"]["dependencies"])):
            self.project = copy.deepcopy(original)
            self.project["project"]["dependencies"].pop(index)
            with self.subTest(index=index):
                self.assertIn("test: direct requirements/extras/markers differ from audit", self.errors())

    def test_empty_project_omission_is_rejected_by_discovery(self):
        owner = next(p for p in self.audit["projects"] if not p["packages"])
        self.audit["projects"].remove(owner)
        with self.assertRaisesRegex(ValueError, "actual maintained discovery"):
            audit_runner.validate_repository(self.audit)

    def test_new_owner_and_lock_are_discovered(self):
        discovered = audit_runner.discover_projects(audit_runner.REPO_ROOT)
        discovered["new_project/pyproject.toml"] = "new_project/uv.lock"
        with patch.object(audit_runner, "discover_projects", return_value=discovered):
            with self.assertRaisesRegex(ValueError, "actual maintained discovery"):
                audit_runner.validate_repository(self.audit)

    def test_project_and_lock_python_lane(self):
        for source in (self.project["project"], self.lock):
            original = source["requires-python"]
            source["requires-python"] = ">=3.13"
            self.assertTrue(any("must select only Python" in e for e in self.errors()))
            source["requires-python"] = original

    def test_locked_package_requires_python(self):
        self.releases["aiosqlite"]["requires_python"] = "<3.14"
        self.assertIn("test: locked aiosqlite Requires-Python excludes 3.14.7", self.errors())

    def test_unconstrained_optional_python_metadata(self):
        self.releases["aiosqlite"]["requires_python"] = None
        self.assertEqual(self.errors(), [])

    def test_marker_cannot_hide_requirement_from_maintained_lane(self):
        self.project["project"]["dependencies"][0] = "aiosqlite>=0.20,<1; python_version < '3.14'"
        self.owner["requirements"] = requirement_records(self.project)
        self.assertIn("test: aiosqlite marker excludes the maintained Python lane", self.errors())

    def test_extras_are_checked_against_locked_release(self):
        self.project["project"]["dependencies"][0] = "aiosqlite[missing]>=0.20,<1"
        self.owner["requirements"] = requirement_records(self.project)
        self.assertIn("test: aiosqlite requests unavailable extras ['missing']", self.errors())

    def test_requirement_must_admit_actual_locked_version(self):
        self.project["project"]["dependencies"][0] = "aiosqlite<0.1"
        self.owner["requirements"] = requirement_records(self.project)
        self.assertTrue(any("requirement does not admit locked" in e for e in self.errors()))

    def test_lock_cannot_drop_requirement_extras(self):
        root = next(p for p in self.lock["package"] if "metadata" in p and p["name"] == self.project["project"]["name"])
        req = next(r for r in root["metadata"]["requires-dist"] if r["name"] == "psycopg")
        req["extras"] = []
        self.assertIn("test: uv.lock direct requirement metadata differs from manifest", self.errors())

    def test_lock_cannot_orphan_a_direct_requirement(self):
        root = next(p for p in self.lock["package"] if p["name"] == self.project["project"]["name"])
        root["dependencies"] = [p for p in root["dependencies"] if p["name"] != "psycopg"]
        self.assertIn("test: uv.lock resolved direct edges differ for project.dependencies", self.errors())

    def test_all_locked_artifact_hashes_are_checked(self):
        package = next(p for p in self.lock["package"] if p["name"] == "numpy")
        package["wheels"][-1]["hash"] = "sha256:" + "0" * 64
        errors = audit_runner.validate_project("test", frozenset(self.owner["packages"]),
                                                self.releases, self.project, self.lock)
        self.assertIn("test: numpy locked artifact is not an authenticated nonyanked PyPI file", errors)

    def test_hatchling_build_requirement_is_exact(self):
        owner = next(p for p in self.audit["projects"] if p["name"] == "verification")
        project = audit_runner.load_toml(audit_runner.REPO_ROOT / owner["pyproject"])
        lock = audit_runner.load_toml(audit_runner.REPO_ROOT / owner["lock"])
        project["build-system"]["requires"] = ["hatchling>=1.32.0"]
        errors = validate_requirements("test", project, lock, self.releases, requirement_records(project))
        self.assertIn("test: build requirement hatchling must be exactly pinned", errors)

    def test_build_constraints_cannot_omit_transitive_backend_input(self):
        owner = next(p for p in self.audit["projects"] if p["name"] == "verification")
        project = audit_runner.load_toml(audit_runner.REPO_ROOT / owner["pyproject"])
        lock = audit_runner.load_toml(audit_runner.REPO_ROOT / owner["lock"])
        project["tool"]["uv"]["build-constraint-dependencies"].pop()
        errors = validate_requirements("test", project, lock, self.releases, requirement_records(project))
        self.assertIn("test: build constraints must pin the complete locked backend closure", errors)


if __name__ == "__main__":
    unittest.main()
