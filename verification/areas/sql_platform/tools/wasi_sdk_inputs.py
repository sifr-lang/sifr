"""Select and verify the official WASI SDK archive and its extracted contents."""

from __future__ import annotations

import hashlib
import os
import platform
import subprocess
import tarfile
from pathlib import Path

SDK_VERSION = "34.0"
SDK_RELEASE = "https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-34"
# Official release asset digests; the archive is checked before it is opened.
SDK_ASSETS = {
    "arm64-macos": (180322002, "9c59398106b417f8f14913380fdf0097a8cc0ff4af9eb3ce0065a859e88d49e9"),
    "x86_64-macos": (182668326, "87d27fa8adc68dee59bfbf2e22a6d34ef717c34d6bf1d8af2a56fc929d9ce0eb"),
    "arm64-linux": (192255247, "f7e243dff54d60bcc576e94d6166b69f410f2500ae4a9ceef34315be10e77971"),
    "x86_64-linux": (192383077, "b761e3a0721dbae9c09a0059e5fdb2bf917d1b4a8a7b430fb3b5aafb0984b2c4"),
    "riscv64-linux": (191280428, "71417f267b3a2015780b10045707eecec9fef6d064d2f79e82ad0618f9e4eac5"),
    "arm64-windows": (620307231, "45e1c71f3e965621e7b98ebe1d37b0e4b1f77f3e8072113ffb4534e67b1a4b7c"),
    "x86_64-windows": (619003408, "cccb5c323a9b34f0349a9b09e8804a0a7632c68c3310f4b5f437ed57d7e71d8f"),
}


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def content_sha256(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            value = b"link\0" + os.readlink(path).encode()
        elif path.is_file():
            value = b"file\0" + bytes.fromhex(sha256(path))
        else:
            continue
        digest.update(path.relative_to(root).as_posix().encode() + b"\0" + value)
    return digest.hexdigest()


def host_platform() -> str:
    systems = {"Darwin": "macos", "Linux": "linux", "Windows": "windows"}
    machines = {"aarch64": "arm64", "arm64": "arm64", "x86_64": "x86_64", "AMD64": "x86_64", "riscv64": "riscv64"}
    key = f"{machines.get(platform.machine(), '')}-{systems.get(platform.system(), '')}"
    if key not in SDK_ASSETS:
        raise ValueError(f"WASI SDK {SDK_VERSION} has no qualified archive for this host")
    return key


def verify_contents(archive: Path, sdk: Path) -> None:
    """Compare every installed file/link to the authenticated release archive."""
    recorded: set[str] = set()
    with tarfile.open(archive, "r:gz") as package:
        for member in package:
            parts = Path(member.name).parts
            if not parts or ".." in parts or Path(member.name).is_absolute():
                raise ValueError("SDK archive contains an invalid path")
            relative = Path(*parts[1:])
            if not relative.parts or member.isdir():
                continue
            path = sdk / relative
            recorded.add(relative.as_posix())
            if member.issym():
                if not path.is_symlink() or os.readlink(path) != member.linkname:
                    raise ValueError(f"SDK symbolic link differs: {relative}")
            elif member.isfile() or member.islnk():
                expected = package.extractfile(member)
                if expected is None or not path.is_file() or path.is_symlink():
                    raise ValueError(f"SDK file is missing or substituted: {relative}")
                with expected:
                    expected_digest = hashlib.file_digest(expected, "sha256").hexdigest()
                if sha256(path) != expected_digest:
                    raise ValueError(f"SDK file differs from official archive: {relative}")
            else:
                raise ValueError(f"unsupported SDK archive entry: {relative}")
    installed = {
        path.relative_to(sdk).as_posix()
        for path in sdk.rglob("*")
        if path.is_file() or path.is_symlink()
    }
    if installed != recorded:
        raise ValueError("SDK installation contains unrecorded or missing files")


def sdk_environment() -> tuple[dict[str, str], dict[str, object]]:
    """Return a deterministic C toolchain, never an ambient alternative."""
    key = host_platform()
    raw_archive = os.environ.get("WASI_SDK_ARCHIVE")
    raw_sdk = os.environ.get("WASI_SDK_PATH")
    if not raw_archive or not raw_sdk:
        raise ValueError("WASI_SDK_ARCHIVE and WASI_SDK_PATH must name the exact SDK 34 archive and extraction")
    archive, sdk = Path(raw_archive).resolve(), Path(raw_sdk).resolve()
    size, expected_digest = SDK_ASSETS[key]
    if not archive.is_file() or archive.stat().st_size != size or sha256(archive) != expected_digest:
        raise ValueError("WASI SDK archive does not match the official host release")
    verify_contents(archive, sdk)
    version_text = sdk_version_text(sdk)
    suffix = ".exe" if key.endswith("windows") else ""
    compiler, archiver = sdk / f"bin/clang{suffix}", sdk / f"bin/llvm-ar{suffix}"
    sysroot = sdk / "share/wasi-sysroot"
    environment = os.environ.copy()
    reject_c_overrides(environment)
    environment.update({
        "WASI_SDK_PATH": str(sdk),
        "WASI_SYSROOT": str(sysroot),
        "CC_wasm32_wasip2": str(compiler),
        "AR_wasm32_wasip2": str(archiver),
        "CFLAGS_wasm32_wasip2": f"--sysroot={sysroot}",
    })
    identity = {
        "version": SDK_VERSION,
        "version_file": version_text,
        "release": SDK_RELEASE,
        "platform": key,
        "archive": f"wasi-sdk-{SDK_VERSION}-{key}.tar.gz",
        "archive_sha256": expected_digest,
        "compiler_sha256": sha256(compiler),
        "compiler_version": subprocess.check_output([str(compiler), "--version"], text=True).splitlines()[0],
        "archiver_sha256": sha256(archiver),
        "sysroot_sha256": content_sha256(sysroot),
    }
    return environment, identity


def reject_c_overrides(environment: dict[str, str]) -> None:
    for name in environment:
        if name in {"CC", "CXX", "AR", "RANLIB", "WASI_SYSROOT"} or name.startswith(
            ("CC_", "CXX_", "AR_", "RANLIB_", "CFLAGS", "CXXFLAGS", "ARFLAGS", "SYNTAQLITE_CFLAG_", "TARGET_CC", "TARGET_AR")
        ):
            raise ValueError(f"ambient component build override is unsupported: {name}")


def sdk_version_text(sdk: Path) -> str:
    # The authenticated upstream file includes revisions after its version line.
    text = (sdk / "VERSION").read_text(encoding="utf-8")
    lines = text.splitlines()
    if not lines or lines[0] != SDK_VERSION:
        raise ValueError("WASI SDK VERSION first line must be exactly 34.0")
    return text
