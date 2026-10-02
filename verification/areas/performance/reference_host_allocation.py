"""Explicit managed-Linux reference allocation, measured from cgroup v2."""

from pathlib import Path


def positive_integer(value: str, field: str) -> int:
    if not value.isascii() or not value.isdecimal() or int(value) <= 0:
        raise ValueError(f"managed Linux reference requires finite positive {field}")
    return int(value)


def exposed_limit(directory: Path, field: str, *, required: bool) -> str | None:
    try:
        return (directory / field).read_text().strip()
    except FileNotFoundError:
        if required:
            raise
        return None


def managed_allocation(
    root: Path = Path("/sys/fs/cgroup"),
    membership: Path = Path("/proc/self/cgroup"),
) -> dict:
    """Bind this process's finite limits and every exposed ancestor control."""
    try:
        entries = membership.read_text().splitlines()
        unified = [row[3:] for row in entries if row.startswith("0::")]
        if len(unified) != 1 or not unified[0].startswith("/"):
            raise ValueError("managed Linux reference requires unified cgroup v2 membership")
        relative = Path(unified[0].lstrip("/"))
        if ".." in relative.parts:
            raise ValueError("managed Linux reference has invalid cgroup membership")
        current = root / relative
        controls = []
        for directory in [current, *current.parents]:
            if directory == root.parent:
                break
            required = directory == current
            cpu_text = exposed_limit(directory, "cpu.max", required=required)
            memory = exposed_limit(directory, "memory.max", required=required)
            quota = period = None
            if cpu_text is not None:
                cpu = cpu_text.split()
                if len(cpu) != 2:
                    raise ValueError("managed Linux reference has invalid cpu.max")
                period = positive_integer(cpu[1], "CPU period")
                quota = None if cpu[0] == "max" else positive_integer(cpu[0], "CPU quota")
            capacity = None if memory in (None, "max") else positive_integer(memory, "memory limit")
            if directory == current and (quota is None or capacity is None):
                raise ValueError("managed Linux reference requires finite CPU and memory limits")
            controls.append({
                "path": str(directory.relative_to(root)),
                "cpu_quota_us": quota, "cpu_period_us": period,
                "memory_max_bytes": capacity,
                "additional_controls": {
                    field: (directory / field).read_text().strip()
                    for field in ("cpu.max.burst", "cpu.weight", "memory.high", "memory.swap.max")
                    if (directory / field).is_file()
                },
            })
            if directory == root:
                break
        return {"source": "cgroup-v2", "membership": unified[0], "controls": controls}
    except OSError as error:
        raise ValueError(f"managed Linux reference allocation unavailable: {error}") from error
