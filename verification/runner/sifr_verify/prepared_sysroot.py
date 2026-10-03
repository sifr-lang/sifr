"""Retain preparation outputs, without claiming any runtime assertion passed.

Producer and consumer run through the same isolated entry point. Receipts are
local, session-bound, source/tool/runtime-bound and immutable. No cross-commit
equivalence or assertion checkpoint is inferred from a successful build.
"""
from __future__ import annotations

import argparse
from datetime import UTC, datetime, timedelta
import json
import os
from pathlib import Path
import shutil
import shlex
import sys
import uuid

from .checkpoint_runtime import isolated_python, stdlib_identity
from .correctness_checkpoints import safe_store
from .compressed_artifacts import restore
from .execution_evidence import write_evidence
from .execution_identity import EvidenceError, artifact_identity, digest, execution_key
from .graph_retirement import GRAPH_PATHS, GraphLease, plain_path
from .paths import REPO_ROOT
from .process_execution import execute
from .profile_commands import CommandFailed
from .sysroot_preparation import producer
from .resource_admission import admit, discover

OWNER_VARIABLE = "SIFR_VERIFY_PREPARED_SYSROOT_OWNER"


def tree_identity(path: Path) -> dict:
    """Registry/toolchain bytes are build inputs even outside the Git tree."""
    path = path.absolute()
    if not path.exists():
        return {"path": str(path), "present": False, "files": []}
    files, links = [], []
    def visit(item: Path, ancestors: set):
        if item.is_symlink():
            links.append({"path": str(item), "target": str(item.resolve(strict=True)),
                          "link": os.readlink(item)})
        if item.is_dir():
            info = item.stat()
            identity = (info.st_dev, info.st_ino)
            if identity in ancestors:
                raise EvidenceError("cyclic directory-link dependency in preparation closure")
            for child in sorted(item.iterdir()):
                visit(child, ancestors | {identity})
        elif item.is_file():
            files.append(artifact_identity(item))
        else:
            raise EvidenceError("unknown non-file dependency in preparation closure")
    visit(path, set())
    return {"path": str(path), "present": True, "files": files, "links": links}


def command(action: str, kind: str, root: Path = REPO_ROOT) -> list[str]:
    return [*isolated_python(str(root / "verification/runner/prepared_sysroot.py")), action, kind]


def store(root: Path, env: dict[str, str]) -> tuple[Path, dict]:
    owner = env.get(OWNER_VARIABLE, "")
    try:
        if str(uuid.UUID(owner)) != owner:
            raise ValueError("noncanonical UUID")
    except ValueError as error:
        raise EvidenceError("prepared sysroot owner must be an explicit session UUID") from error
    path = root / "target/verification/prepared-sysroot" / owner
    expected = {"kind": "local-sysroot-preparation-v1", "uid": os.getuid(),
                "worktree": str(root.resolve()), "session": owner}
    return safe_store(path), expected


def recipe(kind: str, root: Path, env: dict[str, str], directory: Path):
    if kind == "source":
        build = producer("source_build", root)
        argv, environment, output = build.source_build_configuration(root, env)
        return argv, environment, output, GRAPH_PATHS[0]
    if kind == "package":
        build = producer("package_build", root)
        host = build.host_target(env)
        argv, environment = build.package_build_configuration(root, env, host, directory / "archive")
        output = directory / "archive" / f"sifr-{build.RELEASE_VERSION}-{host}.tar.gz"
        return argv, environment, output, GRAPH_PATHS[1]
    raise EvidenceError("unknown preparation output kind")


