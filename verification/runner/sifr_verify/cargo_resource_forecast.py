"""Cache presence informs allocation estimates, never assertion or build reuse.

Cargo always executes its original command and checks fingerprints itself. A
wrong or stale hint can exhaust the monitored allowance and fail preparation;
it cannot turn unexecuted work into a pass.
"""
from __future__ import annotations

import stat
import sys
import re
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


def test_cache_hint(root: Path, env: dict, package: str, *, include_library: bool = False) -> bool:
    debug = target_root(root, env) / 'debug'
    try:
        if debug.resolve(strict=True) != debug.absolute():
            return False
        name = package.replace('-', '_')
        library = include_library and any(regular(path) for path in (debug / 'deps').glob('lib' + name + '-*.rlib')) and any(
            regular(path) for path in (debug / '.fingerprint').glob(package + '-*/lib-' + name))
        return library or any(regular(path) and not path.suffix for path in (debug / 'deps').glob(name + '-*')) and any(
            regular(path) for path in (debug / '.fingerprint').glob(package + '-*/test-lib-' + name))
    except OSError:
        return False


def generated_preparation(command: list[str]) -> bool:
    """Recognize the complete wrapper, whose growth exceeds its outer CLI build.

    Materialization and locked fetch also populate revision-specific Git sources,
    submodules and native caches. An existing outer executable cannot forecast
    that closure; this command always needs its separate preparation allocation.
    """
    return (len(command) == 7 and command[0] == sys.executable
            and command[1:3] == ['-m', 'sifr_verify.generated_cargo_setup']
            and command[3] == '--profile'
            and command[4] in {'create-pr', 'merge', 'nightly', 'cloud'}
            and command[5] == '--revision'
            and re.fullmatch('[0-9a-f]{40}', command[6]) is not None)


def command_cache_hint(root: Path, env: dict, command: list[str], *, include_library: bool = False) -> bool:
    if not command or len(command) < 2:
        return False
    if command[0] != 'cargo':
        return False
    if '--target' in command or '--release' in command or '--manifest-path' in command:
        return False
    if command[1] == 'test' and '-p' in command:
        indices = [index+1 for index,value in enumerate(command) if value=='-p']
        if include_library:
            return all(index<len(command) and test_cache_hint(root,env,command[index],include_library=True) for index in indices)
        index = indices[0]
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
