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
PRODUCTION_INCLUDE = re.compile(r'\binclude!\s*\(\s*"([^"\n]+)"\s*\)\s*;')
MODULE = re.compile(r'\bmod\s+([A-Za-z_][A-Za-z_0-9]*)\s*;')
PATH_ATTRIBUTE = re.compile(r'#\[path\s*=\s*"([^"\n]+)"\]')


def exclusively_test_includes(files: set[Path], is_test: Callable[[Path], bool]) -> set[Path]:
    canonical = {path.resolve(): path for path in files}
    by_module_name = {}
    for path in files:
        name = path.parent.name if path.name == "mod.rs" else path.stem
        by_module_name.setdefault(name, set()).add(path.resolve())
    includes = {}
    modules = {}
    production_includes = {}
    unknown_includes = set()
    for path in files:
        text = path.read_text(encoding="utf-8", errors="replace")
        if re.search(r"\binclude\s*!", PRODUCTION_INCLUDE.sub("", text)):
            unknown_includes.add(path)
        includes[path] = {canonical[target] for value in INCLUDE.findall(text)
                          if (target := (path.parent / value).resolve()) in canonical}
        production_includes[path] = {canonical[target] for value in PRODUCTION_INCLUDE.findall(text)
                                     if (target := (path.parent / value).resolve()) in canonical}
        targets = [(path.parent / value).resolve() for value in PATH_ATTRIBUTE.findall(text)]
        for name in MODULE.findall(text):
            # Conservatively retain every matching module filename. This also
            # covers inline module nesting without implementing a Rust parser.
            targets.extend(by_module_name.get(name, ()))
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
    production_graph = {path: (production_includes[path] | modules[path]) - test_roots for path in files}
    production = reachable(files - test_roots - candidates, production_graph)
    # An unresolved production include could reach any fragment. Fail closed.
    return set() if production & unknown_includes else candidates - production
