#!/usr/bin/env python3
"""Validate deterministic sysroot stdlib bootstrap ordering."""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SOURCES_RS = REPO_ROOT / "crates" / "sifr_stdlib_manifest" / "src" / "sources.rs"
PUBLIC_ROOT = REPO_ROOT / "stdlib" / "sifr"
PRIVATE_ROOT = REPO_ROOT / "stdlib" / "_sifr"

PRIVATE_LIST_RE = re.compile(
    r"PRIVATE_STDLIB_MODULES:\s*&\[&str\]\s*=\s*&\[(.*?)\];",
    re.DOTALL,
)
PUBLIC_MODULE_RE = re.compile(r'module:\s*"((?:sifr)\.[A-Za-z0-9_]+)"')
STRING_RE = re.compile(r'"([^"]+)"')
def main() -> int:
    sources_text = SOURCES_RS.read_text(encoding="utf-8")
    private_modules = _private_modules(sources_text)
    public_modules = _public_modules(sources_text)
    public_sources = {
        module: _module_path(PUBLIC_ROOT, module, "sifr").read_text(encoding="utf-8")
        for module in public_modules
    }
    private_sources = {
        module: _module_path(PRIVATE_ROOT, module, "_sifr").read_text(encoding="utf-8")
        for module in private_modules
    }
    failures = _validate(private_modules, public_modules, private_sources, public_sources)
    if failures:
        print("stdlib bootstrap ordering: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    public_set = set(public_modules)
    edge_count = sum(
        len(_public_imports(source) & public_set) for source in public_sources.values()
    )
    print(
        "stdlib bootstrap ordering: PASS "
        f"(private={len(private_modules)}, public={len(public_modules)}, public_edges={edge_count})"
    )
    return 0


def _private_modules(sources_text: str) -> list[str]:
    match = PRIVATE_LIST_RE.search(sources_text)
    if match is None:
        return []
    return STRING_RE.findall(match.group(1))


def _public_modules(sources_text: str) -> list[str]:
    return PUBLIC_MODULE_RE.findall(sources_text)


def _validate(
    private_modules: list[str],
    public_modules: list[str],
    private_sources: dict[str, str],
    public_sources: dict[str, str],
) -> list[str]:
    failures: list[str] = []
    _validate_unique(failures, private_modules, "private stdlib module")
    _validate_unique(failures, public_modules, "public stdlib module")

    if private_modules != sorted(private_modules):
        failures.append("private stdlib modules must be sorted lexicographically")

    private_set = set(private_modules)
    for module, source in private_sources.items():
        imported_private = sorted(_private_imports(source, private_set))
        if imported_private:
            failures.append(
                f"{module}: private declaration source imports private declarations: "
                + ", ".join(imported_private)
            )
        imported_public = sorted(_public_imports(source))
        if imported_public:
            failures.append(
                f"{module}: private declaration source imports public stdlib modules: "
                + ", ".join(imported_public)
            )

    public_set = set(public_modules)
    public_index = {module: index for index, module in enumerate(public_modules)}
    graph: dict[str, set[str]] = {}
    for module in public_modules:
        deps = _public_imports(public_sources.get(module, ""))
        unknown_deps = sorted(deps - public_set)
        for dep in unknown_deps:
            failures.append(f"{module}: imports unknown public stdlib module {dep}")
        graph[module] = deps & public_set
        for dep in sorted(graph[module]):
            if public_index[dep] > public_index[module]:
                failures.append(
                    f"{module}: imports {dep}, but {dep} appears later in STDLIB_SOURCES"
                )

    cycle = _first_cycle(graph)
    if cycle:
        failures.append("public stdlib import cycle: " + " -> ".join(cycle))

    return failures


def _validate_unique(failures: list[str], modules: list[str], label: str) -> None:
    seen: set[str] = set()
    for module in modules:
        if module in seen:
            failures.append(f"duplicate {label}: {module}")
        seen.add(module)


def _public_imports(source: str) -> set[str]:
    return _imports_with_prefix(source, "sifr.")


def _private_imports(source: str, private_modules: set[str]) -> set[str]:
    return {
        module
        for module in _imports_with_prefix(source, "_sifr.")
        if module in private_modules
    }


def _imports_with_prefix(source: str, prefix: str) -> set[str]:
    imports: set[str] = set()
    for raw_line in source.splitlines():
        line = raw_line.split("#", 1)[0].strip()
        if line.startswith("from "):
            module = line.removeprefix("from ").split(" import ", 1)[0].strip()
            if module.startswith(prefix):
                imports.add(module)
            continue
        if line.startswith("import "):
            for item in line.removeprefix("import ").split(","):
                module = item.strip().split()[0] if item.strip() else ""
                if module.startswith(prefix):
                    imports.add(module)
    return imports


def _first_cycle(graph: dict[str, set[str]]) -> list[str]:
    visiting: set[str] = set()
    visited: set[str] = set()
    stack: list[str] = []

    def visit(module: str) -> list[str]:
        if module in visited:
            return []
        if module in visiting:
            start = stack.index(module)
            return stack[start:] + [module]
        visiting.add(module)
        stack.append(module)
        for dep in sorted(graph.get(module, ())):
            cycle = visit(dep)
            if cycle:
                return cycle
        stack.pop()
        visiting.remove(module)
        visited.add(module)
        return []

    for module in sorted(graph):
        cycle = visit(module)
        if cycle:
            return cycle
    return []


def _module_path(root: Path, module: str, prefix: str) -> Path:
    tail = module.removeprefix(prefix).removeprefix(".")
    return root / f"{tail}.sifr"


def _self_test() -> int:
    private = ["_sifr.a", "_sifr.b"]
    public = ["sifr.a", "sifr.b"]
    private_sources = {"_sifr.a": "", "_sifr.b": ""}
    public_sources = {"sifr.a": "", "sifr.b": "from sifr.a import value\n"}
    if _validate(private, public, private_sources, public_sources):
        print("self-test seed should pass", file=sys.stderr)
        return 1

    forward = {"sifr.a": "from sifr.b import value\n", "sifr.b": ""}
    if not any(
        "appears later" in failure
        for failure in _validate(private, public, private_sources, forward)
    ):
        print("self-test public forward import was not rejected", file=sys.stderr)
        return 1

    comma_import = {"sifr.a": "import sifr.b, sifr.a\n", "sifr.b": ""}
    if not any(
        "appears later" in failure
        for failure in _validate(private, public, private_sources, comma_import)
    ):
        print("self-test comma public import was not rejected", file=sys.stderr)
        return 1

    unknown = {"sifr.a": "from sifr.missing import value\n", "sifr.b": ""}
    if not any(
        "unknown public stdlib module" in failure
        for failure in _validate(private, public, private_sources, unknown)
    ):
        print("self-test unknown public import was not rejected", file=sys.stderr)
        return 1

    cycle = {
        "sifr.a": "from sifr.b import value\n",
        "sifr.b": "from sifr.a import value\n",
    }
    if not any(
        "import cycle" in failure
        for failure in _validate(private, public, private_sources, cycle)
    ):
        print("self-test public import cycle was not rejected", file=sys.stderr)
        return 1

    private_bad = {"_sifr.a": "from _sifr.b import value\n", "_sifr.b": ""}
    if not any(
        "imports private declarations" in failure
        for failure in _validate(private, public, private_bad, public_sources)
    ):
        print("self-test private import was not rejected", file=sys.stderr)
        return 1

    private_public_bad = {"_sifr.a": "from sifr.a import value\n", "_sifr.b": ""}
    if not any(
        "imports public stdlib modules" in failure
        for failure in _validate(private, public, private_public_bad, public_sources)
    ):
        print("self-test private public import was not rejected", file=sys.stderr)
        return 1

    unsorted_private = ["_sifr.b", "_sifr.a"]
    if not any(
        "sorted lexicographically" in failure
        for failure in _validate(
            unsorted_private,
            public,
            private_sources,
            public_sources,
        )
    ):
        print("self-test private sort order was not rejected", file=sys.stderr)
        return 1

    duplicate_public = ["sifr.a", "sifr.a"]
    if not any(
        "duplicate public stdlib module: sifr.a" in failure
        for failure in _validate(private, duplicate_public, private_sources, public_sources)
    ):
        print("self-test duplicate public module was not rejected", file=sys.stderr)
        return 1

    print("stdlib bootstrap ordering self-test: PASS")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        raise SystemExit(_self_test())
    raise SystemExit(main())
