"""One freshly compiled corpus owned by one read-only quality transaction."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import shutil
import stat


def snapshot(root: Path) -> dict[str, str]:
    if root.is_symlink() or not root.is_dir():
        raise RuntimeError("generated corpus root must be a real directory")
    files = {}
    for directory, directories, names in os.walk(root, followlinks=False):
        for name in directories:
            path = Path(directory) / name
            if path.is_symlink():
                raise RuntimeError(f"generated corpus directory alias: {path}")
        for name in names:
            path = Path(directory) / name
            if not stat.S_ISREG(path.lstat().st_mode):
                raise RuntimeError(f"generated corpus input is not regular: {path}")
            with path.open("rb") as stream:
                files[path.relative_to(root).as_posix()] = hashlib.file_digest(stream, "sha256").hexdigest()
    return files


class FreshCorpus:
    """Share fresh generated products, never source freshness or test results.

    A caller creates a new owner for each batch. Only the canonical compiler may
    produce a program. Consumers share that compilation's output, and every
    consumer still performs its own assertions. Nothing loads a previous owner.
    """

    def __init__(self, materialize, root: Path):
        self.materialize = materialize
        self.root = root
        self.root.mkdir(parents=True, exist_ok=False)
        self.programs = {}
        self.failures = {}
        self.consumer = None
        self.poisoned = None

    def get(self, entry, _consumer_workspace: Path) -> Path:
        if not self.consumer:
            raise RuntimeError("fresh corpus access requires an active consumer")
        if self.poisoned:
            raise RuntimeError(self.poisoned)
        if entry.id in self.failures:
            raise RuntimeError(f"fresh generation failed for {entry.id}: {self.failures[entry.id]}")
        if entry.id not in self.programs:
            try:
                crate = self.materialize(entry, self.root)
                if not crate.resolve().is_relative_to(self.root.resolve()):
                    raise RuntimeError("generated program escaped its batch owner")
                self.programs[entry.id] = {
                    "entry": entry, "root": crate, "snapshot": snapshot(crate), "consumers": [],
                }
            except Exception as error:
                self.failures[entry.id] = str(error)
                raise
        program = self.programs[entry.id]
        if program["entry"] != entry:
            raise RuntimeError(f"conflicting generated entry identity: {entry.id}")
        self.verify()
        program["consumers"].append(self.consumer)
        return program["root"]

    def verify(self) -> None:
        if self.poisoned:
            raise RuntimeError(self.poisoned)
        for name, program in self.programs.items():
            try:
                if snapshot(program["root"]) != program["snapshot"]:
                    raise RuntimeError("generated input bytes or file inventory changed")
            except (OSError, RuntimeError) as error:
                self.poisoned = f"read-only generated corpus mutated: {name}: {error}"
                raise RuntimeError(self.poisoned) from error

    def evidence(self) -> dict:
        return {
            "ownership": "fresh-current-batch",
            "programs": [{"id": name, "source_path": row["entry"].source_path,
                          "crate_root": str(row["root"]), "consumers": row["consumers"],
                          "generated_files": row["snapshot"]}
                         for name, row in self.programs.items()],
            "generation_failures": dict(self.failures), "poisoned": self.poisoned,
        }

    def finish(self, *, passed: bool, keep_success: bool) -> None:
        if passed and not keep_success:
            shutil.rmtree(self.root)
