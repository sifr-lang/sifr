"""Run-scoped immutable compiler products; Cargo remains the producer authority.

A reference is valid only while its producer owns the run. Each consumer holds
a shared lease, checks the publication-bound file identity, and retains the product
until process exit. Retirement cannot delete a leased executable.
"""
from __future__ import annotations

import atexit
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import tempfile

REFERENCE_ENV = "SIFR_VERIFY_PREPARED_COMPILER"
_BORROWED = {}


def _stamp(path):
    value = path.stat()
    return (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns, value.st_ctime_ns)


def _digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def _family(root):
    return root / "target/verification/prepared-compilers"


def _owned(directory, root):
    if not re.fullmatch(r"run-[a-zA-Z0-9_-]+", directory.name):
        return False
    try:
        if directory.is_symlink() or directory.stat().st_uid != os.getuid():
            return False
        return json.loads((directory / "owner.json").read_text()) == {
            "schema": 1, "repository": str(root), "role": "prepared-development-compiler"}
    except (OSError, ValueError):
        return False


def retire(directory, root):
    """Only remove our inactive product when no consumer holds its lease."""
    import fcntl
    if not _owned(directory, root):
        return False
    try:
        with (directory / "owner.lock").open("rb") as owner, \
             (directory / "consumers.lock").open("rb") as consumers:
            try:
                fcntl.flock(owner, fcntl.LOCK_EX | fcntl.LOCK_NB)
                (directory / "retired").touch()
                fcntl.flock(consumers, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError:
                return False
            shutil.rmtree(directory)
        return True
    except FileNotFoundError:
        return False


def release_reference(reference):
    borrowed = _BORROWED.pop(str(reference), None)
    if borrowed is not None:
        borrowed["lease"].close()
        retire(borrowed["directory"], borrowed["repository"])


def _release_all():
    for reference in list(_BORROWED):
        release_reference(reference)


atexit.register(_release_all)


class PreparedCompilerSession:
    def __init__(self, repository):
        self.repository = Path(repository).resolve()
        self.directory = None
        self.owner = None
        self.reference = None
        self.token = None

    def __enter__(self):
        import fcntl
        family = _family(self.repository)
        family.mkdir(parents=True, exist_ok=True)
        for directory in family.iterdir():
            if directory.is_dir():
                retire(directory, self.repository)
        self.directory = Path(tempfile.mkdtemp(prefix="run-", dir=family))
        (self.directory / "consumers.lock").touch()
        self.owner = (self.directory / "owner.lock").open("w+b")
        fcntl.flock(self.owner, fcntl.LOCK_EX)
        # Publish ownership only after locking: another session must not reclaim
        # this directory during the interval between creation and admission.
        (self.directory / "owner.json").write_text(json.dumps({
            "schema": 1, "repository": str(self.repository),
            "role": "prepared-development-compiler"}))
        return self

    def publish(self, source, *, identity_reader, producer):
        if self.owner is None or self.reference is not None:
            raise RuntimeError("compiler publication requires one active unpublished owner")
        source = Path(source)
        destination = self.directory / "sifr"
        temporary = self.directory / "sifr.tmp"
        digest = hashlib.sha256()
        before = _stamp(source)
        with source.open("rb") as reader, temporary.open("xb") as writer:
            while block := reader.read(1024 * 1024):
                digest.update(block)
                writer.write(block)
        if _stamp(source) != before:
            raise RuntimeError("compiler producer changed during publication")
        temporary.chmod(stat.S_IMODE(source.stat().st_mode) & 0o555)
        if not os.access(temporary, os.X_OK):
            raise RuntimeError("compiler producer is not executable")
        temporary.rename(destination)
        published_stamp = _stamp(destination)
        compiler_build_id = identity_reader(destination)
        if _stamp(destination) != published_stamp:
            raise RuntimeError("compiler product changed during identity admission")
        if not isinstance(compiler_build_id, str) or not compiler_build_id:
            raise RuntimeError("compiler publication requires its canonical build identity")
        manifest = {
            "schema": 1, "repository": str(self.repository), "binary": "sifr",
            "sha256": digest.hexdigest(), "compiler_build_id": compiler_build_id,
            "producer": producer, "binary_stamp": list(published_stamp),
        }
        temporary_manifest = self.directory / "manifest.tmp"
        temporary_manifest.write_text(json.dumps(manifest, sort_keys=True))
        temporary_manifest.chmod(0o444)
        self.reference = self.directory / "manifest.json"
        temporary_manifest.rename(self.reference)
        # Bind the small admission manifest to the inherited run context. The
        # producer already verified every copied byte; consumers validate the
        # exact inode/size/mtime/ctime instead of rehashing a large binary each.
        self.token = json.dumps({"manifest": str(self.reference), "sha256": _digest(self.reference)})
        return destination

    def __exit__(self, *_):
        if self.owner is not None:
            (self.directory / "retired").touch()
            self.owner.close()
            self.owner = None
        if self.reference is not None:
            release_reference(self.token)
        if self.directory is not None:
            retire(self.directory, self.repository)


def _require_active_owner(directory):
    import fcntl
    if (directory / "retired").exists():
        raise RuntimeError("prepared compiler producer is no longer active")
    with (directory / "owner.lock").open("rb") as owner:
        try:
            fcntl.flock(owner, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            if (directory / "retired").exists():
                raise RuntimeError("prepared compiler producer is no longer active")
            return
        raise RuntimeError("prepared compiler producer is no longer active")


def consume_prepared_compiler(repository, token):
    """Reject invalid references; never fall back to a new Cargo build."""
    import fcntl
    root = Path(repository).resolve()
    try:
        ticket = json.loads(token)
        if (not isinstance(ticket, dict) or set(ticket) != {"manifest", "sha256"}
                or not isinstance(ticket["manifest"], str)
                or not isinstance(ticket["sha256"], str)
                or not re.fullmatch("[0-9a-f]{64}", ticket["sha256"])):
            raise RuntimeError("invalid prepared compiler admission token")
        reference = Path(ticket["manifest"])
        directory = reference.parent
        if (reference.name != "manifest.json" or reference.is_symlink()
                or directory.parent.resolve() != _family(root).resolve()
                or not _owned(directory, root)):
            raise RuntimeError("prepared compiler reference has foreign ownership")
        _require_active_owner(directory)
        existing = _BORROWED.get(token)
        if existing is not None:
            if (existing["binary"].is_symlink() or _stamp(reference) != existing["manifest_stamp"]
                    or _stamp(existing["binary"]) != existing["binary_stamp"]):
                raise RuntimeError("leased compiler product changed")
            return existing["binary"]
        lease = (directory / "consumers.lock").open("rb")
        try:
            fcntl.flock(lease, fcntl.LOCK_SH)
            _require_active_owner(directory)
            manifest_stamp = _stamp(reference)
            if reference.stat().st_mode & 0o222 or _digest(reference) != ticket["sha256"]:
                raise RuntimeError("prepared compiler admission manifest changed")
            manifest = json.loads(reference.read_text())
            if (set(manifest) != {"schema", "repository", "binary", "sha256",
                                  "compiler_build_id", "producer", "binary_stamp"}
                    or manifest["schema"] != 1 or manifest["repository"] != str(root)
                    or manifest["binary"] != "sifr"
                    or not isinstance(manifest["compiler_build_id"], str)
                    or not manifest["compiler_build_id"]
                    or not isinstance(manifest["producer"], dict)
                    or not isinstance(manifest["sha256"], str)
                    or not re.fullmatch("[0-9a-f]{64}", manifest["sha256"])):
                raise RuntimeError("invalid prepared compiler manifest")
            binary = directory / "sifr"
            mode = binary.lstat().st_mode
            if not stat.S_ISREG(mode) or mode & 0o222 or not os.access(binary, os.X_OK):
                raise RuntimeError("prepared compiler is not an immutable executable")
            binary_stamp = _stamp(binary)
            if (list(binary_stamp) != manifest["binary_stamp"]
                    or _stamp(reference) != manifest_stamp):
                raise RuntimeError("prepared compiler integrity mismatch")
            _require_active_owner(directory)
        except BaseException:
            lease.close()
            raise
        _BORROWED[token] = {
            "lease": lease, "binary": binary, "binary_stamp": binary_stamp,
            "manifest_stamp": manifest_stamp, "directory": directory, "repository": root,
        }
        return binary
    except (OSError, ValueError, TypeError) as error:
        raise RuntimeError("invalid prepared compiler reference") from error
