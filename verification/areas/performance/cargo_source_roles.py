"""Cargo entrypoint roles for the source admission guard, without building Cargo.

Only real test targets seed test ownership. Module/include edges determine their
descendants; a production target at the same path always retains production role.
"""
from pathlib import Path
import tomllib


def source_roles(paths: list[Path], repo_root: Path) -> tuple[set[Path], set[Path]]:
    root = repo_root.resolve()
    manifests = set()
    for path in paths:
        for parent in path.resolve().parents:
            if not parent.is_relative_to(root):
                break
            manifest = parent / "Cargo.toml"
            if manifest.is_file():
                manifests.add(manifest)
                break
    tests, production = set(), set()
    for manifest in sorted(manifests):
        data = tomllib.loads(manifest.read_text())
        package = data.get("package")
        if not isinstance(package, dict):
            continue
        base = manifest.parent
        if package.get("autotests", True):
            tests.update((base / "tests").glob("*.rs"))
            tests.update((base / "tests").glob("*/main.rs"))
        for target in data.get("test", []):
            if "path" in target:
                tests.add(base / target["path"])
            else:
                name = target["name"]
                tests.update((base / "tests" / f"{name}.rs", base / "tests" / name / "main.rs"))

        if "lib" in data or package.get("autolib", True):
            production.add(base / data.get("lib", {}).get("path", "src/lib.rs"))
        if package.get("build", True) is not False:
            build = package.get("build", "build.rs")
            production.add(base / (build if isinstance(build, str) else "build.rs"))
        for kind, folder, automatic in (("bin", "src/bin", "autobins"),
                                         ("example", "examples", "autoexamples"),
                                         ("bench", "benches", "autobenches")):
            if package.get(automatic, True):
                production.update((base / folder).glob("*.rs"))
                production.update((base / folder).glob("*/main.rs"))
                if kind == "bin":
                    production.add(base / "src/main.rs")
            for target in data.get(kind, []):
                if "path" in target:
                    production.add(base / target["path"])
                else:
                    name = target["name"]
                    production.update((base / folder / f"{name}.rs", base / folder / name / "main.rs"))
                    if kind == "bin" and name == package.get("name"):
                        production.add(base / "src/main.rs")
    known = {path.resolve() for path in paths}
    return ({path.resolve() for path in tests} & known,
            {path.resolve() for path in production} & known)