def key(kind: str, root: Path, env: dict[str, str], directory: Path, expected: dict) -> dict:
    argv, environment, _, _ = recipe(kind, root, env, directory)
    result = execution_key(selection={"preparation": kind, "required_kinds": {}},
        commands=[argv], artifacts=[], producer=expected, services={}, env=environment, root=root)
    entry = str(root / "verification/runner/prepared_sysroot.py")
    result["inputs"]["runtime"]["isolated_closure"] = stdlib_identity(entry, environment, cwd=root)
    runtime = result["inputs"]["runtime"]
    cargo = Path(environment.get("CARGO_HOME", str(Path.home() / ".cargo")))
    rustc = Path(runtime["tool_bytes"]["selected_rustc"]["path"])
    runtime["build_dependency_trees"] = [tree_identity(path) for path in (
        cargo / "registry/src", cargo / "git/checkouts", rustc.parent.parent / "lib",
        Path("/usr/include"), Path("/usr/lib/gcc"))]
    for variable in ("LIBCLANG_PATH", "LD_LIBRARY_PATH", "RUBYLIB"):
        for value in environment.get(variable, "").split(os.pathsep):
            if value:
                runtime["build_dependency_trees"].append(tree_identity(Path(value)))
    runtime["native_build_tools"] = {}
    for variable, default in (("CC", "cc"), ("CXX", "c++"), ("AR", "ar"), ("LD", "ld"),
                              ("RUSTC", "rustc"), ("RUSTDOC", "rustdoc"),
                              ("RUSTC_WRAPPER", ""), ("RUSTC_WORKSPACE_WRAPPER", ""),
                              ("RUBY", "ruby"), ("CMAKE", "cmake"), ("PKG_CONFIG", "pkg-config")):
        setting = environment.get(variable, default)
        if not setting:
            continue
        tokens = shlex.split(setting)
        executable = shutil.which(tokens[0], path=environment.get("PATH"))
        if executable is None:
            if variable in environment:
                raise EvidenceError("unknown native build tool executable")
            runtime["native_build_tools"][variable] = {"present": False}
            continue
        runtime["native_build_tools"][variable] = artifact_identity(Path(executable))
    runtime["build_settings"] = {name: digest(value) for name, value in sorted(environment.items())
        if (name.startswith(("CARGO_", "RUST", "SIFR_", "SYNTAQLITE_", "DEP_", "LD_"))
            and not name.startswith("SIFR_VERIFY_")) or name in {
                "HOME", "CC", "CXX", "CFLAGS", "CXXFLAGS", "CPPFLAGS", "LDFLAGS", "AR", "LD",
                "WASI_SDK_PATH", "RUBY", "RUBYLIB", "RUBYOPT", "CMAKE", "PKG_CONFIG", "PKG_CONFIG_PATH"}}
    result["input_digest"] = digest(result["inputs"])
    return result


def checked(argv, *, root, env):
    result = execute(argv, cwd=root, env=env, emit=lambda stream, data: (
        sys.stdout.buffer if stream == "stdout" else sys.stderr.buffer).write(data),
        deadline_seconds=env.get("SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS", "7200"))
    if result.returncode or result.cause != "exit":
        error = CommandFailed(result.returncode or 2, result.cause)
        error.outcome = result
        raise error


def prepare(kind: str, *, root: Path, env: dict[str, str]) -> dict:
    directory, expected = store(root, env)
    receipt = directory / f"{kind}.json"
    if receipt.exists():
        raise EvidenceError("preparation receipt already exists; use a new session")
    before = key(kind, root, env, directory, expected)
    argv, environment, output, graph = recipe(kind, root, env, directory)
    graph_owner = env.get("SIFR_VERIFY_GRAPH_OWNER", expected["session"])
    # Only compilation/packaging consumes this build graph. Runtime assertions
    # consume the independently retained output and remain required later.
    with GraphLease(root, graph, graph_owner).acquire({f"prepare-{kind}"}) as lease:
        checked(argv, root=root, env=environment)
        if key(kind, root, env, directory, expected) != before:
            raise EvidenceError("sysroot preparation inputs changed")
        original = artifact_identity(output)
        lease.passed_consumer(f"prepare-{kind}")
        if kind == "source":
            retirement = lease.retire(retained=directory / "source", protected=[output],
                command_runner=lambda argv, env: checked(argv, root=root, env=env), env=environment,
                max_encoded_bytes=512*1024**2)
            if retirement["retired"]:
                copy = retirement["copies"][0]
                # Restore only after Cargo has freed the large graph. The
                # compressed bytes remain durable throughout this transition.
                admit(discover(disk_path=root), dict(disk_growth_bytes=4*1024**2,
                    retained_copy_bytes=copy["decoded_size_bytes"], disk_reserve_bytes=8*1024**3,
                    memory_peak_bytes=1024**3, tmpfs_growth_bytes=0, memory_reserve_bytes=2*1024**3))
                output = restore(copy, Path(copy["retained"]["path"]).with_suffix(""))
            else:
                # Borrowed graphs can be freshly built and consumed, but never
                # cleaned. The selected output still gets an independent copy.
                retained = safe_store(directory / "source") / "sifr"
                admit(discover(disk_path=root), dict(disk_growth_bytes=4*1024**2,
                    retained_copy_bytes=original["size_bytes"], disk_reserve_bytes=8*1024**3,
                    memory_peak_bytes=1024**3, tmpfs_growth_bytes=0, memory_reserve_bytes=2*1024**3))
                with output.open("rb") as src, retained.open("xb") as dst:
                    shutil.copyfileobj(src, dst)
                    dst.flush()
                    os.fsync(dst.fileno())
                retained.chmod(output.stat().st_mode & ~0o222)
                output = retained
        else:
            # The complete archive already retains the packaged compiler and
            # runtime bytes. Keep the raw compiler too for diagnostic custody.
            build = producer("package_build", root)
            binary = build.package_compiler_path(root, env, build.host_target(env))
            output.chmod(output.stat().st_mode & ~0o222)
            with output.open("rb") as stream:
                os.fsync(stream.fileno())
            retirement = lease.retire(retained=directory / "package-compiler", protected=[binary],
                command_runner=lambda argv, env: checked(argv, root=root, env=env), env=environment)
        current = artifact_identity(output)
        if (current["sha256"], current["size_bytes"]) != (original["sha256"], original["size_bytes"]):
            raise EvidenceError("sysroot preparation artifact drifted during retirement")
        if key(kind, root, env, directory, expected) != before:
            raise EvidenceError("sysroot preparation inputs changed during retirement")
        parent = os.open(output.parent, os.O_RDONLY)
        try:
            os.fsync(parent)
        finally:
            os.close(parent)
        payload = {"schema_version": 1, "claim": "preparation-output", "key": before,
                   "finished_at": datetime.now(UTC).isoformat(), "artifact": current,
                   "retirement": retirement, "runtime_assertions_executed": 0}
        payload["receipt_digest"] = digest(payload)
        write_evidence(receipt, payload)
        return payload


