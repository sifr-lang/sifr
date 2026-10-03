"""Watch an admitted preparation's filesystem floor without touching its cache."""
from __future__ import annotations

import errno
import shutil
from dataclasses import dataclass
from pathlib import Path

FLOOR_VARIABLE = 'SIFR_VERIFY_DISK_FLOOR_BYTES'
PATH_VARIABLE = 'SIFR_VERIFY_DISK_FLOOR_PATH'


@dataclass(frozen=True)
class DiskBudget:
    path: Path
    floor: int

    @classmethod
    def from_environment(cls, env):
        if FLOOR_VARIABLE not in env and PATH_VARIABLE not in env:
            return None
        try:
            raw = env[FLOOR_VARIABLE]
            if not raw.isdecimal() or int(raw) <= 0:
                raise ValueError('disk floor must be positive integer bytes')
            path = Path(env[PATH_VARIABLE])
            if not path.is_absolute() or path.resolve(strict=True) != path:
                raise ValueError('disk floor path must be canonical and absolute')
            return cls(path, int(raw))
        except (KeyError, ValueError, OSError) as error:
            raise ValueError(f'invalid prospective disk budget: {error}') from error

    def check(self):
        free = shutil.disk_usage(self.path).free
        if free < self.floor:
            raise OSError(errno.ENOSPC,
                f'declared preparation disk budget exhausted: free={free} floor={self.floor}', str(self.path))
