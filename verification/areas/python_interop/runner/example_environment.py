"""One pinned uv project for independent example packages and native dependencies."""
from pathlib import Path
from env import RunnerPaths


def example_environment_root(paths: RunnerPaths) -> Path:
    root = paths.repo_root / "target/verification/areas/python_interop/examples"
    sources = {name: paths.area_root / name for name in (".venv", "pyproject.toml", "uv.lock")}
    for name, source in sources.items():
        if not (source.is_dir() if name == ".venv" else source.is_file()):
            raise RuntimeError(f"Python examples require the pinned area input: {source}")
    if root.is_symlink():
        raise RuntimeError(f"Python example environment root must not be an alias: {root}")
    root.mkdir(parents=True, exist_ok=True)
    for name, source in sources.items():
        link = root / name
        try:
            link.symlink_to(source, target_is_directory=name == ".venv")
        except FileExistsError:
            if not link.is_symlink() or link.readlink() != source:
                raise RuntimeError(f"Conflicting Python example environment input: {link}") from None
    return root
