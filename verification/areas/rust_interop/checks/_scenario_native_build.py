"""Exact native build-script scenario policy and mutation coverage."""

from __future__ import annotations

import shutil
import tempfile
import tomllib
from pathlib import Path
from typing import Any, Callable

ScenarioValidator = Callable[[list[str], str, Path, dict[str, Any]], int]

NATIVE_BUILD_SCENARIO_TOKENS = (
    'bindgen_upstream = { package = "bindgen", version = "=0.72.1"',
    'cc_upstream = { package = "cc", version = "=1.4.4"',
    'cxx = { version = "=1.0.199"',
    'zstd_upstream = { package = "zstd", version = "=0.13.3"',
    "cc_upstream::Build::new()",
    '.compile("sifr_cc_probe")',
    "bindgen_upstream::Builder::default()",
    '.allowlist_function("sifr_bindgen_probe")',
    "#[cxx::bridge]",
    "zstd_upstream::stream::encode_all",
    "sifr-bindgen-bindings.rs",
    "sifr-cc-evidence.txt",
    "sifr-cxx-evidence.txt",
    "sifr-zstd-evidence.txt",
    '"c++", "cxxbridge1", "link-cplusplus", "sifr_cc_probe", "sifr_zstd_probe", "stdc++", "zstd"',
)

EXPECTED_BUILD_SCRIPTS = ["cc", "bindgen", "cxx", "zstd"]
EXPECTED_NATIVE_LINKS = [
    "c++",
    "cxxbridge1",
    "link-cplusplus",
    "sifr_cc_probe",
    "sifr_zstd_probe",
    "stdc++",
    "zstd",
]
EXPECTED_NO_PANIC = [
    "bindgen.artifact",
    "bridge.native.compress",
    "bridge.native.decompress",
    "cc.artifact",
    "cxx.artifact",
    "zstd.artifact",
]

WRAPPERS = {
    "bindgen": {
        "package": "sifr-bindgen-probe",
        "build_dependencies": {"bindgen_upstream": {"workspace": True}},
        "links": None,
    },
    "cc": {
        "package": "sifr-cc-probe",
        "build_dependencies": {"cc_upstream": {"workspace": True}},
        "links": "sifr_cc_probe",
    },
    "cxx": {
        "package": "sifr-cxx-probe",
        "dependencies": {"cxx": {"workspace": True}},
        "links": None,
    },
    "zstd": {
        "package": "sifr-zstd-probe",
        "dependencies": {"zstd_upstream": {"workspace": True}},
        "links": "sifr_zstd_probe",
    },
}


def validate_native_build_scenario(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    cargo: dict[str, Any],
    dependencies: dict[str, Any],
    trust: dict[str, Any],
    example_dir: Path,
) -> None:
    workspace = cargo.get("workspace", {})
    if workspace.get("members") != [
        "rust/cc",
        "rust/bindgen",
        "rust/cxx",
        "rust/zstd",
    ]:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml workspace members must equal "
            "the four native wrapper crates"
        )
    expected_workspace_dependencies = {
        "bindgen_upstream": {
            "package": "bindgen",
            "version": "=0.72.1",
            "default-features": True,
        },
        "cc_upstream": {
            "package": "cc",
            "version": "=1.4.4",
            "default-features": True,
        },
        "cxx": {"version": "=1.0.199", "default-features": True},
        "zstd_upstream": {
            "package": "zstd",
            "version": "=0.13.3",
            "default-features": True,
        },
    }
    if workspace.get("dependencies") != expected_workspace_dependencies:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml workspace dependencies must "
            "exact-pin cc, bindgen, cxx, and zstd"
        )

    for name, policy in WRAPPERS.items():
        expected_root = {
            "package": policy["package"],
            "path": f"rust/{name}",
        }
        if dependencies.get(name) != expected_root:
            failures.append(
                f"{fixture_id}: {raw_path}/Cargo.toml dependency {name} "
                f"must equal {expected_root!r}"
            )
        _validate_wrapper_manifest(
            failures,
            fixture_id,
            raw_path,
            example_dir / f"rust/{name}/Cargo.toml",
            name,
            policy,
        )

    _require_exact_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-build-scripts",
        EXPECTED_BUILD_SCRIPTS,
    )
    _require_exact_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "native-links",
        EXPECTED_NATIVE_LINKS,
    )
    _require_exact_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-no-panic",
        EXPECTED_NO_PANIC,
    )
    _validate_build_sources(failures, fixture_id, raw_path, example_dir)


