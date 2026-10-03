"""Lossless, bounded independent retention of large immutable build outputs."""
import gzip
import hashlib
import os
from pathlib import Path
import stat

from .execution_identity import artifact_identity
from .resource_admission import ResourceError


def decoded_identity(path: Path, *, limit: int) -> tuple[str, int]:
    hasher, size = hashlib.sha256(), 0
    with gzip.open(path, "rb") as stream:
        for chunk in iter(lambda: stream.read(1024**2), b""):
            size += len(chunk)
            if size > limit:
                raise ResourceError("decoded retained artifact exceeds its bound", "unavailable")
            hasher.update(chunk)
    return hasher.hexdigest(), size


def compress(source: Path, destination: Path, *, max_encoded_bytes: int) -> dict:
    original = artifact_identity(source)
    with source.open("rb") as src, destination.open("xb") as stream:
        class Bounded:
            size = 0
            def write(self, data):
                if self.size + len(data) > max_encoded_bytes:
                    raise ResourceError("encoded retention exceeds admitted allocation", "enospc")
                self.size += len(data)
                return stream.write(data)
        with gzip.GzipFile(filename="", fileobj=Bounded(), mode="wb", compresslevel=1, mtime=0) as dst:
            for chunk in iter(lambda: src.read(1024**2), b""):
                dst.write(chunk)
        stream.flush()
        os.fsync(stream.fileno())
    destination.chmod(0o400)
    if (artifact_identity(source) != original or decoded_identity(destination, limit=original["size_bytes"])
            != (original["sha256"], original["size_bytes"])):
        raise ResourceError("compressed artifact failed exact byte verification", "unavailable")
    return {"original": original, "retained": artifact_identity(destination), "encoding": "gzip",
            "decoded_sha256": original["sha256"], "decoded_size_bytes": original["size_bytes"],
            "original_mode": stat.S_IMODE(source.stat().st_mode)}


def restore(copy: dict, destination: Path) -> Path:
    encoded = Path(copy["retained"]["path"])
    if artifact_identity(encoded) != copy["retained"]:
        raise ResourceError("compressed artifact changed before restoration", "unavailable")
    hasher, size = hashlib.sha256(), 0
    with gzip.open(encoded, "rb") as src, destination.open("xb") as dst:
        for chunk in iter(lambda: src.read(1024**2), b""):
            size += len(chunk)
            if size > copy["decoded_size_bytes"]:
                raise ResourceError("restored artifact exceeds its bound", "unavailable")
            hasher.update(chunk)
            dst.write(chunk)
        dst.flush()
        os.fsync(dst.fileno())
    if ((hasher.hexdigest(), size) != (copy["decoded_sha256"], copy["decoded_size_bytes"])
            or artifact_identity(encoded) != copy["retained"]):
        raise ResourceError("restored artifact failed exact byte verification", "unavailable")
    destination.chmod(copy["original_mode"] & ~0o222)
    return destination