def consume(kind: str, *, root: Path, env: dict[str, str]) -> Path:
    directory, expected = store(root, env)
    receipt = directory / f"{kind}.json"
    if receipt.is_symlink() or receipt.stat().st_uid != os.getuid() or receipt.stat().st_mode & 0o022:
        raise EvidenceError("preparation receipt is not owned")
    payload = json.loads(receipt.read_text())
    value = dict(payload)
    checksum = value.pop("receipt_digest", None)
    if digest(value) != checksum or value.get("claim") != "preparation-output":
        raise EvidenceError("preparation receipt is invalid")
    if value.get("runtime_assertions_executed") != 0 or value.get("schema_version") != 1:
        raise EvidenceError("preparation cannot claim runtime execution")
    finished = datetime.fromisoformat(value["finished_at"])
    now = datetime.now(UTC)
    if finished > now or now - finished > timedelta(hours=24):
        raise EvidenceError("preparation receipt is stale")
    if value["key"] != key(kind, root, env, directory, expected):
        raise EvidenceError("preparation input identity differs")
    artifact = plain_path(directory, Path(value["artifact"]["path"]))
    if not artifact.is_relative_to(directory) or artifact.is_symlink():
        raise EvidenceError("preparation output escapes its owned store")
    if (artifact.stat().st_uid != os.getuid() or artifact.stat().st_mode & 0o222
            or artifact_identity(artifact) != value["artifact"]):
        raise EvidenceError("preparation artifact changed")
    return artifact


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "consume", "identity"))
    parser.add_argument("kind", choices=("source", "package"))
    args = parser.parse_args()
    if args.action == "identity":
        directory, expected = store(REPO_ROOT, os.environ)
        value = key(args.kind, REPO_ROOT, os.environ, directory, expected)
        print(json.dumps({"claim": "preparation-dependency-observation", "input_digest": value["input_digest"],
            "observed_commit": value["observed_commit"], "build_dependency_files": sum(
                len(row["files"]) for row in value["inputs"]["runtime"]["build_dependency_trees"])}))
    elif args.action == "prepare":
        payload = prepare(args.kind, root=REPO_ROOT, env=os.environ.copy())
        print(json.dumps({"claim": payload["claim"], "input_digest": payload["key"]["input_digest"],
                          "artifact": payload["artifact"], "retirement": payload["retirement"],
                          "runtime_assertions_executed": 0}, sort_keys=True))
    else:
        print(consume(args.kind, root=REPO_ROOT, env=os.environ.copy()))
    return 0