def run_native_build_self_test(
    area_root: Path,
    validate_scenarios: ScenarioValidator,
) -> tuple[int, str | None]:
    source = area_root / "fixtures/native_build_script"
    raw_examples = {"native_trust_package": "examples/native_trust_package"}
    cases = 0
    with tempfile.TemporaryDirectory(
        prefix="sifr-rust-native-build-scenario-self-test-"
    ) as raw_temp:
        fixture_dir = Path(raw_temp) / "native_build_script"
        shutil.copytree(source, fixture_dir, ignore=shutil.ignore_patterns("target"))
        baseline_failures: list[str] = []
        validate_scenarios(
            baseline_failures,
            "native_build_script",
            fixture_dir,
            raw_examples,
        )
        if baseline_failures:
            return cases, f"native-build baseline failed: {baseline_failures}"
        cases += 1

        mutation_cases = (
            (
                "cc pin drift",
                "examples/native_trust_package/Cargo.toml",
                'version = "=1.4.4"',
                'version = "1.4.4"',
                "workspace dependencies must exact-pin",
            ),
            (
                "wrapper package identity drift",
                "examples/native_trust_package/rust/cc/Cargo.toml",
                'name = "sifr-cc-probe"',
                'name = "cc"',
                "wrapper cc package must be sifr-cc-probe",
            ),
            (
                "wrapper build-script declaration drift",
                "examples/native_trust_package/rust/zstd/Cargo.toml",
                'build = "build.rs"',
                'build = "other.rs"',
                "wrapper zstd must declare build.rs",
            ),
            (
                "direct native identity drift",
                "examples/native_trust_package/rust/zstd/Cargo.toml",
                'links = "sifr_zstd_probe"',
                'links = "zstd"',
                "wrapper zstd links must be sifr_zstd_probe",
            ),
            (
                "build-script trust drift",
                "examples/native_trust_package/sifr.toml",
                '"bindgen", ',
                "",
                "trust.rust-build-scripts",
            ),
            (
                "native-link envelope drift",
                "examples/native_trust_package/sifr.toml",
                '"stdc++", ',
                "",
                "trust.native-links",
            ),
            (
                "no-panic trust drift",
                "examples/native_trust_package/sifr.toml",
                '"cxx.artifact", ',
                "",
                "trust.rust-no-panic",
            ),
            (
                "cc compile drift",
                "examples/native_trust_package/rust/cc/build.rs",
                '.compile("sifr_cc_probe")',
                '.compile("other")',
                'must contain .compile("sifr_cc_probe")',
            ),
            (
                "bindgen generation drift",
                "examples/native_trust_package/rust/bindgen/build.rs",
                '.allowlist_function("sifr_bindgen_probe")',
                '.allowlist_function("other")',
                'must contain .allowlist_function("sifr_bindgen_probe")',
            ),
            (
                "cxx bridge drift",
                "examples/native_trust_package/rust/cxx/src/lib.rs",
                "#[cxx::bridge]",
                "#[cxx::other]",
                "must contain #[cxx::bridge]",
            ),
            (
                "zstd execution drift",
                "examples/native_trust_package/rust/zstd/src/lib.rs",
                "zstd_upstream::stream::encode_all",
                "zstd_upstream::stream::decode_all",
                "must contain zstd_upstream::stream::encode_all",
            ),
            (
                "artifact version drift",
                "examples/native_trust_package/rust/cc/build.rs",
                "cc=1.4.4;compiled=sifr_cc_probe",
                "cc=0.0.0;compiled=sifr_cc_probe",
                "must contain cc=1.4.4;compiled=sifr_cc_probe",
            ),
            (
                "source-tree artifact drift",
                "examples/native_trust_package/rust/zstd/build.rs",
                'std::env::var("OUT_DIR")',
                'std::env::var("CARGO_MANIFEST_DIR")',
                "must keep build artifacts under OUT_DIR",
            ),
        )
        for name, relative_path, before, after, expected in mutation_cases:
            path = fixture_dir / relative_path
            original = path.read_text(encoding="utf-8")
            if before not in original:
                return cases, f"{name} self-test setup token is missing"
            path.write_text(original.replace(before, after, 1), encoding="utf-8")
            failures: list[str] = []
            validate_scenarios(
                failures,
                "native_build_script",
                fixture_dir,
                raw_examples,
            )
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1
    return cases, None


