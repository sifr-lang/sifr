"""Fixture isolation must not force independent copies of the pinned environment."""
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import tomllib
from example_environment import example_environment_root
from example_packages import ExampleCase, prepare_example_package


def run_example_environment_self_tests():
    with TemporaryDirectory(prefix="sifr-example-environment-") as directory:
        root = Path(directory)
        area = root / "area"
        area.mkdir()
        (area / ".venv").mkdir()
        for name in ("pyproject.toml", "uv.lock"):
            (area / name).write_text("pinned")
        fixtures = root / "fixtures"
        fixtures.mkdir()
        (fixtures / "main.sifr").write_text("def main():\n    assert True\n")
        bridges = fixtures / "python_bridges"
        bridges.mkdir()
        (bridges / "bridge.py").write_text("VALUE = 42\n")
        paths = SimpleNamespace(repo_root=root, area_root=area, fixtures_root=fixtures)
        case = ExampleCase("first", "main.sifr", "marker", ("numpy",), ())
        first = prepare_example_package(paths, "one", case)
        second = prepare_example_package(paths, "two", case)
        assert first != second
        for package in (first, second):
            assert (package / "src/main.sifr").read_bytes() == (fixtures / "main.sifr").read_bytes()
            assert (package / "src/python_bridges/bridge.py").read_bytes() == (bridges / "bridge.py").read_bytes()
            manifest = tomllib.loads((package / "sifr.toml").read_text())
            assert manifest["python"] == {"requires-imports": ["numpy"]}
            assert manifest["trust"] == {"python": ["numpy"], "python-native": []}
            assert not any((package / name).exists() for name in (".venv", "pyproject.toml", "uv.lock"))
        shared = example_environment_root(paths)
        assert first.parent.parent == shared == second.parent.parent
        identities = {name: (shared / name).lstat().st_mtime_ns for name in (".venv", "pyproject.toml", "uv.lock")}
        prepare_example_package(paths, "one", case)
        assert identities == {name: (shared / name).lstat().st_mtime_ns for name in identities}
        for name in identities:
            link = shared / name
            link.unlink()
            for wrong in (area / "missing", area / "pyproject.toml" if name != "pyproject.toml" else area / "uv.lock"):
                link.symlink_to(wrong)
                try:
                    example_environment_root(paths)
                except RuntimeError as error:
                    assert "Conflicting" in str(error)
                else:
                    raise AssertionError("conflicting alias accepted")
                assert link.readlink() == wrong
                link.unlink()
            link.write_text("foreign")
            try:
                example_environment_root(paths)
            except RuntimeError as error:
                assert "Conflicting" in str(error)
            else:
                raise AssertionError("foreign input accepted")
            assert link.read_text() == "foreign"
            link.unlink()
            link.symlink_to(area / name)
        (area / "uv.lock").unlink()
        try:
            example_environment_root(paths)
        except RuntimeError as error:
            assert "pinned area input" in str(error)
        else:
            raise AssertionError("missing locked input accepted")
