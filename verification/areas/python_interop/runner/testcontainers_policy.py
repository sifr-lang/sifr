"""Reject retired Testcontainers APIs in every runner module's executable code."""

from __future__ import annotations

import ast
from pathlib import Path
from tempfile import TemporaryDirectory

RETIRED_MODULES = (
    "testcontainers.kafka",
    "testcontainers.localstack",
    "testcontainers.postgres",
    "testcontainers.redis",
    "testcontainers.core.waiting_utils",
)
RETIRED_WAIT_HELPERS = ("wait_for_logs", "wait_container_is_ready")


def validate_testcontainers_source(source: str, path: Path) -> None:
    module = ast.parse(source, filename=str(path))
    aliases: dict[str, str] = {}

    def reject_retired(name: str, line: int) -> None:
        if any(name == old or name.startswith(old + ".") for old in RETIRED_MODULES):
            raise SystemExit(f"{path}:{line}: retired Testcontainers API: {name}")

    for node in ast.walk(module):
        if isinstance(node, ast.Import):
            for alias in node.names:
                reject_retired(alias.name, node.lineno)
                aliases[alias.asname or alias.name.split(".")[0]] = (
                    alias.name if alias.asname else alias.name.split(".")[0]
                )
        elif isinstance(node, ast.ImportFrom) and node.level == 0:
            for alias in node.names:
                name = f"{node.module}.{alias.name}"
                reject_retired(name, node.lineno)
                aliases[alias.asname or alias.name] = name

    def qualified_name(node: ast.AST) -> str:
        if isinstance(node, ast.Name):
            return aliases.get(node.id, node.id)
        if isinstance(node, ast.Attribute):
            return f"{qualified_name(node.value)}.{node.attr}"
        return ""

    for node in ast.walk(module):
        if isinstance(node, (ast.Name, ast.Attribute)):
            reject_retired(qualified_name(node), node.lineno)
            name = node.id if isinstance(node, ast.Name) else node.attr
            if name in RETIRED_WAIT_HELPERS:
                raise SystemExit(
                    f"{path}:{node.lineno}: retired Testcontainers wait helper: {name}"
                )


def runner_modules(area_root: Path) -> list[Path]:
    return [area_root / "runner.py", *sorted((area_root / "runner").rglob("*.py"))]


def validate_runner_testcontainers(area_root: Path) -> None:
    for path in runner_modules(area_root):
        validate_testcontainers_source(path.read_text(encoding="utf-8"), path)


def run_testcontainers_self_tests(area_root: Path) -> None:
    accepted = (
        "from testcontainers.community.redis import RedisContainer\n"
        "from testcontainers.core.wait_strategies import LogMessageWaitStrategy\n"
        "# from testcontainers.redis import RedisContainer\n"
        "example = 'wait_for_logs(container, pattern)'\n"
    )
    validate_testcontainers_source(accepted, Path("accepted.py"))
    mutations = []
    for module in RETIRED_MODULES:
        parent, _, child = module.rpartition(".")
        mutations.extend(
            (
                f"import {module}\n",
                f"import {module} as retired\n",
                f"from {module} import Example as renamed\n",
                f"from {parent} import (\n    {child} as retired,\n)\n",
                f"import {parent} as parent\nparent.{child}.Example()\n",
            )
        )
    for helper in RETIRED_WAIT_HELPERS:
        mutations.extend(
            (
                f"from testcontainers.core.waiting_utils import {helper} as wait\n",
                f"{helper}(container, pattern)\n",
                f"container.{helper}(pattern)\n",
            )
        )
    for index, source in enumerate(mutations):
        path = Path(f"mutation-{index}.py")
        try:
            validate_testcontainers_source(source, path)
        except SystemExit as error:
            if (
                str(path) not in str(error)
                or "retired Testcontainers" not in str(error)
            ):
                raise AssertionError(f"unexpected policy failure: {error}") from error
        else:
            raise AssertionError(f"retired Testcontainers mutation accepted: {source}")

    # Exercise actual discovery for every maintained module, plus a new nested owner.
    relative_paths = [path.relative_to(area_root) for path in runner_modules(area_root)]
    relative_paths.append(Path("runner/new_owner/nested.py"))
    with TemporaryDirectory(prefix="sifr-testcontainers-policy-") as directory:
        root = Path(directory)
        for relative in relative_paths:
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(accepted, encoding="utf-8")
        validate_runner_testcontainers(root)
        for relative in relative_paths:
            path = root / relative
            path.write_text(mutations[0], encoding="utf-8")
            try:
                validate_runner_testcontainers(root)
            except SystemExit as error:
                if str(path) not in str(error):
                    raise AssertionError(f"wrong mutation owner: {error}") from error
            else:
                raise AssertionError(f"runner module escaped policy scan: {relative}")
            finally:
                path.write_text(accepted, encoding="utf-8")
    print(
        f"Testcontainers runner policy self-test ok: forms={len(mutations)} "
        f"modules={len(relative_paths)}"
    )
