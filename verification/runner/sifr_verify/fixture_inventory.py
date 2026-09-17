"""Input inventory for the existing area manifests and fixture consumers."""
import hashlib
import json
import subprocess
from pathlib import Path
from .paths import REPO_ROOT

def inventory(root=REPO_ROOT):
    names = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard"],
        cwd=root, text=True).splitlines()
    records = []
    for name in sorted(set(names)):
        path = root / name
        if not path.is_file():
            continue
        # Documentation-only phase records do not change compiler/verification inputs.
        # Demos and vendored builds may consume arbitrary data/configuration
        # assets, not only source suffixes. Track their complete file inventory.
        selected = name.startswith(("demos/", "vendor/")) or (
            name.startswith(("crates/", "stdlib/", "verification/", "third_party/", "scripts/", ".cargo/"))
            and path.suffix in {".rs", ".sifr", ".py", ".json", ".toml", ".sh", ".snap", ".txt", ".lock", ".c", ".h", ".pyi"}
        ) or name in {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml"} or (
            name.startswith("crates/sifr_diagnostics/error_page_examples/")) or (
            name.startswith(("docs/errors/", "docs/schemas/", "docs/diagnostics/"))
            and path.suffix in {".mdx", ".json"})
        if selected:
            records.append({"path": name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
    raw = json.dumps(records, sort_keys=True, separators=(",", ":")).encode()
    return {"schema_version": 1, "input_digest": hashlib.sha256(raw).hexdigest(),
            "inputs": records}

def compare(previous, current):
    before = {item["path"]: item["sha256"] for item in previous["inputs"]}
    after = {item["path"]: item["sha256"] for item in current["inputs"]}
    return {"unchanged": before == after,
            "added": sorted(after.keys() - before.keys()),
            "removed": sorted(before.keys() - after.keys()),
            "changed": sorted(name for name in before.keys() & after.keys()
                              if before[name] != after[name])}

def compare_paths(reference, candidate, expected):
    """Later metadata/restored providers plug in real executions, never empty success."""
    if reference is None or candidate is None or expected is None:
        raise ValueError("comparison requires both paths and an independent expectation")
    left, right = reference(), candidate()
    if not expected(left) or not expected(right):
        raise AssertionError("independent expected outcome failed")
    if left != right:
        raise AssertionError("compiler path outcomes differ")
    return left

