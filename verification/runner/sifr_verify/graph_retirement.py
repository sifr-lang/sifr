"""Exclusive, owned Cargo graph lifetimes with immutable artifact retention."""
from __future__ import annotations

import json
import os
import shutil
import stat
import tempfile
from pathlib import Path

from .execution_identity import artifact_identity
from .resource_admission import ResourceError

GRAPH_PATHS = ("target/sysroot_release/source-cargo-target", "target/sysroot_release/cargo-target")
CACHE_TAG = b"Signature: 8a477f597d28d172789f06886806bc55\n"


def plain_path(root: Path, path: Path) -> Path:
    """Reject links before creating anything in the owned worktree."""
    path = path.absolute()
    if not path.is_relative_to(root):
        raise ResourceError("artifact path escapes the owned worktree", "unavailable")
    for candidate in [path, *path.parents]:
        if candidate == root:
            break
        if candidate.is_symlink():
            raise ResourceError("artifact path has a symlink ancestor", "unavailable")
    if path.resolve() != path:
        raise ResourceError("artifact path is not canonical", "unavailable")
    return path


def require_owned_path(root: Path, relative: str) -> Path:
    if relative not in GRAPH_PATHS:
        raise ResourceError("retirement path is not a declared isolated graph", "unavailable")
    root = root.resolve()
    return plain_path(root, root / relative)


def active_builds(root: Path, proc: Path = Path("/proc")) -> list[int]:
    found = []
    for entry in proc.iterdir():
        if not entry.name.isdigit():
            continue
        try:
            if entry.stat().st_uid != os.getuid():
                continue
            if (entry / "comm").read_text().strip() not in {"cargo", "rustc"}:
                continue
            cwd = (entry / "cwd").resolve(strict=True)
            if cwd == root or cwd.is_relative_to(root):
                found.append(int(entry.name))
        except (OSError, ValueError):
            continue  # A reaped process is not a live consumer.
    return found


