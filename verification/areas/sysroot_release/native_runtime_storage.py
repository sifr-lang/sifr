"""Owned physical paths for the separate published-runtime diagnostic."""
import os
from pathlib import Path
import stat
import tomllib

from native_capacity import storage
from sifr_verify.process_disk_budget import DiskBudget
from native_preparation_storage import (canonical, topology, same_volume_tree,
                                       parent_temporary, TOOL_OVERRIDES, TEMP_VARIABLES)

GIB = 1024**3
NAMES = ('temporary', 'cargo-home', 'rustup-home', 'target', 'sifr-cache',
         'bridge-cache', 'clang-cache', 'evidence')
ALLOWED_CARGO = {'CARGO_HOME', 'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS', 'CARGO_INCREMENTAL',
                 'CARGO_NET_OFFLINE', 'CARGO_TERM_COLOR', 'CARGO_PROFILE_DEV_DEBUG'}


def requirements(asset):
    return dict(memory_peak_bytes=768*1024**2, memory_reserve_bytes=2*GIB,
                tmpfs_growth_bytes=0, disk_reserve_bytes=8*GIB,
                disk_growth_bytes=asset['size']+4*GIB+128*1024**2+2*GIB+GIB//2+3*GIB+GIB,
                retained_copy_bytes=GIB//4)


def configuration(root, env):
    for name, value in env.items():
        if not value:
            continue
        if ((name.startswith('CARGO_') and name not in ALLOWED_CARGO)
                or name in TOOL_OVERRIDES or name in ('RUSTC', 'SIFR_CARGO', 'SIFR_RUSTC',
                'SIFR_SYSROOT', 'SIFR_SYSROOT_MODE', 'LD_PRELOAD', 'DYLD_INSERT_LIBRARIES',
                'SDKROOT', 'DEVELOPER_DIR', 'MACOSX_DEPLOYMENT_TARGET')
                or (name.startswith('RUSTUP_') and name not in ('RUSTUP_HOME', 'RUSTUP_TOOLCHAIN'))
                or name.startswith('DYLD_') or name in ('LD_LIBRARY_PATH', 'SIFR_TARGET')
                or name.startswith(('CCACHE_', 'SCCACHE_'))
                or any(name.startswith(p+'_') or name.endswith('_'+p)
                       for p in ('CC', 'CXX', 'AR', 'RANLIB', 'CFLAGS', 'CXXFLAGS', 'LDFLAGS'))):
            raise ValueError('uncontrolled runtime configuration: '+name)
    for directory in (root, *root.parents, root/'dependency', root/'project', root/'package', root/'cargo-home'):
        for suffix in ('.cargo/config', '.cargo/config.toml'):
            path = directory/suffix
            if path.exists() or path.is_symlink():
                if (path == root/'package/.cargo/config.toml' and not path.is_symlink()
                        and tomllib.loads(path.read_text()) == {'source': {'crates-io': {'replace-with': 'sifr-vendor'},
                            'sifr-vendor': {'directory': 'vendor'}}}):
                    continue
                raise ValueError('uncontrolled runtime Cargo configuration: '+str(path))
    for name in ('config', 'config.toml'):
        if (root/'cargo-home'/name).exists() or (root/'cargo-home'/name).is_symlink():
            raise ValueError('uncontrolled Cargo home configuration')


def bindings(root):
    paths = {name: root/name for name in NAMES}
    return {name: str(paths['temporary']) for name in TEMP_VARIABLES} | {
        'CARGO_HOME': str(paths['cargo-home']), 'RUSTUP_HOME': str(paths['rustup-home']),
        'CARGO_TARGET_DIR': str(paths['target']), 'SIFR_CACHE_DIR': str(paths['sifr-cache']),
        'SIFR_RUST_BRIDGE_PROBE_CACHE_DIR': str(paths['bridge-cache']),
        'CLANG_MODULE_CACHE_PATH': str(paths['clang-cache']),
        'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0',
        'CARGO_NET_OFFLINE': 'true', 'RUSTUP_TOOLCHAIN': '1.98.1',
        'PYTHONDONTWRITEBYTECODE': '1', 'LC_ALL': 'C',
        'PATH': str(root/'rustup-home/toolchains/1.98.1-aarch64-apple-darwin/bin')+':/usr/bin:/bin:/usr/sbin:/sbin',
        'SIFR_VERIFY_DISK_FLOOR_PATH': str(root), 'SIFR_VERIFY_DISK_FLOOR_BYTES': str(8*GIB)}


def environment(root, inherited):
    configuration(root, inherited)
    prior = DiskBudget.from_environment(inherited)
    if prior is not None and prior.path != root:
        raise ValueError('inherited diagnostic disk floor must name the same owned root')
    env = dict(inherited) | bindings(root)
    if prior is not None:
        env['SIFR_VERIFY_DISK_FLOOR_BYTES'] = str(max(prior.floor, 8*GIB))
    return env


def observe(root, env):
    root = canonical(root)
    configuration(root, env)
    rows = {}
    for path in (root, *(root/name for name in NAMES)):
        canonical(path)
        info = path.stat()
        if not path.is_dir() or info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) & 0o077:
            raise ValueError('runtime root must be private and owned')
        if info.st_dev != root.stat().st_dev:
            raise ValueError('runtime root moved to another volume')
        rows[str(path.relative_to(root))] = {'device': info.st_dev, 'inode': info.st_ino,
            'uid': info.st_uid, 'mode': stat.S_IMODE(info.st_mode), 'storage': topology(storage(path))}
    if any(row['storage'] != rows['.']['storage'] for row in rows.values()):
        raise ValueError('runtime storage authority differs')
    same_volume_tree(root, root.stat().st_dev)
    return {'paths': rows, 'environment': bindings(root)}


def prepare(root, inherited):
    root.mkdir(mode=0o700, exist_ok=False)
    for name in NAMES:
        (root/name).mkdir(mode=0o700)
    env = environment(root, inherited)
    return env, observe(root, env)


def check(record, root, env):
    if record != observe(root, env):
        raise ValueError('runtime physical storage proof differs')
