"""Validate the exact locked/offline crate catalog used by certification."""

from __future__ import annotations

import os
import subprocess
import tomllib
from pathlib import Path
from typing import Any

EXPECTED_PACKAGE_ALIASES = {"candle": "candle-core"}
# The independent backend fixture owns SQLx query-macro activation.
CATALOG_FEATURE_OVERRIDES = {
    "sqlx": ["runtime-tokio", "tls-rustls-ring-webpki", "postgres"],
}


def validate_crate_catalog(
    failures: list[str],
    repo_root: Path,
    required_crates: set[str],
    feature_policies: dict[str, dict[str, Any]],
) -> None:
    """Require every matrix crate at an exact lockfile version and cached offline."""
    catalog_path = repo_root / "crates" / "sifr_rust_interop_catalog" / "Cargo.toml"
    lock_path = repo_root / "Cargo.lock"
    try:
        catalog = tomllib.loads(catalog_path.read_text(encoding="utf-8"))
        lock = tomllib.loads(lock_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        failures.append(f"cannot load Rust interop crate catalog or lockfile: {error}")
        return

    _validate_catalog_data(
        failures,
        catalog,
        lock,
        required_crates,
        feature_policies,
    )
    if failures:
        return
    proc = subprocess.run(
        ["cargo", "fetch", "--locked", "--offline"],
        cwd=repo_root,
        env={**os.environ, "CARGO_NET_OFFLINE": "true"},
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip()
        failures.append(
            f"Rust interop crate catalog is not cacheable offline: {detail}"
        )


def _validate_catalog_data(
    failures: list[str],
    catalog: dict[str, Any],
    lock: dict[str, Any],
    required_crates: set[str],
    feature_policies: dict[str, dict[str, Any]],
) -> None:
    dependencies = catalog.get("dependencies")
    if not isinstance(dependencies, dict):
        failures.append("Rust interop crate catalog dependencies must be a table")
        return
    actual_aliases = set(dependencies)
    failures.extend(
        f"Rust interop crate catalog is missing {crate}"
        for crate in sorted(required_crates - actual_aliases)
    )
    failures.extend(
        f"Rust interop crate catalog has unexpected dependency {crate}"
        for crate in sorted(actual_aliases - required_crates)
    )
    if catalog.get("features") is not None:
        failures.append(
            "Rust interop crate catalog must not declare features that enable deferred crates"
        )

    locked_packages = {
        (str(package.get("name")), str(package.get("version")))
        for package in lock.get("package", [])
        if isinstance(package, dict)
    }
    for crate in sorted(required_crates & actual_aliases):
        _validate_dependency(
            failures,
            crate,
            dependencies.get(crate),
            locked_packages,
            feature_policies.get(crate, {}),
        )
    _validate_non_cargo_policies(failures, catalog, feature_policies)


def _validate_dependency(
    failures: list[str],
    crate: str,
    dependency: Any,
    locked_packages: set[tuple[str, str]],
    policy: dict[str, Any],
) -> None:
    if not isinstance(dependency, dict):
        failures.append(
            f"Rust interop crate catalog {crate} must use a dependency table"
        )
        return
    if dependency.get("optional") is not True:
        failures.append(f"Rust interop crate catalog {crate} must be optional")
    expected_package = EXPECTED_PACKAGE_ALIASES.get(crate, crate)
    package = str(dependency.get("package", crate))
    if package != expected_package:
        failures.append(
            f"Rust interop crate catalog {crate} package must be {expected_package}"
        )
    version = dependency.get("version")
    if not isinstance(version, str) or not version.startswith("=") or len(version) == 1:
        failures.append(f"Rust interop crate catalog {crate} must pin an exact version")
        return
    if (package, version.removeprefix("=")) not in locked_packages:
        failures.append(
            f"Rust interop crate catalog {crate} {version} is absent from Cargo.lock"
        )
    expected_default = policy.get("default_features", True)
    if dependency.get("default-features") != expected_default:
        failures.append(
            f"Rust interop crate catalog {crate} default-features must be {expected_default}"
        )
    expected_features = CATALOG_FEATURE_OVERRIDES.get(
        crate,
        policy.get("features", []),
    )
    if dependency.get("features", []) != expected_features:
        failures.append(
            f"Rust interop crate catalog {crate} features must be {expected_features!r}"
        )


def _validate_non_cargo_policies(
    failures: list[str],
    catalog: dict[str, Any],
    feature_policies: dict[str, dict[str, Any]],
) -> None:
    package = catalog.get("package")
    metadata = package.get("metadata") if isinstance(package, dict) else None
    interop = metadata.get("sifr-rust-interop") if isinstance(metadata, dict) else None
    if not isinstance(interop, dict):
        failures.append("Rust interop crate catalog metadata is required")
        return
    expected = {
        "schema-version": 1,
        "candle-backend": feature_policies.get("candle", {}).get("backend"),
        "prost-build-generated-output": feature_policies.get("prost-build", {}).get(
            "generated_output"
        ),
    }
    for key, value in expected.items():
        if interop.get(key) != value:
            failures.append(f"Rust interop crate catalog {key} must be {value}")


def run_self_test() -> tuple[int, str | None]:
    """Mutation-test exact catalog membership, pinning, and lockfile binding."""
    required = {"candle", "prost-build", "reqwest", "sqlx"}
    policies = {
        "candle": {"backend": "cpu-only", "default_features": False},
        "prost-build": {"generated_output": "deterministic"},
        "reqwest": {
            "default_features": False,
            "features": ["rustls", "json"],
        },
        "sqlx": {
            "default_features": False,
            "features": [
                "runtime-tokio",
                "tls-rustls-ring-webpki",
                "postgres",
                "macros",
            ],
        },
    }
    catalog = {
        "dependencies": {
            "candle": {
                "package": "candle-core",
                "version": "=0.11.0",
                "default-features": False,
                "optional": True,
            },
            "prost-build": {
                "version": "=0.14.4",
                "default-features": True,
                "optional": True,
            },
            "reqwest": {
                "version": "=0.13.4",
                "default-features": False,
                "features": ["rustls", "json"],
                "optional": True,
            },
            "sqlx": {
                "version": "=0.9.0",
                "default-features": False,
                "features": [
                    "runtime-tokio",
                    "tls-rustls-ring-webpki",
                    "postgres",
                ],
                "optional": True,
            },
        },
        "package": {
            "metadata": {
                "sifr-rust-interop": {
                    "schema-version": 1,
                    "candle-backend": "cpu-only",
                    "prost-build-generated-output": "deterministic",
                }
            }
        },
    }
    lock = {
        "package": [
            {"name": "candle-core", "version": "0.11.0"},
            {"name": "prost-build", "version": "0.14.4"},
            {"name": "reqwest", "version": "0.13.4"},
            {"name": "sqlx", "version": "0.9.0"},
        ]
    }
    control: list[str] = []
    _validate_catalog_data(control, catalog, lock, required, policies)
    if control:
        return 0, f"valid crate catalog was rejected: {control}"

    cases = (
        (
            {**catalog, "dependencies": []},
            lock,
            "catalog dependencies must be a table",
        ),
        (
            {
                **catalog,
                "dependencies": {
                    key: value
                    for key, value in catalog["dependencies"].items()
                    if key != "reqwest"
                },
            },
            lock,
            "catalog is missing reqwest",
        ),
        (
            {
                **catalog,
                "dependencies": {
                    **catalog["dependencies"],
                    "reqwest": "=0.13.4",
                },
            },
            lock,
            "reqwest must use a dependency table",
        ),
        (
            {
                **catalog,
                "dependencies": {
                    **catalog["dependencies"],
                    "reqwest": {
                        **catalog["dependencies"]["reqwest"],
                        "version": "0.13.4",
                    },
                },
            },
            lock,
            "must pin an exact version",
        ),
        (
            catalog,
            {"package": lock["package"][:-1]},
            "is absent from Cargo.lock",
        ),
        (
            {
                **catalog,
                "dependencies": {
                    **catalog["dependencies"],
                    "reqwest": {
                        **catalog["dependencies"]["reqwest"],
                        "features": ["json"],
                    },
                },
            },
            lock,
            "features must be",
        ),
        (
            {
                **catalog,
                "dependencies": {
                    **catalog["dependencies"],
                    "sqlx": {
                        **catalog["dependencies"]["sqlx"],
                        "features": [
                            "runtime-tokio",
                            "tls-rustls-ring-webpki",
                            "postgres",
                            "macros",
                        ],
                    },
                },
            },
            lock,
            "sqlx features must be",
        ),
        (
            {
                **catalog,
                "dependencies": {
                    **catalog["dependencies"],
                    "candle": {
                        key: value
                        for key, value in catalog["dependencies"]["candle"].items()
                        if key != "default-features"
                    },
                },
            },
            lock,
            "default-features must be False",
        ),
        (
            {
                **catalog,
                "dependencies": {
                    **catalog["dependencies"],
                    "prost-build": {
                        key: value
                        for key, value in catalog["dependencies"]["prost-build"].items()
                        if key != "default-features"
                    },
                },
            },
            lock,
            "default-features must be True",
        ),
        (
            {**catalog, "features": {"default": ["candle"]}},
            lock,
            "must not declare features",
        ),
        (
            {
                **catalog,
                "package": {
                    "metadata": {
                        "sifr-rust-interop": {
                            "schema-version": 1,
                            "candle-backend": "cpu-only",
                            "prost-build-generated-output": "nondeterministic",
                        }
                    }
                },
            },
            lock,
            "prost-build-generated-output must be deterministic",
        ),
        (
            {
                **catalog,
                "dependencies": {
                    **catalog["dependencies"],
                    "unexpected": {
                        "version": "=1.0.0",
                        "default-features": True,
                        "optional": True,
                    },
                },
            },
            lock,
            "catalog has unexpected dependency unexpected",
        ),
        (
            {
                **catalog,
                "dependencies": {
                    **catalog["dependencies"],
                    "reqwest": {
                        **catalog["dependencies"]["reqwest"],
                        "optional": False,
                    },
                },
            },
            lock,
            "reqwest must be optional",
        ),
        (
            {
                **catalog,
                "dependencies": {
                    **catalog["dependencies"],
                    "candle": {
                        **catalog["dependencies"]["candle"],
                        "package": "candle",
                    },
                },
            },
            lock,
            "candle package must be candle-core",
        ),
        (
            {
                "dependencies": catalog["dependencies"],
                "package": {"metadata": {}},
            },
            lock,
            "crate catalog metadata is required",
        ),
    )
    for case_catalog, case_lock, expected in cases:
        failures: list[str] = []
        _validate_catalog_data(
            failures,
            case_catalog,
            case_lock,
            required,
            policies,
        )
        if not any(expected in failure for failure in failures):
            return 0, f"crate catalog mutation did not report {expected!r}: {failures}"
    return len(cases) + 1, None
