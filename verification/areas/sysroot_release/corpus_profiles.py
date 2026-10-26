"""Use the existing normal libtest as the development half of corpus verification."""
from __future__ import annotations
import json
from pathlib import Path
import subprocess


def prepare_development(root: Path, env: dict[str, str], run=subprocess.run) -> dict[str, str]:
    # Reuse the normal driver libtest graph already required by crate testing.
    result = run(["cargo", "test", "--locked", "--offline", "-p", "sifr_driver",
                  "--lib", "--no-run", "--message-format=json-render-diagnostics"],
                 cwd=root, env=env, check=True, stdout=subprocess.PIPE, text=True)
    artifacts = set()
    for line in result.stdout.splitlines():
        message = json.loads(line)
        target = message.get("target", {})
        if (message.get("reason") == "compiler-artifact"
                and target.get("name") == "sifr_driver"
                and "lib" in target.get("kind", [])
                and message.get("profile", {}).get("test") is True
                and message.get("executable")):
            artifacts.add(str(Path(message["executable"]).resolve()))
    if len(artifacts) != 1:
        raise RuntimeError("Cargo did not report exactly one development driver libtest")
    return {"driver_test": artifacts.pop()}


def validate_development(result) -> None:
    from sifr_verify.corpus_ownership import CORPUS_CASE
    from sifr_verify.native_test_roles import require_one_executed_case
    require_one_executed_case(result, CORPUS_CASE)
    if f"test {CORPUS_CASE} ... ok".encode() not in result.stdout.splitlines():
        raise RuntimeError("development libtest did not execute the declared corpus assertion")


def compare_profiles(root: Path, development: Path, release: Path) -> None:
    fixtures = root / "crates/sifr/tests/e2e/pass"
    names = sorted(p.stem for p in fixtures.glob("*.sifr"))
    assert len(names) > 700, "maintained corpus unexpectedly incomplete"
    reports = [json.loads((path / "coverage.json").read_text()) for path in (development, release)]
    for report in reports:
        assert report["selection"] is None and not report["failures"]
        assert [row["fixture"] for row in report["rows"]] == names, "incomplete/duplicate corpus"
    for left, right in zip(reports[0]["rows"], reports[1]["rows"], strict=True):
        name = left["fixture"]
        rows = []
        for row in (left, right):
            modules = row["stdlib_modules"]
            assert len(modules) == len(set(modules)), (name, "duplicate module")
            # GeneratedRust stores a HashSet; its JSON iteration order is not semantic.
            rows.append(dict(row, stdlib_modules=sorted(modules)))
        assert rows[0] == rows[1], (name, "compiler-profile corpus fields differ")
        assert (development / (name + ".source.rs")).read_bytes() == (
            release / (name + ".source.rs")).read_bytes(), (name, "compiler-profile Rust differs")
        expected = (fixtures / (name + ".sifr")).read_bytes()
        assert (development / (name + ".sifr")).read_bytes() == expected
        assert (release / (name + ".sifr")).read_bytes() == expected
    # Portable metadata includes versioned inputs. Each oracle verifies its own
    # source/metadata equivalence; installed qualification checks the release payload.