def _validate_wrapper_manifest(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    path: Path,
    name: str,
    policy: dict[str, Any],
) -> None:
    try:
        cargo = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        failures.append(f"{fixture_id}: {raw_path}/{name}/Cargo.toml is invalid: {error}")
        return
    package = cargo.get("package", {})
    if package.get("name") != policy["package"]:
        failures.append(
            f"{fixture_id}: {raw_path} wrapper {name} package must be "
            f"{policy['package']}"
        )
    if package.get("build") != "build.rs":
        failures.append(
            f"{fixture_id}: {raw_path} wrapper {name} must declare build.rs"
        )
    if package.get("links") != policy["links"]:
        failures.append(
            f"{fixture_id}: {raw_path} wrapper {name} links must be "
            f"{policy['links']}"
        )
    for section in ("dependencies", "build_dependencies"):
        expected = policy.get(section)
        if expected is not None and cargo.get(section.replace("_", "-")) != expected:
            failures.append(
                f"{fixture_id}: {raw_path} wrapper {name} {section} must equal "
                f"{expected!r}"
            )
    if not path.with_name("build.rs").is_file():
        failures.append(
            f"{fixture_id}: {raw_path} wrapper {name} build.rs is required"
        )


def _validate_build_sources(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    example_dir: Path,
) -> None:
    requirements = {
        "rust/cc/build.rs": (
            "cc_upstream::Build::new()",
            '.compile("sifr_cc_probe")',
            "sifr-cc-evidence.txt",
            "cc=1.4.4;compiled=sifr_cc_probe",
        ),
        "rust/bindgen/build.rs": (
            "bindgen_upstream::Builder::default()",
            '.allowlist_function("sifr_bindgen_probe")',
            "sifr-bindgen-bindings.rs",
            "sifr-bindgen-evidence.txt",
            "bindgen=0.72.1;function=sifr_bindgen_probe",
        ),
        "rust/cxx/build.rs": (
            "sifr-cxx-evidence.txt",
            "cxx=1.0.199;bridge=sifr_cxx_probe",
        ),
        "rust/cxx/src/lib.rs": ("#[cxx::bridge]", "sifr_cxx_probe_value"),
        "rust/zstd/build.rs": (
            "sifr-zstd-evidence.txt",
            "zstd=0.13.3;level=3",
        ),
        "rust/zstd/src/lib.rs": (
            "zstd_upstream::stream::encode_all",
            "zstd_upstream::stream::decode_all",
        ),
        "rust/cc/native/probe.c": (
            "unsigned int sifr_cc_probe(void)",
            "return 1263U;",
        ),
    }
    for relative_path, tokens in requirements.items():
        path = example_dir / relative_path
        try:
            source = path.read_text(encoding="utf-8")
        except OSError as error:
            failures.append(
                f"{fixture_id}: {raw_path}/{relative_path} is unreadable: {error}"
            )
            continue
        for token in tokens:
            if token not in source:
                failures.append(
                    f"{fixture_id}: {raw_path} {relative_path} must contain {token}"
                )
        if relative_path.endswith("build.rs"):
            if "OUT_DIR" not in source or "CARGO_MANIFEST_DIR" in source:
                failures.append(
                    f"{fixture_id}: {raw_path} {relative_path} must keep build "
                    "artifacts under OUT_DIR"
                )


def _require_exact_trust(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    trust: Any,
    key: str,
    expected: list[str],
) -> None:
    actual = trust.get(key) if isinstance(trust, dict) else None
    if actual != expected:
        failures.append(
            f"{fixture_id}: {raw_path}/sifr.toml trust.{key} must equal "
            f"{expected!r}"
        )
