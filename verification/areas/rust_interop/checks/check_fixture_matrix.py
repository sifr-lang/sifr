"""Validate the Rust interop fixture matrix inventory."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

from _binding_helpers import contains_empty_pass_body as _contains_empty_pass_body
from _binding_helpers import decorated_function_name as _decorated_function_name
from _binding_helpers import package_example_binding_token
from _binding_helpers import rust_bound_declarations as _rust_bound_declarations
from _binding_helpers import verifier_binds_call as _verifier_binds_call
from _crate_catalog import validate_crate_catalog
from _crate_catalog import run_self_test as run_catalog_self_test
from _evidence_expectations import run_self_test as run_expectation_self_test
from _evidence_expectations import validate_evidence_expectation
from _matrix_inventory import ALLOWED_EXECUTION_KINDS
from _matrix_inventory import EXECUTION_KINDS
from _matrix_inventory import EXPECTED_FEATURE_POLICIES
from _matrix_inventory import REQUIRED_CRATES
from _matrix_inventory import REQUIRED_DIAGNOSTICS
from _matrix_inventory import REQUIRED_FIXTURES
from _matrix_inventory import VALID_EVIDENCE_STATUS
from _provenance_checks import load_profiles
from _provenance_checks import run_self_test as run_provenance_self_test
from _provenance_checks import validate_evidence_provenance
from _scenario_checks import run_self_test as run_scenario_self_test
from _scenario_checks import validate_scenario_examples

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "rust_interop"
MATRIX_PATH = AREA_ROOT / "data" / "rust_interop_fixture_matrix.json"
FIXTURES_ROOT = AREA_ROOT / "fixtures"


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    if args == ["--self-test"]:
        return _run_self_test()
    if args:
        print(f"usage: {Path(__file__).name} [--self-test]", file=sys.stderr)
        return 2

    matrix = json.loads(MATRIX_PATH.read_text(encoding="utf-8"))
    failures: list[str] = []

    if matrix.get("schema_version") != 1:
        failures.append("matrix schema_version must be 1")
    if matrix.get("phase") != "39_rust_interop":
        failures.append("matrix phase must be 39_rust_interop")
    if matrix.get("bridge_version") != 1:
        failures.append("matrix bridge_version must be 1")

    diagnostics = set(matrix.get("diagnostic_families", {}))
    failures.extend(
        f"missing diagnostic family reservation: {code}"
        for code in sorted(REQUIRED_DIAGNOSTICS.difference(diagnostics))
    )
    unexpected_diagnostics = diagnostics.difference(REQUIRED_DIAGNOSTICS)
    failures.extend(
        f"unexpected diagnostic family reservation: {code}"
        for code in sorted(unexpected_diagnostics)
    )

    fixtures = matrix.get("fixtures", [])
    if not isinstance(fixtures, list):
        failures.append("fixtures must be a list")
        fixtures = []
    fixture_ids = [str(fixture.get("id")) for fixture in fixtures if isinstance(fixture, dict)]
    fixture_id_set = set(fixture_ids)
    if len(fixture_ids) != len(fixture_id_set):
        failures.append("fixture ids must be unique")
    failures.extend(f"missing fixture matrix entry: {item}" for item in sorted(REQUIRED_FIXTURES - fixture_id_set))
    failures.extend(f"unexpected fixture matrix entry: {item}" for item in sorted(fixture_id_set - REQUIRED_FIXTURES))

    discovered_dirs = {path.name for path in FIXTURES_ROOT.iterdir() if path.is_dir()}
    failures.extend(f"missing fixture directory: {item}" for item in sorted(REQUIRED_FIXTURES - discovered_dirs))
    failures.extend(f"unexpected fixture directory: {item}" for item in sorted(discovered_dirs - REQUIRED_FIXTURES))
    for fixture_id in sorted(REQUIRED_FIXTURES & discovered_dirs):
        if not (FIXTURES_ROOT / fixture_id / "README.md").is_file():
            failures.append(f"{fixture_id}: fixture README.md is required for evidence notes")
        if not (FIXTURES_ROOT / fixture_id / "fixture.json").is_file():
            failures.append(f"{fixture_id}: fixture.json is required for evidence files")

    covered_crates: set[str] = set()
    package_example_count = 0
    scenario_example_count = 0
    profiles = load_profiles(REPO_ROOT)
    used_evidence_tests: dict[tuple[str, str], str] = {}
    for fixture in fixtures:
        if not isinstance(fixture, dict):
            failures.append("fixture entries must be objects")
            continue
        fixture_id = str(fixture.get("id"))
        tier = fixture.get("tier")
        if tier not in {0, 1, 2, 3, 4}:
            failures.append(f"{fixture_id}: tier must be 0..4")
        if not fixture.get("capability"):
            failures.append(f"{fixture_id}: capability is required")
        _validate_execution_semantics(failures, fixture)
        crates = fixture.get("required_crates", [])
        if not isinstance(crates, list):
            failures.append(f"{fixture_id}: required_crates must be a list")
            crates = []
        covered_crates.update(str(crate) for crate in crates)
        _validate_feature_policies(failures, fixture_id, fixture.get("features"), crates)
        _validate_evidence(failures, fixture_id, fixture.get("positive_evidence"), "positive_evidence")
        _validate_evidence(failures, fixture_id, fixture.get("negative_evidence"), "negative_evidence")
        package_count, scenario_count = _validate_fixture_files(
            failures,
            fixture,
            crates,
            profiles,
            used_evidence_tests,
        )
        package_example_count += package_count
        scenario_example_count += scenario_count

    failures.extend(f"required crate lacks fixture coverage: {crate}" for crate in sorted(REQUIRED_CRATES - covered_crates))
    validate_crate_catalog(
        failures,
        REPO_ROOT,
        REQUIRED_CRATES,
        EXPECTED_FEATURE_POLICIES,
    )

    if failures:
        for failure in failures:
            print(f"rust interop fixture matrix error: {failure}", file=sys.stderr)
        return 1
    print(
        "rust interop fixture matrix ok: "
        f"fixtures={len(fixture_id_set)} diagnostics={len(diagnostics)} "
        f"crates={len(covered_crates)} package_examples={package_example_count} "
        f"scenario_examples={scenario_example_count}"
    )
    return 0


def _validate_execution_semantics(
    failures: list[str],
    fixture: dict[str, Any],
) -> None:
    fixture_id = str(fixture.get("id"))
    tier = fixture.get("tier")
    execution_kind = fixture.get("execution_kind")
    if execution_kind not in EXECUTION_KINDS:
        failures.append(f"{fixture_id}: invalid execution_kind")
    elif tier in ALLOWED_EXECUTION_KINDS and execution_kind not in ALLOWED_EXECUTION_KINDS[tier]:
        allowed = ", ".join(sorted(ALLOWED_EXECUTION_KINDS[tier]))
        failures.append(
            f"{fixture_id}: tier {tier} does not allow execution_kind {execution_kind}; "
            f"allowed: {allowed}"
        )

    crates = fixture.get("required_crates")
    has_crates = isinstance(crates, list) and bool(crates)
    rationale = fixture.get("diagnostic_crate_rationale")
    if execution_kind != "compiler-diagnostic":
        if rationale is not None:
            failures.append(
                f"{fixture_id}: diagnostic_crate_rationale is allowed only for compiler-diagnostic rows"
            )
        return
    if rationale is not None:
        _validate_diagnostic_crate_rationale(failures, fixture_id, rationale)
    elif has_crates:
        failures.append(
            f"{fixture_id}: compiler-diagnostic rows with required_crates need "
            "diagnostic_crate_rationale"
        )


def _validate_diagnostic_crate_rationale(
    failures: list[str],
    fixture_id: str,
    rationale: Any,
) -> None:
    if not isinstance(rationale, dict) or not rationale:
        failures.append(
            f"{fixture_id}: compiler-diagnostic rows with required_crates need "
            "diagnostic_crate_rationale"
        )
        return
    if set(rationale) != {"purpose", "linked", "executed"}:
        failures.append(
            f"{fixture_id}: diagnostic_crate_rationale must contain exactly "
            "purpose, linked, and executed"
        )
    purpose = rationale.get("purpose")
    if not isinstance(purpose, str) or not purpose.strip():
        failures.append(f"{fixture_id}: diagnostic_crate_rationale.purpose must be non-empty")
    if rationale.get("linked") is not False:
        failures.append(f"{fixture_id}: diagnostic_crate_rationale.linked must be false")
    if rationale.get("executed") is not False:
        failures.append(f"{fixture_id}: diagnostic_crate_rationale.executed must be false")


def _run_self_test() -> int:
    allowed_pairs = {
        (tier, execution_kind)
        for tier, execution_kinds in ALLOWED_EXECUTION_KINDS.items()
        for execution_kind in execution_kinds
    }
    cases = 0
    for tier in range(5):
        for execution_kind in sorted(EXECUTION_KINDS):
            fixture = {
                "id": f"tier_{tier}_{execution_kind}",
                "tier": tier,
                "execution_kind": execution_kind,
                "required_crates": [],
            }
            failures: list[str] = []
            _validate_execution_semantics(failures, fixture)
            is_allowed = (tier, execution_kind) in allowed_pairs
            has_pair_failure = any("does not allow execution_kind" in item for item in failures)
            if is_allowed == has_pair_failure:
                print(
                    "rust interop fixture matrix self-test error: "
                    f"tier {tier}/{execution_kind} allowed={is_allowed} failures={failures}",
                    file=sys.stderr,
                )
                return 1
            cases += 1

    rationale = {
        "purpose": "crate API supplies a rejected diagnostic shape only",
        "linked": False,
        "executed": False,
    }
    mutation_cases = (
        (
            "missing diagnostic rationale",
            {
                "id": "diagnostic_missing",
                "tier": 0,
                "execution_kind": "compiler-diagnostic",
                "required_crates": ["example"],
            },
            "need diagnostic_crate_rationale",
        ),
        (
            "malformed diagnostic rationale",
            {
                "id": "diagnostic_malformed",
                "tier": 0,
                "execution_kind": "compiler-diagnostic",
                "required_crates": ["example"],
                "diagnostic_crate_rationale": {"purpose": ""},
            },
            "must contain exactly",
        ),
        (
            "rationale on cargo probe",
            {
                "id": "cargo_probe_rationale",
                "tier": 1,
                "execution_kind": "cargo-probe",
                "required_crates": [],
                "diagnostic_crate_rationale": rationale,
            },
            "allowed only for compiler-diagnostic",
        ),
        (
            "malformed rationale without crates",
            {
                "id": "diagnostic_empty_crates_malformed",
                "tier": 0,
                "execution_kind": "compiler-diagnostic",
                "required_crates": [],
                "diagnostic_crate_rationale": {"junk": 1},
            },
            "must contain exactly",
        ),
        (
            "tier one downgrade",
            {
                "id": "tier_one_contract",
                "tier": 1,
                "execution_kind": "contract-only",
                "required_crates": [],
            },
            "does not allow execution_kind contract-only",
        ),
    )
    for name, fixture, expected in mutation_cases:
        failures = []
        _validate_execution_semantics(failures, fixture)
        if not any(expected in failure for failure in failures):
            print(
                f"rust interop fixture matrix self-test error: {name} did not report {expected!r}",
                file=sys.stderr,
            )
            return 1
        cases += 1

    proc_macro_example = """# fixture: proc_macro_trust
