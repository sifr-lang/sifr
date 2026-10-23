"""Preparation must retain package authority and stop before dependent work."""
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import prepare_examples
from example_environment_checks import run_example_environment_self_tests


def run_preparation_self_tests():
    run_example_environment_self_tests()
    run_multiple_suite_preparation_self_tests()
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


def run_multiple_suite_preparation_self_tests():
    paths = SimpleNamespace(area_root=Path("/owned/area"))
    first = SimpleNamespace(case_id="first")
    second = SimpleNamespace(case_id="second")
    modules = {
        "arrow_examples": SimpleNamespace(ARROW_EXAMPLE_CASES={"first": first}),
        "library_examples": SimpleNamespace(LIBRARY_EXAMPLE_CASES={"second": second}),
    }
    events = []

    def package(actual_paths, suite, case):
        assert actual_paths is paths
        events.append(("prepare", suite, case.case_id))
        return Path("/owned/examples") / case.case_id

    def command(actual_paths, root, arguments):
        assert actual_paths is paths
        events.append(("command", root.name, arguments))
        return {"exit_code": 0}

    certify = ["python", "certify", "--check"]
    with (
        patch.object(prepare_examples, "require_canonical_python"),
        patch.object(prepare_examples.importlib, "import_module", side_effect=modules.__getitem__),
        patch.object(prepare_examples, "prepare_example_package", side_effect=package),
        patch.object(prepare_examples, "certification_commands_for", return_value=[certify]),
        patch.object(prepare_examples, "package_snapshot", return_value={}),
        patch.object(prepare_examples, "_run_sifr_command", side_effect=command),
        redirect_stdout(StringIO()),
    ):
        prepare_examples.prepare(["arrow-examples", "libraries"], paths)
        assert events == [
            ("prepare", "arrow", "first"), ("command", "first", certify),
            ("command", "first", ["build", "src/main.sifr", "--release", "--output",
                                   "/owned/examples/first-prepared-native"]),
            ("prepare", "library", "second"), ("command", "second", certify),
            ("command", "second", ["build", "src/main.sifr", "--release", "--output",
                                    "/owned/examples/second-prepared-native"]),
        ]
        events.clear()
        with patch.object(prepare_examples, "_run_sifr_command", return_value={"exit_code": 1}):
            try:
                prepare_examples.prepare(["arrow-examples", "libraries"], paths)
            except RuntimeError:
                pass
            else:
                raise AssertionError("failed certification was accepted")
        assert events == [("prepare", "arrow", "first")], "later work ran after failure"
