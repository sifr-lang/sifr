"""Cache presence informs allocation estimates, never assertion or build reuse.

Cargo always executes its original command and checks fingerprints itself. A
wrong or stale hint can exhaust the monitored allowance and fail preparation;
it cannot turn unexecuted work into a pass.
"""
from __future__ import annotations

import stat
from pathlib import Path


def target_root(root: Path, env: dict) -> Path:
    path = Path(env.get('CARGO_TARGET_DIR', root / 'target'))
    return path if path.is_absolute() else root / path


def regular(path: Path) -> bool:
    try:
        info = path.lstat()
        return stat.S_ISREG(info.st_mode) and info.st_size > 0
    except OSError:
        return False


def test_cache_hint(root: Path, env: dict, package: str) -> bool:
    debug = target_root(root, env) / 'debug'
    try:
        if debug.resolve(strict=True) != debug.absolute():
            return False
        name = package.replace('-', '_')
        return any(regular(path) and not path.suffix for path in (debug / 'deps').glob(name + '-*')) and any(
            regular(path) for path in (debug / '.fingerprint').glob(package + '-*/test-lib-' + name))
    except OSError:
        return False


def command_cache_hint(root: Path, env: dict, command: list[str]) -> bool:
    if not command or command[0] != 'cargo' or len(command) < 2:
        return False
    if '--target' in command or '--release' in command or '--manifest-path' in command:
        return False
    if command[1] == 'test' and '-p' in command:
        index = command.index('-p') + 1
        return index < len(command) and test_cache_hint(root, env, command[index])
    if command[1] == 'build':
        if '--bin' in command:
            index = command.index('--bin') + 1
        elif '-p' in command:
            index = command.index('-p') + 1
        else:
            return False
        if index >= len(command):
            return False
        path = target_root(root, env) / 'debug' / command[index]
        try:
            return path.resolve(strict=True) == path.absolute() and regular(path)
        except OSError:
            return False
    return False
