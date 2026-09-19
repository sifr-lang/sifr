"""Preparation must retain package authority and stop before dependent work."""
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import prepare_examples


def run_preparation_self_tests():
    paths = SimpleNamespace(area_root=Path("/owned/area"))
    package = Path("/owned/examples/case")
    case = SimpleNamespace(case_id="case")
    module = SimpleNamespace(LIBRARY_EXAMPLE_CASES={"case": case})
    certification = ["python", "arrow", "certify", "--check"]
    calls = []

    def invoke(actual_paths, actual_package, arguments):
        assert actual_paths is paths and actual_package == package
        calls.append(arguments)
        return {"exit_code": 0}

    with (
        patch.object(prepare_examples, "require_canonical_python"),
        patch.object(prepare_examples.importlib, "import_module", return_value=module),
        patch.object(prepare_examples, "prepare_example_package", return_value=package),
        patch.object(prepare_examples, "certification_commands_for", return_value=[certification]),
        patch.object(prepare_examples, "package_snapshot", return_value={"source": b"unchanged"}),
        patch.object(prepare_examples, "_run_sifr_command", side_effect=invoke),
        redirect_stdout(StringIO()),
    ):
        prepare_examples.prepare(["libraries"], paths)
        assert calls == [certification, [
            "build", "src/main.sifr", "--release", "--output",
            "/owned/examples/case-prepared-native",
        ]]
        calls.clear()
        with patch.object(prepare_examples, "package_snapshot", side_effect=[{}, {"changed": b""}]):
            try:
                prepare_examples.prepare(["libraries"], paths)
            except RuntimeError as error:
                assert "certification recheck mutated" in str(error)
            else:
                raise AssertionError("mutated certification was accepted")
        assert calls == [certification], "build ran after certification failure"
