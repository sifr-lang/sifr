"""Read effective capacity for profile admission, without scheduling work."""
from __future__ import annotations

from dataclasses import dataclass
import os
from pathlib import Path
import platform
import re
import subprocess

from .errors import VerificationError


@dataclass(frozen=True)
class Capacity:
    cpu_count: int
    available_memory_bytes: int


def linux_capacity(proc: Path = Path('/proc'), cgroup: Path = Path('/sys/fs/cgroup')) -> Capacity:
    cpus = len(os.sched_getaffinity(0))
    values = dict(line.split(':', 1) for line in (proc / 'meminfo').read_text().splitlines())
    memory = int(values['MemAvailable'].split()[0]) * 1024
    entries = (proc / 'self/cgroup').read_text().splitlines()
    unified = [line[3:] for line in entries if line.startswith('0::')]
    if not unified:
        raise VerificationError('profile admission requires cgroup v2 capacity accounting on Linux')
    # The mount may expose a subtree, including in a container namespace.
    mounts = [line.split() for line in (proc / 'self/mountinfo').read_text().splitlines()
              if ' - cgroup2 ' in line]
    if len(mounts) != 1:
        raise VerificationError('cannot identify the effective cgroup v2 mount')
    mount_root = Path(mounts[0][3])
    group = Path(unified[0])
    try:
        relative = group.relative_to(mount_root)
    except ValueError as exc:
        raise VerificationError('cgroup path is outside the visible mount') from exc
    directory = cgroup / relative
    if not directory.is_dir():
        raise VerificationError('effective cgroup directory is unavailable')
    while True:
        quota_file = directory / 'cpu.max'
        if quota_file.exists():
            quota, period = quota_file.read_text().split()
            if quota != 'max':
                cpus = min(cpus, max(1, int(quota) // int(period)))
        limit_file = directory / 'memory.max'
        if limit_file.exists():
            limit = limit_file.read_text().strip()
            if limit != 'max':
                current = int((directory / 'memory.current').read_text())
                stats = dict(line.split() for line in (directory / 'memory.stat').read_text().splitlines())
                # Warm Cargo cache is reclaimable, not committed process memory.
                # Keep dirty/writeback pages charged until their I/O completes.
                reclaimable = max(0, int(stats['inactive_file'])
                                  - int(stats['file_dirty']) - int(stats['file_writeback']))
                used = max(0, current - reclaimable)
                memory = min(memory, max(0, int(limit) - used))
        if directory == cgroup:
            break
        directory = directory.parent
    return Capacity(cpus, memory)


def discover_capacity() -> Capacity:
    try:
        system = platform.system()
        if system == 'Linux':
            return linux_capacity()
        cpus = os.process_cpu_count() or 1
        if system == 'Darwin':
            result = subprocess.run(['vm_stat'], text=True, capture_output=True,
                                    check=True, timeout=10).stdout
            page_size = int(re.search(r'page size of (\d+) bytes', result)[1])
            pages = dict(re.findall(r'^(Pages [^:]+):\s+(\d+)\.', result, re.M))
            # Free + inactive memory is conservative; do not count compressed pages.
            memory = (int(pages['Pages free']) + int(pages['Pages inactive'])) * page_size
            return Capacity(cpus, memory)
        if system == 'Windows':
            import ctypes
            from ctypes import wintypes

            class MemoryStatus(ctypes.Structure):
                _fields_ = [('length', wintypes.DWORD), ('load', wintypes.DWORD)] + [
                    (name, ctypes.c_ulonglong) for name in
                    ('total_physical', 'available_physical', 'total_page', 'available_page',
                     'total_virtual', 'available_virtual', 'extended')]

            status = MemoryStatus()
            status.length = ctypes.sizeof(status)
            if not ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status)):
                raise OSError('GlobalMemoryStatusEx failed')
            return Capacity(cpus, status.available_physical)
        raise VerificationError(f'profile capacity discovery is unsupported on {system}')
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as exc:
        raise VerificationError(f'cannot read profile execution capacity: {exc}') from exc