class GraphLease:
    def __init__(self, root: Path, relative: str, owner: str):
        self.root = root.resolve()
        self.path = require_owned_path(self.root, relative)
        if not owner:
            raise ResourceError("graph owner must be explicit", "unavailable")
        self.owner = owner
        self.relative = relative
        leases = plain_path(self.root, self.root / "target/.validation-graph-leases")
        leases.mkdir(parents=True, exist_ok=True)
        if leases.is_symlink():
            raise ResourceError("graph lease directory must not be a symlink", "unavailable")
        self.marker = leases / (self.path.name + ".json")
        self.lock = leases / (self.path.name + ".lock")
        self.consumers: set[str] = set()
        self.passed: set[str] = set()
        self._stream = None
        self.eligible = False

    def acquire(self, consumers: set[str]):
        import fcntl  # Only the Linux cloud scheduler uses graph retirement.
        if not consumers or any(not value for value in consumers):
            raise ResourceError("graph lifetime requires all consumer IDs", "unavailable")
        if self.marker.is_symlink() or self.lock.is_symlink():
            raise ResourceError("graph marker/lock must not be a symlink", "unavailable")
        self._stream = self.lock.open("a+")
        try:
            fcntl.flock(self._stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
            # Unknown legacy caches can be consumed, but never reclaimed by us.
            existed = self.path.exists()
            self.path.mkdir(parents=True, exist_ok=True)
            if not existed and not self.marker.exists():
                info = self.path.stat()
                data = {"owner": self.owner, "worktree": str(self.root), "graph": self.relative,
                        "device": info.st_dev, "inode": info.st_ino, "uid": os.getuid()}
                with self.marker.open("x") as stream:
                    json.dump(data, stream, sort_keys=True)
            if self.marker.exists():
                data = json.loads(self.marker.read_text())
                info = self.path.stat()
                self.eligible = data == {"owner": self.owner, "worktree": str(self.root), "graph": self.relative,
                                         "device": info.st_dev, "inode": info.st_ino, "uid": os.getuid()}
            if self.eligible:
                # Cargo only tags target roots it creates itself. Our lease
                # creates the root first, so initialize the standard cache tag
                # for this proven owned graph; never tag an unknown cache.
                tag = self.path / "CACHEDIR.TAG"
                if tag.is_symlink():
                    raise ResourceError("Cargo cache tag must not be a symlink", "unavailable")
                if not tag.exists():
                    with tag.open("xb") as stream:
                        stream.write(CACHE_TAG + b"# Session-owned Cargo target initialized before preparation.\n")
                        stream.flush()
                        os.fsync(stream.fileno())
                if not tag.read_bytes().startswith(CACHE_TAG):
                    raise ResourceError("owned Cargo cache tag is invalid", "unavailable")
            self.consumers = set(consumers)
            return self
        except BaseException:
            self.close()
            raise

    def passed_consumer(self, identifier: str):
        if identifier not in self.consumers:
            raise ResourceError("unselected graph consumer cannot retire a graph", "unavailable")
        self.passed.add(identifier)

    def retire(self, *, retained: Path, protected: list[Path], command_runner, env: dict) -> dict:
        if self._stream is None or self.passed != self.consumers:
            raise ResourceError("active, failed or incomplete graph consumers prevent retirement", "unavailable")
        require_owned_path(self.root, self.relative)
        if not self.eligible:
            return {"graph": self.relative, "retired": False, "reason": "unknown-or-other-owner", "reclaimed_bytes": 0}
        info = self.path.stat()
        data = json.loads(self.marker.read_text())
        if data != {"owner": self.owner, "worktree": str(self.root), "graph": self.relative,
                    "device": info.st_dev, "inode": info.st_ino, "uid": os.getuid()}:
            raise ResourceError("graph identity changed under its lease", "unavailable")
        if active_builds(self.root):
            raise ResourceError("live Cargo/rustc processes prevent graph retirement", "unavailable")
        # Directory links could make Cargo cleanup touch another owner's inputs.
        if any(path.is_symlink() for path in self.path.rglob("*")):
            raise ResourceError("symlinks prevent safe graph retirement", "unavailable")
        retained = plain_path(self.root, retained)
        if retained == self.path or retained.is_relative_to(self.path):
            raise ResourceError("retained artifact destination is inside retired graph", "unavailable")
        # Net recovery includes the disk cost of making durable retained copies.
        before = shutil.disk_usage(self.root).free
        copies = []
        for artifact in protected:
            artifact = plain_path(self.root, artifact)
            if not artifact.is_relative_to(self.path):
                raise ResourceError("protected artifact is outside the selected graph", "unavailable")
            original = artifact_identity(artifact)
            destination = plain_path(self.root, retained / original["sha256"] / artifact.name)
            destination.parent.mkdir(parents=True, exist_ok=True)
            if not destination.exists():
                with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as stream:
                    temporary = Path(stream.name)
                try:
                    shutil.copyfile(artifact, temporary)
                    os.chmod(temporary, stat.S_IMODE(artifact.stat().st_mode) & ~0o222)
                    with temporary.open("rb") as stream:
                        os.fsync(stream.fileno())
                    os.link(temporary, destination)
                finally:
                    temporary.unlink(missing_ok=True)
            current = artifact_identity(destination)
            if current["sha256"] != original["sha256"] or current["size_bytes"] != original["size_bytes"]:
                raise ResourceError("retained compiler artifact identity differs", "unavailable")
            copies.append({"original": original, "retained": current})
        if not copies:
            raise ResourceError("graph retirement requires retained compiler artifacts", "unavailable")
        # Cargo owns removal of its isolated graph; mutable targets are never linked.
        command_runner(["cargo", "clean", "--target-dir", str(self.path)], env=env)
        after = shutil.disk_usage(self.root).free
        for copy in copies:
            if artifact_identity(Path(copy["retained"]["requested_path"])) != copy["retained"]:
                raise ResourceError("retained compiler drifted during cleanup", "unavailable")
        self.marker.unlink()
        self.eligible = False
        return {"graph": self.relative, "retired": True, "copies": copies,
                "reclaimed_bytes": after - before, "free_bytes_after": after}

    def close(self):
        if self._stream is not None:
            self._stream.close()
            self._stream = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
