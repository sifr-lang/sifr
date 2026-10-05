"""Exact CPU distribution identity and unchanged PyPI ownership controls."""
from __future__ import annotations

import copy
import json
import unittest
from unittest.mock import patch

import dependency_versions as audit_runner
from dependency_distribution import (
    CPU_MARKER, PYPI_MARKER, LOCK_CPU_MARKER, LOCK_PYPI_MARKER,
    cpu_requirements, runtime_distribution_version,
    validate_distribution_audit, validate_distribution_project,
)
from dependency_requirements import validate_requirements


class CpuDistributionTests(unittest.TestCase):
    def setUp(self):
        self.audit = json.loads(audit_runner.AUDIT_PATH.read_text())
        self.releases = audit_runner.release_map(self.audit)
        self.variant = self.releases["torch"]["verification_cpu"]
        self.owner = next(p for p in self.audit["projects"] if p["name"] == "python-interop")
        self.project = audit_runner.load_toml(audit_runner.REPO_ROOT / self.owner["pyproject"])
        self.lock = audit_runner.load_toml(audit_runner.REPO_ROOT / self.owner["lock"])
        self.cpu = next(p for p in self.lock["package"] if p["version"] == "2.14.0+cpu")
        self.root = next(p for p in self.lock["package"] if p["name"] == self.project["project"]["name"])

    def errors(self):
        return validate_distribution_project("test", self.project, self.lock, self.releases)

    def test_exact_cpu_and_pypi_graph_is_accepted(self):
        self.assertEqual(self.errors(), [])
        self.assertEqual(len(self.project["project"]["dependencies"]), 26)
        self.assertEqual(len(self.cpu["dependencies"]), 7)
        self.assertEqual(validate_requirements("test", self.project, self.lock,
                                              self.releases, self.owner["requirements"]), [])

    def test_resolution_forks_cannot_be_missing_broadened_or_swapped(self):
        base = next(p for p in self.lock["package"] if p["name"] == "torch" and p["version"] == "2.14.0")
        for owner in (self.lock, base, self.cpu):
            original = owner["resolution-markers"]
            for forks in (None, [], ["sys_platform == 'linux'"], [LOCK_PYPI_MARKER]):
                owner["resolution-markers"] = forks
                with self.subTest(owner=owner.get("name", "lock"), forks=forks):
                    self.assertTrue(any("resolution forks" in error for error in self.errors()))
            owner["resolution-markers"] = original

    def test_cpu_emitted_metadata_is_bound_to_authenticated_requirements(self):
        original = copy.deepcopy(self.cpu["metadata"])
        for mutation in ("missing", "requirement", "version", "optional-marker", "extras", "unknown"):
            metadata = copy.deepcopy(original)
            if mutation == "missing":
                metadata = None
            elif mutation == "requirement":
                metadata["requires-dist"].append({"name": "triton"})
            elif mutation == "version":
                next(row for row in metadata["requires-dist"] if row["name"] == "sympy")["specifier"] = ">=1"
            elif mutation == "optional-marker":
                next(row for row in metadata["requires-dist"] if row["name"] == "optree").pop("marker")
            elif mutation == "extras":
                metadata["provides-extras"].append("cuda")
            else:
                metadata["unreviewed"] = True
            self.cpu["metadata"] = metadata
            with self.subTest(mutation=mutation):
                self.assertTrue(any("authenticated wheel" in error for error in self.errors()))
        self.cpu["metadata"] = original

    def test_historical_hand_constructed_lock_is_rejected(self):
        self.cpu.pop("metadata")
        self.cpu.pop("resolution-markers")
        self.assertTrue(self.errors())

    def test_audit_restricts_source_owner_platform_and_version(self):
        for key, value in (
            ("project", "demo"), ("pyproject", "demos/python_dlpack/pyproject.toml"),
            ("marker", "sys_platform == 'linux'"), ("distribution_version", "2.14.0+other"),
            ("upstream_version", "2.13.0"), ("url", self.variant["url"].replace("download.", "download-r2.")),
            ("metadata_url", self.variant["url"]), ("index_url", "https://example.org/torch/"),
            ("sha256", "not-a-hash"), ("metadata_sha256", ""), ("index_sha256", ""),
            ("requires_python", "<3.14"), ("size_bytes", 0),
        ):
            releases = copy.deepcopy(self.releases)
            releases["torch"]["verification_cpu"][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_distribution_audit(releases)

    def test_missing_or_other_package_variant_is_rejected(self):
        for remove in (True, False):
            releases = copy.deepcopy(self.releases)
            if remove:
                del releases["torch"]["verification_cpu"]
            else:
                releases["numpy"]["verification_cpu"] = self.variant
            with self.subTest(remove=remove), self.assertRaises(ValueError):
                validate_distribution_audit(releases)

    def test_explicit_source_cannot_be_absent_broadened_or_substituted(self):
        source = self.project["tool"]["uv"]["sources"]
        original = copy.deepcopy(source["torch"])
        values = [None, [], [*original, *original],
                  [{"url": self.variant["url"], "marker": "sys_platform == 'linux'"}],
                  [{"url": "https://example.org/torch.whl", "marker": CPU_MARKER}]]
        for value in values:
            source["torch"] = value
            with self.subTest(value=value):
                self.assertTrue(self.errors())
        source["torch"] = original

    def test_source_cannot_leak_into_demo(self):
        demo = next(p for p in self.audit["projects"] if p["name"] == "dlpack-demo")
        project = audit_runner.load_toml(audit_runner.REPO_ROOT / demo["pyproject"])
        lock = audit_runner.load_toml(audit_runner.REPO_ROOT / demo["lock"])
        self.assertEqual(validate_distribution_project("demo", project, lock, self.releases), [])
        project["tool"]["uv"]["sources"] = copy.deepcopy(self.project["tool"]["uv"]["sources"])
        self.assertTrue(validate_distribution_project("demo", project, lock, self.releases))

    def test_locked_variant_omission_duplicate_and_version_are_rejected(self):
        for mutation in ("remove", "duplicate", "version"):
            original = copy.deepcopy(self.lock["package"])
            if mutation == "remove":
                self.lock["package"].remove(self.cpu)
            elif mutation == "duplicate":
                self.lock["package"].append(copy.deepcopy(self.cpu))
            else:
                self.cpu["version"] = "2.14.0+wrong"
            with self.subTest(mutation=mutation):
                self.assertTrue(self.errors())
            self.lock["package"] = original
            self.cpu = next(p for p in original if p["version"] == "2.14.0+cpu")

    def test_locked_cpu_wheel_and_source_are_exact(self):
        for key, value in (
            ("source", {"registry": "https://pypi.org/simple"}),
            ("wheels", []),
            ("wheels", [{"url": self.variant["url"], "hash": "sha256:" + "0" * 64}]),
            ("wheels", [{"url": "https://example.org/torch.whl", "hash": "sha256:" + self.variant["sha256"]}]),
            ("sdist", {"url": self.variant["url"]}),
        ):
            original = copy.deepcopy(self.cpu)
            self.cpu[key] = value
            with self.subTest(key=key, value=value):
                self.assertTrue(self.errors())
            self.cpu.clear()
            self.cpu.update(original)

    def test_direct_edge_partition_cannot_change_platform_or_source(self):
        for edge in [p for p in self.root["dependencies"] if p["name"] == "torch"]:
            for key, value in (("marker", "sys_platform == 'linux'"),
                               ("source", {"registry": "https://example.org/simple"}),
                               ("version", "2.14.0+other")):
                original = edge[key]
                edge[key] = value
                with self.subTest(edge=edge, key=key):
                    self.assertTrue(self.errors())
                edge[key] = original

    def test_pypi_wheel_authentication_is_preserved(self):
        base = next(p for p in self.lock["package"] if p["name"] == "torch" and p["version"] == "2.14.0")
        base["wheels"][0]["hash"] = "sha256:" + "0" * 64
        errors = audit_runner.validate_project("test", frozenset(self.owner["packages"]),
                                                self.releases, self.project, self.lock)
        self.assertIn("test: torch locked artifact is not an authenticated nonyanked PyPI file", errors)

    def test_cpu_dependency_metadata_and_versions_are_enforced(self):
        original = copy.deepcopy(self.cpu["dependencies"])
        for rows in (original[:-1], [*original, {"name": "triton"}],
                     [{"name": row["name"], "marker": CPU_MARKER} for row in original]):
            self.cpu["dependencies"] = rows
            self.assertTrue(self.errors())
        self.cpu["dependencies"] = original
        setuptools = next(p for p in self.lock["package"] if p["name"] == "setuptools")
        setuptools["version"] = "1.0"
        self.assertTrue(self.errors())

    def test_cpu_metadata_cannot_add_gpu_or_hide_runtime_dependency(self):
        for requirement in ("triton", "filelock", "filelock @ https://example.org/metadata"):
            releases = copy.deepcopy(self.releases)
            releases["torch"]["verification_cpu"]["requires_dist"].append(requirement)
            with self.subTest(requirement=requirement), self.assertRaises(ValueError):
                validate_distribution_audit(releases)
        self.assertNotIn("optree", cpu_requirements(self.variant))

    def test_installed_version_uses_exact_distribution_on_only_cpu_lane(self):
        for platform, machine, expected in (
            ("linux", "x86_64", "2.14.0+cpu"), ("linux", "aarch64", "2.14.0"),
            ("darwin", "arm64", "2.14.0"), ("win32", "AMD64", "2.14.0"),
        ):
            with self.subTest(platform=platform, machine=machine):
                self.assertEqual(runtime_distribution_version("torch", self.releases,
                    {"sys_platform": platform, "platform_machine": machine}), expected)
        for installed in ("2.14.0", "2.14.0+cu130", "2.14.1+cpu"):
            with patch("dependency_distribution.default_environment", return_value={
                "sys_platform": "linux", "platform_machine": "x86_64"}), \
                 patch.object(audit_runner, "version", return_value=installed):
                with self.assertRaises(RuntimeError):
                    audit_runner.runtime_version_marker("torch")
        with patch("dependency_distribution.default_environment", return_value={
            "sys_platform": "linux", "platform_machine": "x86_64"}), \
             patch.object(audit_runner, "version", return_value="2.14.0+cpu"):
            self.assertEqual(audit_runner.runtime_version_marker("torch"), "torch=2.14.0+cpu")

    def test_source_requirement_metadata_cannot_substitute_markers(self):
        for row in self.root["metadata"]["requires-dist"]:
            if row["name"] == "torch":
                original = row["marker"]
                row["marker"] = LOCK_CPU_MARKER if original == LOCK_PYPI_MARKER else LOCK_PYPI_MARKER
                errors = validate_requirements("test", self.project, self.lock,
                                               self.releases, self.owner["requirements"])
                self.assertIn("test: uv.lock direct requirement metadata differs from manifest", errors)
                row["marker"] = original


if __name__ == "__main__":
    unittest.main()
