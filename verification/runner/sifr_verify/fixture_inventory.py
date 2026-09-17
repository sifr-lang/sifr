"""Content inventory of compiler, generation, and verification input authorities."""
import hashlib
import json
import os
import subprocess
from pathlib import Path
from .paths import REPO_ROOT
from .declared_fixture_inputs import declared_selector

# Ownership determines inputs, never a file-extension allowlist: generators and
# fixtures consume arbitrary assets. Prose/phase records outside these roots do
# not invalidate evidence. Diagnostic documentation is tested compiler input.
INPUT_ROOTS = (
    "crates", "stdlib", "verification", "third_party", "scripts", ".cargo",
    "demos", "vendor", "editor_integrations", ".github",
    "docs/errors", "docs/schemas", "docs/diagnostics",
)
ROOT_INPUTS = {
    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "sifr.toml", "sysroot.toml",
    ".gitmodules", ".gitignore", ".gitattributes",
}

def _selected(name):
    return name in ROOT_INPUTS or any(
        name == owner or name.startswith(owner + "/") for owner in INPUT_ROOTS)

def _git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root)

def _records(root, additional, prefix=""):
    # NUL-delimited names preserve arbitrary fixture filenames. Git's index is
    # authoritative for tracked files; nonignored additions invalidate evidence
    # before commit. Ignored target/cache artifacts are not inputs.
    entries = {}
    for item in _git(root, "ls-files", "--stage", "-z").split(b"\0"):
        if item:
            metadata, name = item.split(b"\t", 1)
            mode, oid, stage = metadata.decode().split()
            if stage != "0":
                raise ValueError("cannot inventory an unmerged index")
            entries[os.fsdecode(name)] = (mode, oid)
    for name in _git(root, "ls-files", "--others", "--exclude-standard", "-z").split(b"\0"):
        if name:
            entries.setdefault(os.fsdecode(name), (None, None))
    for name, (mode, oid) in sorted(entries.items()):
        full_name = prefix + name
        if not (_selected(full_name) or additional(full_name)):
            continue
        path = root / name
        if mode == "160000":
            # Record the declared gitlink even when uninitialized, plus the
            # actual checkout and dirty input contents when present.
            identity = {"gitlink": oid, "checkout": None}
            initialized = (path / ".git").exists()
            if initialized:
                identity["checkout"] = _git(path, "rev-parse", "HEAD").decode().strip()
            content = json.dumps(identity, sort_keys=True).encode()
            yield {"path": full_name, "sha256": hashlib.sha256(content).hexdigest()}
            if initialized:
                yield from _records(path, additional, full_name + "/")
        elif path.is_symlink():
            # Include link identity and the bytes consumed through a file link.
            # Do not traverse directory links into ignored/external inventories.
            content = b"symlink\0" + os.fsencode(os.readlink(path))
            if path.is_file():
                content += b"\0" + path.read_bytes()
            yield {"path": full_name, "sha256": hashlib.sha256(content).hexdigest()}
        elif path.is_file():
            yield {"path": full_name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}

def inventory(root=REPO_ROOT):
    root = Path(root)
    records = sorted(_records(root, declared_selector(root)), key=lambda record: record["path"])
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