# package-example: serde_derive
# required-crate: serde_derive
# execution-kind: cargo-probe
# expected-result: package-example
class PackageExampleError(Error):
    message: str
@rust(bridge.generated.decode, panic=trusted_no_panic)
def decode(input: bytes) -> str: ...
def verify_serde_derive_package() -> str:
    marker: str = decode(b"sifr")
    assert "serde_derive=1.0.228;upstream=compiled;sifr_wrapper_macro=executed" in marker
    return marker
"""
    proc_macro_failures: list[str] = []
    _validate_package_example_text(
        proc_macro_failures,
        "proc_macro_trust",
        "examples/serde_derive.sifr",
        proc_macro_example,
        "serde_derive",
        "cargo-probe",
    )
    if proc_macro_failures:
        print(
            "rust interop fixture matrix self-test error: "
            f"valid proc-macro bridge example failed: {proc_macro_failures}",
            file=sys.stderr,
        )
        return 1
    cases += 1

    proc_macro_marker_failures: list[str] = []
    _validate_package_example_text(
        proc_macro_marker_failures,
        "proc_macro_trust",
        "examples/serde_derive.sifr",
        proc_macro_example.replace(
            "serde_derive=1.0.228;upstream=compiled;sifr_wrapper_macro=executed",
            "missing",
        ),
        "serde_derive",
        "cargo-probe",
    )
    if not any(
        "must observe the generated serde_derive marker" in failure
        for failure in proc_macro_marker_failures
    ):
        print(
            "rust interop fixture matrix self-test error: "
            "proc-macro marker mutation passed",
            file=sys.stderr,
        )
        return 1
    cases += 1

    negative_proc_macro_source = (
        FIXTURES_ROOT
        / "proc_macro_trust"
        / "negative"
        / "untrusted_proc_macro_rejected_pre_execution.sifr"
    ).read_text(encoding="utf-8")
    negative_signature_failures: list[str] = []
    _validate_evidence_example_text(
        negative_signature_failures,
        "proc_macro_trust",
        "negative/untrusted_proc_macro_rejected_pre_execution.sifr",
        negative_proc_macro_source.replace(
            "decode_without_proc_macro_trust(input: bytes) -> str",
            "decode_without_proc_macro_trust(input: bytes) -> bytes",
        ),
        "untrusted_proc_macro_rejected_pre_execution",
    )
    if not any(
        "must match the scenario bridge `str` return" in failure
        for failure in negative_signature_failures
    ):
        print(
            "rust interop fixture matrix self-test error: "
            "negative proc-macro signature mutation passed",
            file=sys.stderr,
        )
        return 1
    cases += 1

    feature_failures: list[str] = []
    _validate_feature_policies(
        feature_failures,
        "zerocopy_feature_missing",
        {},
        ["zerocopy"],
    )
    if not any(
        "feature policy for zerocopy must be" in failure
        for failure in feature_failures
    ):
        print(
            "rust interop fixture matrix self-test error: "
            f"missing zerocopy feature policy passed: {feature_failures}",
            file=sys.stderr,
        )
        return 1
    cases += 1

    alignment_failures: list[str] = []
    _validate_manifest_alignment(
        alignment_failures,
        "diagnostic_mismatch",
        {"diagnostic_crate_rationale": rationale},
        {
            "diagnostic_crate_rationale": {
                **rationale,
                "purpose": "different rationale",
            }
        },
    )
    if not any("diagnostic_crate_rationale must match" in item for item in alignment_failures):
        print(
            "rust interop fixture matrix self-test error: mismatched manifest rationale passed",
            file=sys.stderr,
        )
        return 1
    cases += 1

    diagnostic_failures: list[str] = []
    _validate_diagnostic_family_alignment(
        diagnostic_failures,
        "diagnostic_family_mismatch",
        {"diagnostic_family": "SIFR-RUST-CARGO-0001"},
        {
            "negative": {
                "expected_result": "diagnostic",
                "expected_diagnostic": "SIFR-RUST-RESOLVE-0001",
            }
        },
    )
    if not any("diagnostic_family must match" in item for item in diagnostic_failures):
        print(
            "rust interop fixture matrix self-test error: diagnostic family drift passed",
            file=sys.stderr,
        )
        return 1
    cases += 1

    provenance_cases, provenance_error = run_provenance_self_test()
    if provenance_error is not None:
        print(
            f"rust interop fixture matrix self-test error: {provenance_error}",
            file=sys.stderr,
        )
        return 1
    cases += provenance_cases

    expectation_cases, expectation_error = run_expectation_self_test(
        REQUIRED_DIAGNOSTICS
    )
    if expectation_error is not None:
        print(
            f"rust interop fixture matrix self-test error: {expectation_error}",
            file=sys.stderr,
        )
        return 1
    cases += expectation_cases

    catalog_cases, catalog_error = run_catalog_self_test()
    if catalog_error is not None:
        print(
            f"rust interop fixture matrix self-test error: {catalog_error}",
            file=sys.stderr,
        )
        return 1
    cases += catalog_cases

    scenario_cases, scenario_error = run_scenario_self_test()
    if scenario_error is not None:
        print(
            f"rust interop fixture matrix self-test error: {scenario_error}",
            file=sys.stderr,
        )
        return 1
    cases += scenario_cases

    print(f"rust interop fixture matrix self-test ok: cases={cases}")
    return 0


def _validate_evidence(failures: list[str], fixture_id: str, value: Any, field: str) -> None:
    if not isinstance(value, dict):
        failures.append(f"{fixture_id}: {field} must be an object")
        return
    if not value.get("id"):
        failures.append(f"{fixture_id}: {field}.id is required")
    status = value.get("status")
    if status not in VALID_EVIDENCE_STATUS:
        failures.append(f"{fixture_id}: {field}.status is invalid")


def _validate_feature_policies(
    failures: list[str],
    fixture_id: str,
    raw_features: Any,
    crates: list[Any],
) -> None:
    required_pins = {
        str(crate): EXPECTED_FEATURE_POLICIES[str(crate)]
        for crate in crates
        if str(crate) in EXPECTED_FEATURE_POLICIES
    }
    if not required_pins:
        return
    if not isinstance(raw_features, dict):
        failures.append(f"{fixture_id}: missing features block for feature-sensitive crates")
        return
    for crate, expected in sorted(required_pins.items()):
        actual = raw_features.get(crate)
        if actual != expected:
            failures.append(
                f"{fixture_id}: feature policy for {crate} must be {expected!r}, got {actual!r}"
            )


def _validate_fixture_files(
    failures: list[str],
    fixture: dict[str, Any],
    crates: list[Any],
    profiles: dict[str, dict[str, Any]],
    used_evidence_tests: dict[tuple[str, str], str],
) -> tuple[int, int]:
    fixture_id = str(fixture.get("id"))
    fixture_dir = FIXTURES_ROOT / fixture_id
    manifest_path = fixture_dir / "fixture.json"
    if not manifest_path.is_file():
        return 0, 0
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        failures.append(f"{fixture_id}: fixture.json is not valid JSON: {error}")
        return 0, 0
    if not isinstance(manifest, dict):
        failures.append(f"{fixture_id}: fixture.json must be an object")
        return 0, 0

    _validate_manifest_alignment(failures, fixture_id, fixture, manifest)
    if manifest.get("features", {}) != fixture.get("features", {}):
        failures.append(f"{fixture_id}: fixture.json features must match fixture matrix")
    if manifest.get("schema_version") != 2:
        failures.append(f"{fixture_id}: fixture.json schema_version must be 2")
    if manifest.get("diagnostic_family") not in REQUIRED_DIAGNOSTICS:
        failures.append(f"{fixture_id}: fixture.json diagnostic_family must be a reserved SIFR-RUST code")

    evidence = manifest.get("evidence")
    if not isinstance(evidence, dict):
        failures.append(f"{fixture_id}: fixture.json evidence must be an object")
        return 0, 0
    _validate_diagnostic_family_alignment(failures, fixture_id, manifest, evidence)
    _validate_fixture_evidence_file(
        failures,
        fixture_id,
        fixture_dir,
        evidence.get("positive"),
        fixture.get("positive_evidence"),
        fixture.get("execution_kind"),
        "positive",
    )
    validate_evidence_provenance(
        failures,
        repo_root=REPO_ROOT,
        profiles=profiles,
        fixture_id=fixture_id,
        side="positive",
        evidence=evidence.get("positive"),
        execution_kind=str(fixture.get("execution_kind")),
        used_tests=used_evidence_tests,
    )
    _validate_fixture_evidence_file(
        failures,
        fixture_id,
        fixture_dir,
        evidence.get("negative"),
        fixture.get("negative_evidence"),
        fixture.get("execution_kind"),
        "negative",
    )
    validate_evidence_provenance(
        failures,
        repo_root=REPO_ROOT,
        profiles=profiles,
        fixture_id=fixture_id,
        side="negative",
        evidence=evidence.get("negative"),
        execution_kind=str(fixture.get("execution_kind")),
        used_tests=used_evidence_tests,
    )
    package_count = _validate_package_examples(
        failures,
        fixture_id,
        fixture_dir,
        manifest.get("package_examples"),
        crates,
        fixture.get("execution_kind"),
    )
    scenario_count = validate_scenario_examples(
        failures,
        fixture_id,
        fixture_dir,
        manifest.get("scenario_examples"),
    )
    return package_count, scenario_count


def _validate_manifest_alignment(
    failures: list[str],
    fixture_id: str,
    fixture: dict[str, Any],
    manifest: dict[str, Any],
) -> None:
    fields = (
        "id",
        "capability",
        "tier",
        "execution_kind",
        "required_crates",
        "diagnostic_crate_rationale",
    )
    for field in fields:
        if manifest.get(field) != fixture.get(field):
            failures.append(f"{fixture_id}: fixture.json {field} must match fixture matrix")


def _validate_diagnostic_family_alignment(
    failures: list[str],
    fixture_id: str,
    manifest: dict[str, Any],
    evidence: dict[str, Any],
) -> None:
    negative = evidence.get("negative")
    if not isinstance(negative, dict):
        return
    if negative.get("expected_result") not in {"diagnostic", "future-owned-diagnostic"}:
        return
    expected_diagnostic = negative.get("expected_diagnostic")
    if manifest.get("diagnostic_family") != expected_diagnostic:
        failures.append(
            f"{fixture_id}: fixture.json diagnostic_family must match the negative "
            f"expected_diagnostic {expected_diagnostic}"
        )


def _validate_package_examples(
    failures: list[str],
    fixture_id: str,
    fixture_dir: Path,
    raw_examples: Any,
    crates: list[Any],
    execution_kind: Any,
) -> int:
    expected_crates = {str(crate) for crate in crates}
    if not expected_crates:
        if raw_examples not in ({}, None):
            failures.append(f"{fixture_id}: package_examples must be empty when required_crates is empty")
        return 0
    if not isinstance(raw_examples, dict):
        failures.append(f"{fixture_id}: fixture.json package_examples must cover every required crate")
        return 0

    actual_crates = {str(crate) for crate in raw_examples}
    for crate in sorted(expected_crates - actual_crates):
        failures.append(f"{fixture_id}: missing package example for crate {crate}")
    for crate in sorted(actual_crates - expected_crates):
        failures.append(f"{fixture_id}: unexpected package example for crate {crate}")

    valid_examples = 0
    for crate in sorted(expected_crates & actual_crates):
        raw_path = raw_examples.get(crate)
        if not isinstance(raw_path, str) or not raw_path:
            failures.append(f"{fixture_id}: package_examples.{crate} path is required")
            continue
        raw_source_path = Path(raw_path)
        expected_path = Path("examples") / f"{crate}.sifr"
        if raw_source_path.is_absolute() or ".." in raw_source_path.parts:
            failures.append(f"{fixture_id}: package_examples.{crate} must stay inside the fixture directory")
            continue
        if raw_source_path != expected_path:
            failures.append(f"{fixture_id}: package_examples.{crate} must be {expected_path.as_posix()}")
            continue

        source_path = fixture_dir / raw_source_path
        if not source_path.is_file():
            failures.append(f"{fixture_id}: missing package example source {raw_path}")
            continue
        text = source_path.read_text(encoding="utf-8")
        _validate_package_example_text(failures, fixture_id, raw_path, text, crate, execution_kind)
        valid_examples += 1
    return valid_examples


def _validate_package_example_text(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    text: str,
    crate: str,
    execution_kind: Any,
) -> None:
    if len(text.strip().splitlines()) < 10:
        failures.append(f"{fixture_id}: {raw_path} must contain a full package example")
    required_headers = (
        f"# fixture: {fixture_id}",
        f"# package-example: {crate}",
        f"# required-crate: {crate}",
        f"# execution-kind: {execution_kind}",
        "# expected-result: package-example",
    )
    for header in required_headers:
        if header not in text:
            failures.append(f"{fixture_id}: {raw_path} missing header {header!r}")
    if _contains_empty_pass_body(text):
        failures.append(f"{fixture_id}: {raw_path} must not use empty placeholder class bodies")
    crate_token = crate.replace("-", "_")
    binding_token = package_example_binding_token(fixture_id, crate_token)
    bound_functions = _rust_bound_function_names(text, binding_token)
    if not bound_functions:
        failures.append(
            f"{fixture_id}: {raw_path} must declare a Rust binding for "
            f"{binding_token}"
        )
        return
    if fixture_id == "proc_macro_trust":
        marker = {
            "serde_derive": (
                "serde_derive=1.0.228;"
                "upstream=compiled;sifr_wrapper_macro=executed"
            ),
            "prost-build": "prost-build=0.14.4;message=sifr.probe.Probe",
        }.get(crate)
        if marker is None or marker not in text:
            failures.append(
                f"{fixture_id}: {raw_path} must observe the generated {crate} marker"
            )

    verifier_marker = f"def verify_{crate_token}_package("
    async_verifier_marker = f"async def verify_{crate_token}_package("
    if verifier_marker not in text and async_verifier_marker not in text:
        failures.append(f"{fixture_id}: {raw_path} must include verify_{crate_token}_package")
        return
    verifier_start = min(
        index for index in (text.find(verifier_marker), text.find(async_verifier_marker)) if index >= 0
    )
    verifier_body = text[verifier_start:]
    for bound_function in bound_functions:
        if f"{bound_function}(" not in verifier_body:
            failures.append(f"{fixture_id}: {raw_path} verifier must call {bound_function}")
        if not _verifier_binds_call(verifier_body, bound_function):
            failures.append(f"{fixture_id}: {raw_path} verifier must bind {bound_function} result before returning")


def _rust_bound_function_names(text: str, crate_token: str) -> list[str]:
    names: list[str] = []
    lines = text.splitlines()
    for index, line in enumerate(lines):
        stripped = line.lstrip()
        binding_prefix = f"@rust({crate_token}"
        if not stripped.startswith(binding_prefix):
            continue
        if not _has_crate_token_boundary(stripped, len(binding_prefix)):
            continue
        for following in lines[index + 1 :]:
            following_stripped = following.lstrip()
            if following_stripped.startswith("@"):
                continue
            name = _decorated_function_name(following_stripped)
            if name is not None and name not in names:
                names.append(name)
            break
    return names


def _has_crate_token_boundary(text: str, index: int) -> bool:
    if index >= len(text):
        return True
    next_char = text[index]
    return not (next_char.isalnum() or next_char == "_")


def _validate_fixture_evidence_file(
    failures: list[str],
    fixture_id: str,
    fixture_dir: Path,
    manifest_evidence: Any,
    matrix_evidence: Any,
    execution_kind: Any,
    side: str,
) -> None:
    if not isinstance(manifest_evidence, dict):
        failures.append(f"{fixture_id}: fixture.json evidence.{side} must be an object")
        return
    if not isinstance(matrix_evidence, dict):
        return
    for field in ("id", "status"):
        if manifest_evidence.get(field) != matrix_evidence.get(field):
            failures.append(f"{fixture_id}: fixture.json evidence.{side}.{field} must match fixture matrix")

    raw_path = manifest_evidence.get("path")
    if not isinstance(raw_path, str) or not raw_path:
        failures.append(f"{fixture_id}: fixture.json evidence.{side}.path is required")
        return
    raw_source_path = Path(raw_path)
    if raw_source_path.is_absolute() or ".." in raw_source_path.parts:
        failures.append(f"{fixture_id}: evidence.{side}.path must stay inside the fixture directory")
        return
    expected_path = Path(side) / f"{matrix_evidence.get('id')}.sifr"
    if raw_source_path != expected_path:
        failures.append(f"{fixture_id}: evidence.{side}.path must be {expected_path.as_posix()}")
        return
    source_path = fixture_dir / raw_path
    try:
        source_path.relative_to(fixture_dir)
    except ValueError:
        failures.append(f"{fixture_id}: evidence.{side}.path must stay inside the fixture directory")
        return
    if source_path.suffix != ".sifr":
        failures.append(f"{fixture_id}: evidence.{side}.path must point to a .sifr file")
    if not source_path.is_file():
        failures.append(f"{fixture_id}: missing evidence source {raw_path}")
        return

    text = source_path.read_text(encoding="utf-8")
    if len(text.strip().splitlines()) < 5:
        failures.append(f"{fixture_id}: {raw_path} must contain a concrete fixture, not an empty stub")
    required_headers = (
        f"# fixture: {fixture_id}",
        f"# evidence: {side}/{matrix_evidence.get('id')}",
        f"# evidence-status: {matrix_evidence.get('status')}",
    )
    for header in required_headers:
        if header not in text:
            failures.append(f"{fixture_id}: {raw_path} missing header {header!r}")
    if not any(line.lstrip().startswith("@rust") for line in text.splitlines()):
        failures.append(f"{fixture_id}: {raw_path} must exercise a Rust interop declaration")
    _validate_evidence_example_text(failures, fixture_id, raw_path, text, str(matrix_evidence.get("id")))

    if f"# execution-kind: {execution_kind}" not in text:
        failures.append(f"{fixture_id}: {raw_path} missing execution-kind header")
    validate_evidence_expectation(
        failures,
        fixture_id=fixture_id,
        side=side,
        raw_path=raw_path,
        text=text,
        evidence=manifest_evidence,
        status=matrix_evidence.get("status"),
        execution_kind=execution_kind,
        required_diagnostics=REQUIRED_DIAGNOSTICS,
    )


def _validate_evidence_example_text(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    text: str,
    evidence_id: str,
) -> None:
    if len(text.strip().splitlines()) < 9:
        failures.append(f"{fixture_id}: {raw_path} must include a binding and concrete verifier call site")
    if _contains_empty_pass_body(text):
        failures.append(f"{fixture_id}: {raw_path} must not use empty placeholder class bodies")
    if (
        fixture_id == "proc_macro_trust"
        and raw_path == "negative/untrusted_proc_macro_rejected_pre_execution.sifr"
        and (
            "decode_without_proc_macro_trust(input: bytes) -> str" not in text
            or "decode_without_proc_macro_trust_result: str" not in text
        )
    ):
        failures.append(
            f"{fixture_id}: {raw_path} binding must match the scenario bridge `str` return"
        )

    verifier_markers = (
        f"def verify_{evidence_id}(",
        f"async def verify_{evidence_id}(",
    )
    verifier_start = min((text.find(marker) for marker in verifier_markers if marker in text), default=-1)
    if verifier_start < 0:
        failures.append(f"{fixture_id}: {raw_path} must include verify_{evidence_id}")
        return

    verifier_body = text[verifier_start:]
    bound_declarations = _rust_bound_declarations(text)
    if not bound_declarations:
        failures.append(f"{fixture_id}: {raw_path} must include a Rust-decorated binding declaration")
    for name, return_type in bound_declarations:
        if f"{name}(" not in verifier_body and f".{name}(" not in verifier_body:
            failures.append(f"{fixture_id}: {raw_path} verifier must call {name}")
        if return_type != "None" and not _verifier_binds_call(verifier_body, name):
            failures.append(f"{fixture_id}: {raw_path} verifier must bind {name} result before returning")


if __name__ == "__main__":
    raise SystemExit(main())
