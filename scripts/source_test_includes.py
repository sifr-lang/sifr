"""Find literal Rust includes owned exclusively by already-classified test files.

This extends the direction guard's existing test-source convention to split
includes. Unknown include expressions stay scanned. Production includes and
module references take precedence over test ownership.
"""
from __future__ import annotations

from collections.abc import Callable
from pathlib import Path
import re

INCLUDE = re.compile(r'(?m)^\s*include!\s*\(\s*"([^"\n]+)"\s*\)\s*;')
MODULE = re.compile(r'(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z_0-9]*)\s*;')
PATH_ATTRIBUTE = re.compile(r'#\[path\s*=\s*"([^"\n]+)"\]')


def exclusively_test_includes(files: set[Path], is_test: Callable[[Path], bool]) -> set[Path]:
    canonical = {path.resolve(): path for path in files}
    includes = {}
    modules = {}
    for path in files:
        text = path.read_text(encoding="utf-8", errors="replace")
        includes[path] = {canonical[target] for value in INCLUDE.findall(text)
                          if (target := (path.parent / value).resolve()) in canonical}
        module_root = path.parent if path.name in {"lib.rs", "main.rs", "mod.rs"} else path.with_suffix("")
        targets = [(path.parent / value).resolve() for value in PATH_ATTRIBUTE.findall(text)]
        for name in MODULE.findall(text):
            targets.extend(((module_root / f"{name}.rs").resolve(),
                            (module_root / name / "mod.rs").resolve()))
        modules[path] = {canonical[target] for target in targets if target in canonical}

    def reachable(roots, graph):
        seen = set(roots)
        pending = list(roots)
        while pending:
            for child in graph.get(pending.pop(), ()):
                if child not in seen:
                    seen.add(child)
                    pending.append(child)
        return seen

    test_roots = {path for path in files if is_test(path)}
    candidates = reachable(test_roots, includes) - test_roots
    # The guard already classifies test roots; following their cfg(test) module
    # declarations from lib.rs must not reclassify their fragments as production.
    production_graph = {path: (includes[path] | modules[path]) - test_roots for path in files}
    production = reachable(files - test_roots - candidates, production_graph)
    return candidates - production
