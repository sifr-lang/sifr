"""Reuse existing consumers' declarations for inputs outside source ownership."""
import importlib.util
import json
import sys
from pathlib import Path

COMPATIBILITY_CHECK = "verification/areas/developer_tooling/check_no_pre_v1_compatibility.py"
TAXONOMY_CHECK = "verification/areas/coverage_matrix/checks/verification_taxonomy.py"
DOCUMENTATION_INVENTORY = "verification/areas/documentation/docs_inventory.json"
GUARDRAIL_POLICY = "verification/policy/guardrails.json"

def _load(root, relative):
    path = root / relative
    if not path.is_file():
        raise ValueError(f"declared input consumer is missing: {relative}")
    # These are trusted repository checks. Import declarations, never their main
    # routines. A unique temporary module registration supports dataclasses.
    name = "_sifr_inventory_" + str(abs(hash(str(path))))
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    prior = sys.modules.get(name)
    sys.modules[name] = module
    prior_path = sys.path[:]
    sys.path[:0] = [str(path.parent), str(root)]
    try:
        exec(compile(path.read_bytes(), str(path), "exec"), module.__dict__)
    finally:
        sys.path[:] = prior_path
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior
    return module

def _paths(value):
    if isinstance(value, Path):
        yield value
    elif isinstance(value, dict):
        for item in value.values():
            yield from _paths(item)
    elif isinstance(value, (tuple, list, set, frozenset)):
        for item in value:
            yield from _paths(item)

def declared_selector(root):
    consumers = set()
    guardrails = root / GUARDRAIL_POLICY
    if guardrails.is_file():
        consumers.update(item["entrypoint"] for item in
                         json.loads(guardrails.read_text())["guardrails"])
    documentation = root / DOCUMENTATION_INVENTORY
    if documentation.is_file():
        for check in json.loads(documentation.read_text())["checks"]:
            if check["status"] == "active":
                command = check["command"]
                if len(command) < 2 or not command[1].endswith(".py"):
                    raise ValueError("documentation input consumer must declare a Python entrypoint")
                consumers.add(command[1])
    # These active source-sweep consumers declare input authorities beyond the
    # source roots (notably the taxonomy check's top-level README). Their Path
    # collections remain the authority, rather than duplicating a file list.
    for relative in (COMPATIBILITY_CHECK, TAXONOMY_CHECK):
        if (root / relative).is_file():
            consumers.add(relative)
    exact = set()
    modules = [_load(root, relative) for relative in sorted(consumers)]
    for module in modules:
        for value in vars(module).values():
            for path in _paths(value):
                if not path.is_absolute():
                    path = root / path
                try:
                    relative = path.relative_to(root)
                except ValueError:
                    continue
                # File constants (including missing/deleted ones) are authority.
                # A module's REPO_ROOT/AREA_ROOT directory is not a blanket input.
                if relative.parts and not path.is_dir():
                    exact.add(relative.as_posix())
        # The GA consumer also scans all public prose, beyond canonical pages.
        if hasattr(module, "PUBLIC_DOC_SUFFIXES"):
            exact.update(path.relative_to(root).as_posix()
                         for path in (root / "docs").rglob("*")
                         if path.is_file() and path.suffix in module.PUBLIC_DOC_SUFFIXES)
    compatibility = next((module for module in modules
                          if Path(module.__file__) == root / COMPATIBILITY_CHECK), None)

    def selected(name):
        if name in exact:
            return True
        relative = Path(name)
        # Use the checker itself: its skip rules intentionally exclude archived
        # phase records, and its source roots omit active planning records.
        return compatibility is not None and any(
            relative.is_relative_to(scan_root) for scan_root in compatibility.SCAN_ROOTS
        ) and not compatibility.should_skip(relative)
    return selected
