"""Use the normal CLI graph for a second, differently optimized corpus oracle."""
from __future__ import annotations
import json
from pathlib import Path
import subprocess


def prepare_development(root: Path, env: dict[str, str], run=subprocess.run) -> dict[str, str]:
    # Cargo selects the executable, including caller target/configuration overrides.
    result = run(["cargo", "build", "--locked", "--offline", "-p", "sifr",
                  "-p", "sifr_corpus_oracle", "--bins", "--message-format=json-render-diagnostics"],
                 cwd=root, env=env, check=True, stdout=subprocess.PIPE, text=True)
    artifacts = {}
    for line in result.stdout.splitlines():
        message = json.loads(line)
        target = message.get("target", {})
        name = target.get("name")
        if (message.get("reason") == "compiler-artifact" and name in {"sifr", "sifr_corpus_oracle"}
                and "bin" in target.get("kind", []) and message.get("executable")):
            path = str(Path(message["executable"]).resolve())
            if name in artifacts and artifacts[name] != path:
                raise RuntimeError(f"ambiguous development corpus artifact: {name}")
            artifacts[name] = path
    if set(artifacts) != {"sifr", "sifr_corpus_oracle"}:
        raise RuntimeError("Cargo did not report both development corpus executables")
    return artifacts


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
