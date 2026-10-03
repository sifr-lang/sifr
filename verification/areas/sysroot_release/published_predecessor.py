"""Acquire verified published predecessor bytes; preparation is not qualification."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import tarfile
import time
import urllib.request

from packaging.version import Version

POLICY = Path(__file__).with_name("published_predecessors.json")
RESERVE_BYTES = 8 * 1024**3
MAX_UNPACKED_BYTES = 4 * 1024**3


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def select(policy: dict, target: str, candidate_version: str) -> dict:
    targets = ("aarch64-apple-darwin", "aarch64-unknown-linux-gnu",
               "x86_64-apple-darwin", "x86_64-unknown-linux-gnu")
    if (policy.get("schema_version") != 1 or policy.get("repository") != "sifr-lang/sifr"
            or not re.fullmatch(r"[0-9a-f]{40}", policy.get("source_commit", ""))
            or type(policy.get("release_id")) is not int or policy["release_id"] <= 0
            or sorted(row.get("name", "") for row in policy.get("assets", [])) != sorted(
                f"sifr-{policy['version']}-{native}.tar.gz" for native in targets)):
        raise ValueError("unsupported predecessor registry")
    if Version(candidate_version) <= Version(policy["version"]):
        raise ValueError("registered published predecessor must precede the candidate")
    name = f"sifr-{policy['version']}-{target}.tar.gz"
    matches = [row for row in policy["assets"] if row["name"] == name]
    if len(matches) != 1:
        raise ValueError("one registered native predecessor asset is required")
    asset = matches[0]
    expected_url = f"https://github.com/sifr-lang/sifr/releases/download/{policy['version']}/{name}"
    if (asset.get("browser_download_url") != expected_url or type(asset.get("size")) is not int
            or not 0 < asset["size"] <= 512 * 1024**2
            or not isinstance(asset.get("digest"), str) or not asset["digest"].startswith("sha256:")
            or not re.fullmatch(r"sha256:[0-9a-f]{64}", asset["digest"])):
        raise ValueError("predecessor asset identity is incomplete")
    return asset


def fetch_archive(asset: dict, output: Path, *, opener=urllib.request.urlopen) -> None:
    if shutil.disk_usage(output.parent).free < RESERVE_BYTES + asset["size"]:
        raise OSError("published archive exceeds the admitted disk allowance")
    count = 0
    deadline = time.monotonic() + 300
    with opener(asset["browser_download_url"], timeout=30) as response, output.open("xb") as stream:
        while chunk := response.read(1024 * 1024):
            count += len(chunk)
            if count > asset["size"] or time.monotonic() > deadline:
                raise ValueError("published predecessor transfer exceeds declared size/deadline")
            if shutil.disk_usage(output.parent).free < RESERVE_BYTES:
                raise OSError("published predecessor transfer reached disk reserve")
            stream.write(chunk)
        stream.flush()
        os.fsync(stream.fileno())
    if count != asset["size"] or "sha256:" + digest(output) != asset["digest"]:
        raise ValueError("published predecessor bytes differ from registered size/hash")
    output.chmod(0o400)


def unpack(archive: Path, root: Path) -> dict:
    with tarfile.open(archive, "r:gz") as package:
        members = package.getmembers()
        if len(members) > 100000:
            raise ValueError("published package inventory exceeds the declared limit")
        total = sum(member.size for member in members if member.isfile())
        if total <= 0 or total > MAX_UNPACKED_BYTES:
            raise ValueError("published package exceeds the declared decoded allocation")
        allocated = sum(((member.size + 4095) // 4096) * 4096 + 4096 for member in members)
        if shutil.disk_usage(root.parent).free < RESERVE_BYTES + allocated + 128 * 1024**2:
            raise OSError("published package exceeds available decoded allocation")
        root.mkdir(mode=0o700)
        # Python's data filter rejects escaping paths/links, devices and special
        # files. Check every entry before allowing the first extraction write.
        for member in members:
            tarfile.data_filter(member, str(root))
        def admitted_members():
            for member in members:
                if shutil.disk_usage(root.parent).free < RESERVE_BYTES + member.size + 128 * 1024**2:
                    raise OSError("published extraction reached its monitored reserve")
                yield member
        package.extractall(root, members=admitted_members(), filter="data")
    paths = sorted(path for path in root.rglob("*") if path.is_file())
    return {"decoded_bytes": total, "files": [
        {"path": str(path.relative_to(root)), "size_bytes": path.stat().st_size, "sha256": digest(path)}
        for path in paths]}


def prepare(*, policy: dict, target: str, candidate_version: str, output: Path) -> dict:
    asset = select(policy, target, candidate_version)
    output = output.absolute()
    if output.is_symlink() or any(parent.is_symlink() for parent in output.parents):
        raise ValueError("predecessor store must not have symlink ancestors")
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    started = datetime.now(timezone.utc).isoformat()
    receipt = {"schema_version": 1, "kind": "preparation-output", "runtime_assertions": 0,
               "status": "incomplete", "started_utc": started, "target": target,
               "candidate_version": candidate_version, "published_version": policy["version"],
               "release_id": policy["release_id"], "publication_source_commit": policy["source_commit"],
               "policy_sha256": hashlib.sha256(json.dumps(policy, sort_keys=True).encode()).hexdigest(),
               "producer_tool_sha256": digest(Path(__file__)),
               "asset": asset}
    def save():
        path = output / "receipt.json"
        temporary = output / "receipt.tmp"
        with temporary.open("w") as stream:
            json.dump(receipt, stream, indent=2)
            stream.flush()
            os.fsync(stream.fileno())
        temporary.replace(path)
    save()
    try:
        archive = output / asset["name"]
        fetch_archive(asset, archive)
        receipt["archive"] = {"path": str(archive), "sha256": digest(archive), "size_bytes": archive.stat().st_size}
        receipt["package"] = unpack(archive, output / "package")
        receipt["status"] = "prepared"
    except BaseException as error:
        receipt["status"] = "failed"
        receipt["failure"] = type(error).__name__ + ": " + str(error)
        raise
    finally:
        receipt["finished_utc"] = datetime.now(timezone.utc).isoformat()
        save()
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True)
    parser.add_argument("--candidate-version", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    receipt = prepare(policy=json.loads(POLICY.read_text()), target=args.target,
                      candidate_version=args.candidate_version, output=args.output)
    print(json.dumps({"status": receipt["status"], "receipt": str(args.output / "receipt.json"),
                      "runtime_assertions": 0}))


if __name__ == "__main__":
    main()
